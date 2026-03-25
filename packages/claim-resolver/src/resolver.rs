//! On-chain parametric trigger confirmation.
//!
//! When a claim is opened for a price-based cover (Depeg, Oracle), the resolver
//! confirms the parametric condition directly from a Switchboard On-Demand pull
//! feed before approving an automatic payout -- no human in the loop. For cover
//! types whose signal lives off chain (Exploit / Slashing / Liquidation), a
//! registered keeper submits a signed attestation instead; the pure
//! confirmation predicates used by both paths live here so they can be tested
//! without a live feed.

use anchor_lang::prelude::*;
use switchboard_on_demand::PullFeedAccountData;

use crate::error::ResolverError;

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// A normalised price read from an on-demand feed.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeedPrice {
    /// Signed mantissa.
    pub mantissa: i64,
    /// Base-10 exponent.
    pub expo: i32,
}

/// Read a Switchboard On-Demand pull feed value, enforcing staleness.
pub fn read_feed(account: &AccountInfo, clock_slot: u64) -> Result<FeedPrice> {
    let data = account.try_borrow_data()?;
    let feed = PullFeedAccountData::parse(data).map_err(|_| error!(ResolverError::OracleRead))?;
    let value = feed
        .value(clock_slot)
        .map_err(|_| error!(ResolverError::OracleRead))?;
    let mantissa = i64::try_from(value.mantissa()).map_err(|_| error!(ResolverError::OracleRead))?;
    Ok(FeedPrice {
        mantissa,
        expo: -(value.scale() as i32),
    })
}

/// Rescale a feed price to a target exponent.
pub fn rescale(price: FeedPrice, target_expo: i32) -> Option<i64> {
    if price.expo == target_expo {
        return Some(price.mantissa);
    }
    let diff = price.expo - target_expo;
    if diff > 0 {
        price.mantissa.checked_mul(10i64.checked_pow(diff as u32)?)
    } else {
        Some(price.mantissa / 10i64.checked_pow((-diff) as u32)?)
    }
}

/// Confirm a depeg: price out of the inclusive `[lower, upper]` band at `expo`.
pub fn confirm_depeg(price: FeedPrice, lower: i64, upper: i64, expo: i32) -> Result<u64> {
    let p = rescale(price, expo).ok_or(error!(ResolverError::Overflow))?;
    let peg_mid = (lower + upper) / 2;
    let depth = if p < lower {
        (lower - p).max(0)
    } else if p > upper {
        (p - upper).max(0)
    } else {
        return err!(ResolverError::TriggerNotMet);
    };
    Ok(((depth as i128 * BPS as i128) / peg_mid.max(1) as i128) as u64)
}

/// Confirm an oracle divergence between two same-purpose feeds.
pub fn confirm_divergence(a: FeedPrice, b: FeedPrice, expo: i32, threshold_bps: u64) -> Result<u64> {
    let pa = rescale(a, expo).ok_or(error!(ResolverError::Overflow))?;
    let pb = rescale(b, expo).ok_or(error!(ResolverError::Overflow))?;
    let diff = (pa as i128 - pb as i128).unsigned_abs();
    let denom = (pa.unsigned_abs().min(pb.unsigned_abs())).max(1) as u128;
    let bps = ((diff * BPS as u128) / denom) as u64;
    if bps < threshold_bps {
        return err!(ResolverError::TriggerNotMet);
    }
    Ok(bps)
}

/// Payout (asset units) for a confirmed trigger: `depth_bps` of coverage, capped.
pub fn scaled_payout(coverage: u64, depth_bps: u64) -> u64 {
    (((coverage as u128) * depth_bps.min(BPS) as u128) / BPS as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rescale_changes_exponent() {
        let p = FeedPrice {
            mantissa: 100,
            expo: -2,
        };
        assert_eq!(rescale(p, -4).unwrap(), 10_000);
    }

    #[test]
    fn depeg_depth_only_out_of_band() {
        let out = FeedPrice {
            mantissa: 90_000_000,
            expo: -8,
        };
        assert!(confirm_depeg(out, 95_000_000, 105_000_000, -8).unwrap() > 0);
        let inb = FeedPrice {
            mantissa: 100_000_000,
            expo: -8,
        };
        assert!(confirm_depeg(inb, 95_000_000, 105_000_000, -8).is_err());
    }

    #[test]
    fn divergence_threshold_enforced() {
        let a = FeedPrice {
            mantissa: 100_000_000,
            expo: -8,
        };
        let b = FeedPrice {
            mantissa: 104_000_000,
            expo: -8,
        };
        assert_eq!(confirm_divergence(a, b, -8, 200).unwrap(), 400);
        assert!(confirm_divergence(a, a, -8, 200).is_err());
    }

    #[test]
    fn payout_caps_at_full_coverage() {
        assert_eq!(scaled_payout(1_000_000, 500), 50_000);
        assert_eq!(scaled_payout(1_000_000, 20_000), 1_000_000);
    }
}

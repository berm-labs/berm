//! DepegCover trigger.
//!
//! A stablecoin depeg is parametrically defined as the aggregate price leaving
//! the `[lower_bound, upper_bound]` peg band and *staying* out of band for at
//! least `persistence_slots` consecutive slots. Persistence guards against a
//! single-slot oracle glitch arming the cover. The payout scales with the depth
//! of the depeg -- how far past the band the price travelled -- so a 0.90 print
//! pays more than a 0.94 print.

use crate::cover_type::{DepegParams, BPS};
use crate::policy::Policy;

use super::{MarketSnapshot, PegReading, TriggerOutcome};

/// Rescale a `(price, expo)` to the params' exponent for comparison.
fn rescale_to(price: i64, from_expo: i32, to_expo: i32) -> i64 {
    if from_expo == to_expo {
        price
    } else if from_expo > to_expo {
        price.saturating_mul(10i64.saturating_pow((from_expo - to_expo) as u32))
    } else {
        price / 10i64.saturating_pow((to_expo - from_expo) as u32)
    }
}

/// Depeg depth in bps relative to the nearer peg bound (0 if in band).
pub fn depeg_depth_bps(params: &DepegParams, reading: &PegReading) -> u64 {
    let price = rescale_to(reading.price, reading.expo, params.expo);
    let peg_mid = (params.lower_bound + params.upper_bound) / 2;
    if price < params.lower_bound {
        let dev = (params.lower_bound - price).max(0) as u128;
        ((dev * BPS as u128) / peg_mid.max(1) as u128) as u64
    } else if price > params.upper_bound {
        let dev = (price - params.upper_bound).max(0) as u128;
        ((dev * BPS as u128) / peg_mid.max(1) as u128) as u64
    } else {
        0
    }
}

/// Whether the reading meets the depeg trigger (out of band + persistent).
pub fn is_depeg(params: &DepegParams, reading: &PegReading) -> bool {
    depeg_depth_bps(params, reading) > 0 && reading.out_of_band_slots >= params.persistence_slots
}

/// Evaluate a depeg policy against a snapshot.
pub fn evaluate(
    policy: &Policy,
    params: &DepegParams,
    snap: &MarketSnapshot,
) -> Option<TriggerOutcome> {
    let reading = snap.peg.get(&policy.subject)?;
    if !is_depeg(params, reading) {
        return None;
    }
    let depth = depeg_depth_bps(params, reading);
    let payout = ((policy.coverage as u128 * depth as u128) / BPS as u128) as u64;
    Some(TriggerOutcome::new(
        policy,
        payout,
        snap.slot,
        format!(
            "{} depegged {depth} bps for {} slots",
            policy.subject, reading.out_of_band_slots
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cover_type::CoverParams;
    use crate::policy::PolicyStatus;

    fn policy() -> Policy {
        Policy {
            id: 2,
            holder: "H".into(),
            subject: "USDC/USD".into(),
            coverage: 1_000_000,
            params: CoverParams::Depeg(DepegParams::default()),
            start_slot: 0,
            end_slot: 100_000,
            status: PolicyStatus::Active,
        }
    }

    #[test]
    fn fires_on_persistent_depeg() {
        let mut snap = MarketSnapshot::at(50);
        snap.peg.insert(
            "USDC/USD".into(),
            PegReading {
                price: 90_000_000, // 0.90 at expo -8, below 0.95 band
                expo: -8,
                out_of_band_slots: 2_000, // > 1_200 persistence
            },
        );
        let o = evaluate(&policy(), &DepegParams::default(), &snap).unwrap();
        assert!(o.payout > 0);
    }

    #[test]
    fn ignores_transient_glitch() {
        let mut snap = MarketSnapshot::at(50);
        snap.peg.insert(
            "USDC/USD".into(),
            PegReading {
                price: 90_000_000,
                expo: -8,
                out_of_band_slots: 5, // below persistence threshold
            },
        );
        assert!(evaluate(&policy(), &DepegParams::default(), &snap).is_none());
    }

    #[test]
    fn in_band_price_is_not_a_depeg() {
        let p = DepegParams::default();
        let r = PegReading {
            price: 100_000_000,
            expo: -8,
            out_of_band_slots: 5_000,
        };
        assert_eq!(depeg_depth_bps(&p, &r), 0);
        assert!(!is_depeg(&p, &r));
    }
}

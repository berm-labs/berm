//! Normalised price representation shared by every oracle source.
//!
//! Pyth, Switchboard, and Chainlink each express prices with their own scaling
//! convention (Pyth uses a signed exponent, Switchboard uses an 18-decimal
//! fixed point, Chainlink OCR2 uses a configurable `decimals` field). To compare
//! them we normalise everything into a single [`NormalizedPrice`] expressed as a
//! signed mantissa with a base-10 exponent, plus a confidence interval in the
//! same scale. This mirrors the design used by Pyth's `Price` struct.

use serde::{Deserialize, Serialize};

use crate::error::{OracleError, OracleResult};

/// One basis point = 1/10_000.
pub const BPS_DENOMINATOR: u64 = 10_000;

/// A price normalised to a signed mantissa and base-10 exponent.
///
/// The real value is `price * 10^expo`. Confidence is the one-sigma interval in
/// the same scale. `publish_slot` is the Solana slot at which the source last
/// updated, used by [`crate::staleness`] to enforce freshness budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedPrice {
    /// Signed price mantissa.
    pub price: i64,
    /// One-sigma confidence interval mantissa (always non-negative).
    pub conf: u64,
    /// Base-10 exponent applied to both `price` and `conf`.
    pub expo: i32,
    /// Solana slot at which this observation was published.
    pub publish_slot: u64,
}

impl NormalizedPrice {
    /// Build a price, rejecting non-finite mantissas.
    pub fn new(price: i64, conf: u64, expo: i32, publish_slot: u64) -> Self {
        Self {
            price,
            conf,
            expo,
            publish_slot,
        }
    }

    /// Re-express this price at a target exponent without losing the value, used
    /// to put two sources on a common scale before comparison.
    pub fn rescale(&self, target_expo: i32, feed: &str) -> OracleResult<NormalizedPrice> {
        let (price, conf) = rescale_mantissa(self.price, self.conf, self.expo, target_expo, feed)?;
        Ok(NormalizedPrice {
            price,
            conf,
            expo: target_expo,
            publish_slot: self.publish_slot,
        })
    }

    /// Confidence-to-price ratio in basis points, used for confidence gating.
    pub fn confidence_bps(&self) -> u64 {
        let abs_price = self.price.unsigned_abs();
        if abs_price == 0 {
            return u64::MAX;
        }
        // (conf / price) * 10_000, computed in u128 to avoid overflow.
        ((self.conf as u128 * BPS_DENOMINATOR as u128) / abs_price as u128) as u64
    }

    /// Absolute value of the price mantissa as an unsigned integer.
    pub fn abs_price(&self) -> u64 {
        self.price.unsigned_abs()
    }

    /// Render the price as an `f64` for reporting and back-test display only.
    /// Never use this for on-chain or settlement math.
    pub fn to_f64(&self) -> f64 {
        self.price as f64 * 10f64.powi(self.expo)
    }

    /// Lower bound of the price given the confidence interval.
    pub fn lower_bound(&self) -> i64 {
        self.price.saturating_sub(self.conf as i64)
    }

    /// Upper bound of the price given the confidence interval.
    pub fn upper_bound(&self) -> i64 {
        self.price.saturating_add(self.conf as i64)
    }
}

/// Rescale a `(price, conf)` mantissa pair from `from_expo` to `to_expo`.
///
/// When the target exponent is smaller we multiply (more decimals); when it is
/// larger we divide (fewer decimals). Multiplication is checked so that a hostile
/// or buggy feed cannot wrap a mantissa silently.
pub fn rescale_mantissa(
    price: i64,
    conf: u64,
    from_expo: i32,
    to_expo: i32,
    feed: &str,
) -> OracleResult<(i64, u64)> {
    if from_expo == to_expo {
        return Ok((price, conf));
    }
    let diff = from_expo - to_expo;
    if diff > 0 {
        let factor = pow10(diff as u32, feed)?;
        let p = (price as i128)
            .checked_mul(factor as i128)
            .ok_or_else(|| OracleError::Overflow { feed: feed.into() })?;
        let c = (conf as u128)
            .checked_mul(factor as u128)
            .ok_or_else(|| OracleError::Overflow { feed: feed.into() })?;
        let p = i64::try_from(p).map_err(|_| OracleError::Overflow { feed: feed.into() })?;
        let c = u64::try_from(c).map_err(|_| OracleError::Overflow { feed: feed.into() })?;
        Ok((p, c))
    } else {
        let factor = pow10((-diff) as u32, feed)?;
        Ok((price / factor as i64, conf / factor))
    }
}

/// Compute `10^n` as a `u64`, erroring on overflow rather than wrapping.
pub fn pow10(n: u32, feed: &str) -> OracleResult<u64> {
    10u64
        .checked_pow(n)
        .ok_or_else(|| OracleError::Overflow { feed: feed.into() })
}

/// Absolute divergence between two same-scale prices expressed in basis points
/// relative to the smaller magnitude (the conservative denominator).
pub fn divergence_bps(a: i64, b: i64) -> u64 {
    let diff = (a as i128 - b as i128).unsigned_abs();
    let denom = a.unsigned_abs().min(b.unsigned_abs()).max(1) as u128;
    ((diff * BPS_DENOMINATOR as u128) / denom) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rescales_without_loss_when_widening_decimals() {
        let p = NormalizedPrice::new(100, 5, -2, 10); // 1.00
        let r = p.rescale(-4, "X").unwrap();
        assert_eq!(r.price, 10_000);
        assert_eq!(r.conf, 500);
        assert!((r.to_f64() - 1.00).abs() < 1e-9);
    }

    #[test]
    fn confidence_ratio_in_bps() {
        let p = NormalizedPrice::new(10_000, 50, -4, 1); // price 1.0, conf 0.005
        assert_eq!(p.confidence_bps(), 50); // 0.5%
    }

    #[test]
    fn divergence_is_symmetric_and_relative() {
        assert_eq!(divergence_bps(10_000, 10_100), divergence_bps(10_100, 10_000));
        // 100/10_000 = 1% = 100 bps relative to the smaller value.
        assert_eq!(divergence_bps(10_100, 10_000), 100);
    }

    #[test]
    fn pow10_rejects_overflow() {
        assert!(pow10(64, "X").is_err());
        assert_eq!(pow10(3, "X").unwrap(), 1000);
    }
}

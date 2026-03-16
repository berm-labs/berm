//! Freshness and confidence gating.
//!
//! Parametric cover must never settle on a price that the market no longer
//! believes. Following the Pyth Lazer guidance, every observation passes through
//! two gates before it is eligible to contribute to an aggregate:
//!
//! 1. **Staleness** -- the observation must be younger than `max_age_slots`.
//! 2. **Confidence** -- the one-sigma interval must be tighter than
//!    `max_confidence_bps` relative to the price.
//!
//! A price that fails either gate is discarded from aggregation, which can in
//! turn arm an `OracleCover` policy if too many sources drop out at once.

use serde::{Deserialize, Serialize};

use crate::error::{OracleError, OracleResult};
use crate::price::NormalizedPrice;

/// Tunable freshness / confidence budget for a single feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StalenessPolicy {
    /// Maximum tolerated age, in slots, of the latest observation.
    pub max_age_slots: u64,
    /// Maximum tolerated confidence-to-price ratio, in basis points.
    pub max_confidence_bps: u64,
}

impl Default for StalenessPolicy {
    fn default() -> Self {
        // ~25 slots ≈ 10s at Solana slot times; 50 bps ≈ 0.5% confidence band.
        Self {
            max_age_slots: 50,
            max_confidence_bps: 50,
        }
    }
}

impl StalenessPolicy {
    /// Construct a custom policy.
    pub fn new(max_age_slots: u64, max_confidence_bps: u64) -> Self {
        Self {
            max_age_slots,
            max_confidence_bps,
        }
    }

    /// Age of an observation relative to `current_slot`, saturating at zero for
    /// observations that claim a slot in the future (clock skew).
    pub fn age_of(&self, price: &NormalizedPrice, current_slot: u64) -> u64 {
        current_slot.saturating_sub(price.publish_slot)
    }

    /// Returns `Ok` if the observation is fresh enough.
    pub fn check_age(
        &self,
        feed: &str,
        price: &NormalizedPrice,
        current_slot: u64,
    ) -> OracleResult<()> {
        let age = self.age_of(price, current_slot);
        if age > self.max_age_slots {
            return Err(OracleError::Stale {
                feed: feed.to_string(),
                age_slots: age,
                max_slots: self.max_age_slots,
            });
        }
        Ok(())
    }

    /// Returns `Ok` if the confidence interval is tight enough.
    pub fn check_confidence(&self, feed: &str, price: &NormalizedPrice) -> OracleResult<()> {
        let ratio = price.confidence_bps();
        if ratio > self.max_confidence_bps {
            return Err(OracleError::LowConfidence {
                feed: feed.to_string(),
                ratio_bps: ratio,
                max_bps: self.max_confidence_bps,
            });
        }
        Ok(())
    }

    /// Apply both gates. Used by the aggregator to filter healthy sources.
    pub fn validate(
        &self,
        feed: &str,
        price: &NormalizedPrice,
        current_slot: u64,
    ) -> OracleResult<()> {
        self.check_age(feed, price, current_slot)?;
        self.check_confidence(feed, price)?;
        Ok(())
    }

    /// Convenience predicate form of [`StalenessPolicy::validate`].
    pub fn is_healthy(&self, feed: &str, price: &NormalizedPrice, current_slot: u64) -> bool {
        self.validate(feed, price, current_slot).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn px(conf: u64, slot: u64) -> NormalizedPrice {
        NormalizedPrice::new(1_000_000, conf, -6, slot)
    }

    #[test]
    fn rejects_stale_observation() {
        let pol = StalenessPolicy::new(10, 100);
        let p = px(100, 100);
        assert!(pol.check_age("X", &p, 130).is_err());
        assert!(pol.check_age("X", &p, 105).is_ok());
    }

    #[test]
    fn rejects_wide_confidence() {
        let pol = StalenessPolicy::new(100, 50);
        // conf 10_000 on price 1_000_000 = 100 bps > 50 bps.
        assert!(pol.check_confidence("X", &px(10_000, 1)).is_err());
        assert!(pol.check_confidence("X", &px(2_000, 1)).is_ok());
    }

    #[test]
    fn future_slot_clamps_to_zero_age() {
        let pol = StalenessPolicy::default();
        assert_eq!(pol.age_of(&px(1, 200), 100), 0);
    }
}

//! Utilisation-based rate model.
//!
//! A cover pool's marginal price must rise as its capital gets committed, both to
//! protect solvency and to incentivise fresh LP capital -- the same dynamic as a
//! lending market's interest-rate curve. We use a two-slope (kinked) curve: rates
//! climb gently below an optimal utilisation, then steeply above it. This is the
//! Aave/Compound "jump rate" shape applied to cover capacity.

use serde::{Deserialize, Serialize};

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// Parameters of the kinked utilisation curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtilizationCurve {
    /// Utilisation (bps) at which the slope kinks upward.
    pub optimal_bps: u64,
    /// Multiplier (bps over 1.0) applied per unit utilisation below the kink.
    pub slope_low_bps: u64,
    /// Multiplier (bps over 1.0) applied per unit utilisation above the kink.
    pub slope_high_bps: u64,
}

impl Default for UtilizationCurve {
    fn default() -> Self {
        Self {
            optimal_bps: 8_000,   // 80%
            slope_low_bps: 2_000, // up to +20% at the kink
            slope_high_bps: 30_000, // steep beyond the kink
        }
    }
}

impl UtilizationCurve {
    /// Multiplier (in bps, where 10_000 = 1.0x) applied to the base premium at a
    /// given utilisation. Always >= 1.0x.
    pub fn multiplier_bps(&self, utilization_bps: u64) -> u64 {
        let u = utilization_bps.min(BPS);
        if u <= self.optimal_bps {
            // 1.0 + slope_low * (u / optimal)
            let extra = self.slope_low_bps * u / self.optimal_bps.max(1);
            BPS + extra
        } else {
            let at_kink = BPS + self.slope_low_bps;
            let over = u - self.optimal_bps;
            let span = (BPS - self.optimal_bps).max(1);
            let extra = self.slope_high_bps * over / span;
            at_kink + extra
        }
    }
}

/// Compute utilisation (bps) given committed and total capital.
pub fn utilization_bps(committed: u64, total_capital: u64) -> u64 {
    if total_capital == 0 {
        return BPS; // a pool with no capital is fully "utilised" -- closed.
    }
    ((committed as u128 * BPS as u128) / total_capital as u128).min(BPS as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplier_is_one_at_zero_utilization() {
        let c = UtilizationCurve::default();
        assert_eq!(c.multiplier_bps(0), BPS);
    }

    #[test]
    fn multiplier_climbs_with_utilization() {
        let c = UtilizationCurve::default();
        let low = c.multiplier_bps(4_000);
        let kink = c.multiplier_bps(8_000);
        let high = c.multiplier_bps(9_500);
        assert!(low < kink);
        assert!(kink < high);
        // Above the kink the slope is much steeper.
        assert!(high - kink > kink - low);
    }

    #[test]
    fn utilization_clamps_and_handles_empty_pool() {
        assert_eq!(utilization_bps(50, 100), 5_000);
        assert_eq!(utilization_bps(0, 0), BPS);
        assert_eq!(utilization_bps(200, 100), BPS);
    }
}

//! On-chain premium math.
//!
//! Mirrors the off-chain `berm-risk-underwriter` pricing in integer basis-point
//! arithmetic so a premium quoted by the marketplace backend can be recomputed
//! and verified on chain at purchase time. Keeping the formula identical on both
//! sides prevents a malicious client from under-paying.

/// Basis-point denominator.
pub const BPS: u64 = 10_000;

/// Days per pricing year.
pub const DAYS_PER_YEAR: u64 = 365;

/// Maximum risk score (matches the underwriter's `FACTOR_MAX`).
pub const RISK_MAX: u64 = 1_000;

/// Hazard multiplier (bps) from a 0..=1000 *safety* score: a prime protocol
/// (score 1000 -> hazard 0) pays 1.0x; a speculative one (score 0) pays 4.0x.
pub fn hazard_multiplier_bps(safety_score: u64) -> u64 {
    let hazard = RISK_MAX.saturating_sub(safety_score.min(RISK_MAX));
    BPS + (hazard * 30_000) / RISK_MAX
}

/// Utilisation multiplier (bps) on a two-slope curve kinked at 80%.
pub fn utilization_multiplier_bps(committed: u64, total: u64) -> u64 {
    if total == 0 {
        return BPS;
    }
    let u = ((committed as u128 * BPS as u128) / total as u128).min(BPS as u128) as u64;
    let optimal = 8_000u64;
    if u <= optimal {
        BPS + (2_000 * u) / optimal
    } else {
        let at_kink = BPS + 2_000;
        let over = u - optimal;
        let span = BPS - optimal;
        at_kink + (30_000 * over) / span
    }
}

/// Effective annual rate (bps) for a product given its base rate, the subject's
/// safety score, and the pool's utilisation after the new policy.
pub fn annual_rate_bps(base_rate_bps: u64, safety_score: u64, committed: u64, total: u64) -> u64 {
    let hazard = hazard_multiplier_bps(safety_score);
    let util = utilization_multiplier_bps(committed, total);
    let r = base_rate_bps as u128 * hazard as u128 / BPS as u128;
    (r * util as u128 / BPS as u128) as u64
}

/// Premium owed (in asset units) for `coverage` over `duration_days`.
pub fn premium(coverage: u64, annual_rate_bps: u64, duration_days: u64) -> Option<u64> {
    let annual = (coverage as u128).checked_mul(annual_rate_bps as u128)? / BPS as u128;
    let p = annual.checked_mul(duration_days as u128)? / DAYS_PER_YEAR as u128;
    u64::try_from(p.max(1)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hazard_scales_inversely_with_safety() {
        assert_eq!(hazard_multiplier_bps(1000), BPS);
        assert!(hazard_multiplier_bps(0) > hazard_multiplier_bps(1000));
        assert_eq!(hazard_multiplier_bps(0), BPS + 30_000);
    }

    #[test]
    fn utilization_curve_is_kinked() {
        let low = utilization_multiplier_bps(4_000, 10_000);
        let kink = utilization_multiplier_bps(8_000, 10_000);
        let high = utilization_multiplier_bps(9_500, 10_000);
        assert!(low < kink && kink < high);
        assert!(high - kink > kink - low);
    }

    #[test]
    fn premium_scales_with_duration() {
        let rate = annual_rate_bps(450, 800, 0, 10_000_000);
        let p30 = premium(1_000_000, rate, 30).unwrap();
        let p360 = premium(1_000_000, rate, 360).unwrap();
        assert!(p360 > p30 * 10);
    }
}

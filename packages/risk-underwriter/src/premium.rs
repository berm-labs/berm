//! Premium pricing engine.
//!
//! The premium for a cover policy is built up multiplicatively:
//!
//! ```text
//! annual_rate = category_base_rate
//!             * hazard_multiplier(risk_score)
//!             * utilisation_multiplier(pool)
//! premium     = coverage * annual_rate * (duration_days / 365)
//! ```
//!
//! All arithmetic is integer basis-point math (no floats) so the result is
//! deterministic and reproducible on chain. This is the parametric-pricing
//! analogue of Nexus Mutual's cost-of-capital model: the price of risk scales
//! with both the intrinsic hazard of the protocol and the scarcity of pool
//! capital.

use serde::{Deserialize, Serialize};

use crate::error::{UnderwriteError, UnderwriteResult};
use crate::factors::FACTOR_MAX;
use crate::model::{utilization_bps, UtilizationCurve, BPS};
use crate::protocol::ProtocolCategory;
use crate::score::RiskScore;

/// Days in the pricing year.
pub const DAYS_PER_YEAR: u64 = 365;

/// A fully itemised premium quote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PremiumQuote {
    /// Coverage amount (USD cents) this quote prices.
    pub coverage: u64,
    /// Duration of cover in days.
    pub duration_days: u64,
    /// Effective annualised rate in basis points.
    pub annual_rate_bps: u64,
    /// Premium owed for the requested duration, in USD cents.
    pub premium: u64,
}

impl PremiumQuote {
    /// Annualised premium (full-year cost) implied by this quote.
    pub fn annualised_premium(&self) -> u64 {
        ((self.coverage as u128 * self.annual_rate_bps as u128) / BPS as u128) as u64
    }
}

/// Stateless premium pricer parameterised by the utilisation curve.
///
/// `Default` uses [`UtilizationCurve::default`] (the calibrated kinked curve).
#[derive(Debug, Clone, Copy, Default)]
pub struct PremiumEngine {
    curve: UtilizationCurve,
}

impl PremiumEngine {
    /// Construct an engine with a custom utilisation curve.
    pub fn new(curve: UtilizationCurve) -> Self {
        Self { curve }
    }

    /// Hazard multiplier (bps) from a risk score. A prime protocol (hazard ~0)
    /// pays ~1.0x; a speculative one (hazard near max) pays up to ~4.0x.
    pub fn hazard_multiplier_bps(&self, score: &RiskScore) -> u64 {
        // 1.0x + (hazard / max) * 3.0x  ->  range [1.0x, 4.0x].
        let extra = (score.hazard() as u64 * 30_000) / FACTOR_MAX as u64;
        BPS + extra
    }

    /// Effective annualised rate (bps) for a protocol+pool state.
    pub fn annual_rate_bps(
        &self,
        category: ProtocolCategory,
        score: &RiskScore,
        committed: u64,
        total_capital: u64,
    ) -> u64 {
        let base = category.base_rate_bps();
        let hazard = self.hazard_multiplier_bps(score);
        let util = self.curve.multiplier_bps(utilization_bps(committed, total_capital));
        // base * hazard/BPS * util/BPS, kept in u128.
        let r = base as u128 * hazard as u128 / BPS as u128;
        (r * util as u128 / BPS as u128) as u64
    }

    /// Produce a full premium quote, validating inputs and pool capacity.
    pub fn quote(
        &self,
        category: ProtocolCategory,
        score: &RiskScore,
        coverage: u64,
        duration_days: u64,
        committed: u64,
        total_capital: u64,
    ) -> UnderwriteResult<PremiumQuote> {
        if coverage == 0 {
            return Err(UnderwriteError::InvalidInput {
                field: "coverage",
                reason: "must be positive".into(),
            });
        }
        if duration_days == 0 {
            return Err(UnderwriteError::InvalidInput {
                field: "duration_days",
                reason: "must be positive".into(),
            });
        }
        let available = total_capital.saturating_sub(committed);
        if coverage > available {
            return Err(UnderwriteError::InsufficientCapacity {
                requested: coverage,
                available,
            });
        }
        // Price at the utilisation that *includes* the new policy, so the buyer
        // pays the marginal scarcity they create.
        let annual_rate_bps =
            self.annual_rate_bps(category, score, committed + coverage, total_capital);
        let annual = (coverage as u128 * annual_rate_bps as u128) / BPS as u128;
        let premium = (annual * duration_days as u128 / DAYS_PER_YEAR as u128) as u64;
        Ok(PremiumQuote {
            coverage,
            duration_days,
            annual_rate_bps,
            premium: premium.max(1), // never quote a zero premium for real cover
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{AuditRecord, ProtocolCategory, ProtocolProfile};

    fn prime_score() -> RiskScore {
        let p = ProtocolProfile {
            name: "X".into(),
            category: ProtocolCategory::Lending,
            tvl_usd_cents: 1_500_000_000 * 100,
            audits: vec![AuditRecord::clean("A"), AuditRecord::clean("B")],
            sloc: 15_000,
            complexity_score: 100,
            days_live: 900,
            past_incidents: 0,
        };
        RiskScore::evaluate_default(&p)
    }

    fn risky_score() -> RiskScore {
        let p = ProtocolProfile {
            name: "Y".into(),
            category: ProtocolCategory::YieldVault,
            tvl_usd_cents: 500_000 * 100,
            audits: vec![],
            sloc: 80_000,
            complexity_score: 380,
            days_live: 20,
            past_incidents: 1,
        };
        RiskScore::evaluate_default(&p)
    }

    #[test]
    fn risky_protocol_pays_more_than_prime() {
        let eng = PremiumEngine::default();
        let prime = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 1_000_000, 30, 0, 10_000_000)
            .unwrap();
        let risky = eng
            .quote(ProtocolCategory::Lending, &risky_score(), 1_000_000, 30, 0, 10_000_000)
            .unwrap();
        assert!(risky.premium > prime.premium);
        assert!(risky.annual_rate_bps > prime.annual_rate_bps);
    }

    #[test]
    fn rejects_coverage_over_capacity() {
        let eng = PremiumEngine::default();
        let err = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 2_000_000, 30, 9_000_000, 10_000_000)
            .unwrap_err();
        assert!(matches!(err, UnderwriteError::InsufficientCapacity { .. }));
    }

    #[test]
    fn longer_duration_costs_proportionally_more() {
        let eng = PremiumEngine::default();
        let m30 = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 1_000_000, 30, 0, 10_000_000)
            .unwrap();
        let m360 = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 1_000_000, 360, 0, 10_000_000)
            .unwrap();
        assert!(m360.premium > m30.premium * 10);
    }

    #[test]
    fn higher_utilization_raises_rate() {
        let eng = PremiumEngine::default();
        let low = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 1_000_000, 30, 0, 10_000_000)
            .unwrap();
        let high = eng
            .quote(ProtocolCategory::Lending, &prime_score(), 1_000_000, 30, 8_500_000, 10_000_000)
            .unwrap();
        assert!(high.annual_rate_bps > low.annual_rate_bps);
    }
}

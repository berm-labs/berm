//! # berm-risk-underwriter
//!
//! Protocol risk scoring and premium underwriting for the Berm cover protocol.
//!
//! Parametric cover settles automatically on an oracle signal, but someone still
//! has to decide *what a policy costs* and *whether the pool can afford to write
//! it*. That is underwriting. This crate turns an observable
//! [`protocol::ProtocolProfile`] into:
//!
//! 1. a transparent, factor-decomposed [`score::RiskScore`]
//!    ([`factors`] -> [`score`]);
//! 2. a deterministic, integer-only [`premium::PremiumQuote`] that scales with
//!    both intrinsic hazard and pool capital scarcity ([`model`] -> [`premium`]);
//! 3. a capital-adequacy check that refuses to over-write the pool
//!    ([`capital`]).
//!
//! The design borrows the audit-grounded risk tiers of Sherlock, the
//! cost-of-capital pricing of Nexus Mutual, and the kinked utilisation curve of
//! on-chain lending markets.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod capital;
pub mod error;
pub mod factors;
pub mod model;
pub mod premium;
pub mod protocol;
pub mod score;

pub use capital::PoolCapital;
pub use error::{UnderwriteError, UnderwriteResult};
pub use model::UtilizationCurve;
pub use premium::{PremiumEngine, PremiumQuote};
pub use protocol::{AuditRecord, ProtocolCategory, ProtocolProfile};
pub use score::{FactorWeights, RiskScore, RiskTier};

/// End-to-end underwriting: score the protocol, check pool solvency, and -- if
/// solvent -- return a premium quote. This is the single call the marketplace
/// backend makes to price a cover request.
pub fn underwrite(
    profile: &ProtocolProfile,
    pool: &PoolCapital,
    coverage: u64,
    duration_days: u64,
) -> UnderwriteResult<(RiskScore, PremiumQuote)> {
    let score = RiskScore::evaluate_default(profile);
    pool.can_underwrite(coverage)?;
    let engine = PremiumEngine::default();
    let quote = engine.quote(
        profile.category,
        &score,
        coverage,
        duration_days,
        pool.locked_capital(),
        pool.total_capital,
    )?;
    Ok((score, quote))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ProtocolProfile {
        ProtocolProfile {
            name: "Kamino".into(),
            category: ProtocolCategory::Lending,
            tvl_usd_cents: 1_200_000_000 * 100,
            audits: vec![AuditRecord::clean("OtterSec"), AuditRecord::clean("Sec3")],
            sloc: 30_000,
            complexity_score: 200,
            days_live: 720,
            past_incidents: 0,
        }
    }

    #[test]
    fn underwrites_a_solvent_pool() {
        let pool = PoolCapital {
            total_capital: 50_000_000,
            outstanding_cover: 10_000_000,
            unearned_premium: 0,
        };
        let (score, quote) = underwrite(&profile(), &pool, 1_000_000, 90).unwrap();
        assert!(score.composite > 600);
        assert!(quote.premium > 0);
        assert_eq!(quote.coverage, 1_000_000);
    }

    #[test]
    fn refuses_when_pool_cannot_back_cover() {
        let pool = PoolCapital {
            total_capital: 1_000_000,
            outstanding_cover: 800_000,
            unearned_premium: 0,
        };
        assert!(underwrite(&profile(), &pool, 5_000_000, 90).is_err());
    }
}

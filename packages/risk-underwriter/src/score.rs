//! Composite risk score.
//!
//! The factors in [`crate::factors`] are blended into a single 0..=1000 safety
//! score using fixed weights. Higher is safer. The score then maps to a
//! multiplicative premium adjustment in [`crate::premium`]. Keeping the blend
//! linear and the weights explicit makes the score reproducible and contestable
//! by governance, mirroring Sherlock's transparent risk-tier model.

use serde::{Deserialize, Serialize};

use crate::factors::{
    audit_factor, complexity_factor, reputation_factor, tvl_factor, FACTOR_MAX,
};
use crate::protocol::ProtocolProfile;

/// Relative weights of each factor (must sum to 100).
pub struct FactorWeights {
    /// Weight of the TVL / Lindy factor.
    pub tvl: u32,
    /// Weight of the audit factor.
    pub audit: u32,
    /// Weight of the code-complexity factor.
    pub complexity: u32,
    /// Weight of the reputation / longevity factor.
    pub reputation: u32,
}

impl Default for FactorWeights {
    fn default() -> Self {
        // Audits and track record dominate; complexity is a smaller modifier.
        Self {
            tvl: 25,
            audit: 35,
            complexity: 15,
            reputation: 25,
        }
    }
}

impl FactorWeights {
    /// Sum of all weights; should be 100.
    pub fn total(&self) -> u32 {
        self.tvl + self.audit + self.complexity + self.reputation
    }
}

/// A decomposed risk score with each factor preserved for transparency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskScore {
    /// TVL sub-score.
    pub tvl: u32,
    /// Audit sub-score.
    pub audit: u32,
    /// Complexity sub-score.
    pub complexity: u32,
    /// Reputation sub-score.
    pub reputation: u32,
    /// Weighted composite safety score (0..=1000, higher safer).
    pub composite: u32,
}

/// Qualitative risk tiers derived from the composite score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier {
    /// Composite >= 800.
    Prime,
    /// Composite 600..800.
    Standard,
    /// Composite 400..600.
    Elevated,
    /// Composite < 400.
    Speculative,
}

impl RiskScore {
    /// Compute a risk score for `p` using `weights`.
    pub fn evaluate(p: &ProtocolProfile, weights: &FactorWeights) -> RiskScore {
        let tvl = tvl_factor(p);
        let audit = audit_factor(p);
        let complexity = complexity_factor(p);
        let reputation = reputation_factor(p);
        let total = weights.total().max(1) as u64;
        let composite = ((tvl as u64 * weights.tvl as u64
            + audit as u64 * weights.audit as u64
            + complexity as u64 * weights.complexity as u64
            + reputation as u64 * weights.reputation as u64)
            / total) as u32;
        RiskScore {
            tvl,
            audit,
            complexity,
            reputation,
            composite: composite.min(FACTOR_MAX),
        }
    }

    /// Convenience scorer using the default weights.
    pub fn evaluate_default(p: &ProtocolProfile) -> RiskScore {
        RiskScore::evaluate(p, &FactorWeights::default())
    }

    /// Map the composite onto a qualitative tier.
    pub fn tier(&self) -> RiskTier {
        match self.composite {
            c if c >= 800 => RiskTier::Prime,
            c if c >= 600 => RiskTier::Standard,
            c if c >= 400 => RiskTier::Elevated,
            _ => RiskTier::Speculative,
        }
    }

    /// Hazard score = the complement of safety (higher means riskier), used as
    /// the premium multiplier input.
    pub fn hazard(&self) -> u32 {
        FACTOR_MAX - self.composite
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{AuditRecord, ProtocolCategory};

    fn prime() -> ProtocolProfile {
        ProtocolProfile {
            name: "BlueChip".into(),
            category: ProtocolCategory::Lending,
            tvl_usd_cents: 1_500_000_000 * 100,
            audits: vec![
                AuditRecord::clean("OtterSec"),
                AuditRecord::clean("Sec3"),
                AuditRecord::clean("Trail of Bits"),
            ],
            sloc: 18_000,
            complexity_score: 120,
            days_live: 900,
            past_incidents: 0,
        }
    }

    fn risky() -> ProtocolProfile {
        ProtocolProfile {
            name: "FreshFork".into(),
            category: ProtocolCategory::YieldVault,
            tvl_usd_cents: 800_000 * 100,
            audits: vec![],
            sloc: 90_000,
            complexity_score: 380,
            days_live: 30,
            past_incidents: 1,
        }
    }

    #[test]
    fn weights_sum_to_100() {
        assert_eq!(FactorWeights::default().total(), 100);
    }

    #[test]
    fn prime_outscores_risky() {
        let a = RiskScore::evaluate_default(&prime());
        let b = RiskScore::evaluate_default(&risky());
        assert!(a.composite > b.composite);
        assert_eq!(a.tier(), RiskTier::Prime);
        assert_eq!(b.tier(), RiskTier::Speculative);
        assert!(b.hazard() > a.hazard());
    }

    #[test]
    fn composite_stays_in_range() {
        let s = RiskScore::evaluate_default(&prime());
        assert!(s.composite <= FACTOR_MAX);
    }
}

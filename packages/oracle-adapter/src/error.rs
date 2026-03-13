//! Error taxonomy for the oracle adapter layer.
//!
//! Every fallible path in the adapter returns a typed [`OracleError`] so that the
//! off-chain keeper (`berm-cover-engine`) can branch on the precise failure mode
//! instead of inspecting opaque strings. The variants mirror the failure classes
//! documented in the Pyth Lazer and Switchboard On-Demand integration guides:
//! staleness, low confidence, and cross-source divergence.

use thiserror::Error;

/// Result alias used throughout the crate.
pub type OracleResult<T> = Result<T, OracleError>;

/// Typed failures surfaced by the oracle aggregation pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OracleError {
    /// A feed's most recent update is older than the configured staleness budget.
    #[error("feed `{feed}` is stale: {age_slots} slots old (max {max_slots})")]
    Stale {
        /// Human readable feed identifier.
        feed: String,
        /// Age of the latest update in slots.
        age_slots: u64,
        /// Maximum tolerated age in slots.
        max_slots: u64,
    },

    /// The reported confidence interval is too wide relative to the price.
    #[error("feed `{feed}` confidence too wide: {ratio_bps} bps (max {max_bps} bps)")]
    LowConfidence {
        /// Human readable feed identifier.
        feed: String,
        /// Confidence-to-price ratio expressed in basis points.
        ratio_bps: u64,
        /// Maximum tolerated ratio in basis points.
        max_bps: u64,
    },

    /// No healthy source was available to produce an aggregate price.
    #[error("no healthy sources for feed `{feed}` ({healthy}/{total} usable)")]
    NoHealthySource {
        /// Human readable feed identifier.
        feed: String,
        /// Count of sources that passed health checks.
        healthy: usize,
        /// Total configured sources.
        total: usize,
    },

    /// Two or more sources disagree beyond the divergence threshold.
    #[error("source divergence on `{feed}`: {observed_bps} bps (threshold {threshold_bps} bps)")]
    Divergence {
        /// Human readable feed identifier.
        feed: String,
        /// Observed maximum pairwise divergence in basis points.
        observed_bps: u64,
        /// Configured divergence threshold in basis points.
        threshold_bps: u64,
    },

    /// A raw account buffer could not be decoded into the expected layout.
    #[error("malformed account for `{feed}`: {reason}")]
    Decode {
        /// Human readable feed identifier.
        feed: String,
        /// Reason the decode failed.
        reason: String,
    },

    /// Arithmetic over- or under-flowed while normalising exponents.
    #[error("arithmetic overflow while normalising `{feed}`")]
    Overflow {
        /// Human readable feed identifier.
        feed: String,
    },
}

impl OracleError {
    /// Returns `true` when the failure is transient and the keeper should retry
    /// on the next slot rather than escalating to an `OracleCover` divergence.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            OracleError::Stale { .. } | OracleError::NoHealthySource { .. }
        )
    }

    /// Returns `true` when the failure itself constitutes an oracle-failure
    /// signal eligible to arm an `OracleCover` policy.
    pub fn is_divergence(&self) -> bool {
        matches!(self, OracleError::Divergence { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_transient_vs_divergence() {
        let stale = OracleError::Stale {
            feed: "USDC/USD".into(),
            age_slots: 200,
            max_slots: 50,
        };
        assert!(stale.is_transient());
        assert!(!stale.is_divergence());

        let div = OracleError::Divergence {
            feed: "USDC/USD".into(),
            observed_bps: 600,
            threshold_bps: 200,
        };
        assert!(div.is_divergence());
        assert!(!div.is_transient());
    }

    #[test]
    fn renders_human_readable_messages() {
        let e = OracleError::LowConfidence {
            feed: "SOL/USD".into(),
            ratio_bps: 90,
            max_bps: 50,
        };
        assert!(e.to_string().contains("SOL/USD"));
        assert!(e.to_string().contains("90"));
    }
}

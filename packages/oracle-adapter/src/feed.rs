//! Feed configuration and the source registry.
//!
//! A "feed" is a logical asset price (e.g. `USDC/USD`) backed by one or more
//! physical oracle accounts on different networks. The dual/triple-oracle pattern
//! recommended by both Pyth and Switchboard requires that we track which source
//! a given observation came from so divergence can be attributed.

use serde::{Deserialize, Serialize};

use crate::price::NormalizedPrice;
use crate::staleness::StalenessPolicy;

/// The provenance of an observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceKind {
    /// Pyth Network price account (pull / Lazer).
    Pyth,
    /// Switchboard On-Demand pull feed.
    Switchboard,
    /// Chainlink OCR2 aggregator report.
    Chainlink,
}

impl SourceKind {
    /// Stable lowercase label used in logs and metrics.
    pub fn label(&self) -> &'static str {
        match self {
            SourceKind::Pyth => "pyth",
            SourceKind::Switchboard => "switchboard",
            SourceKind::Chainlink => "chainlink",
        }
    }

    /// Relative trust weight applied during weighted aggregation. Pyth and
    /// Switchboard are first-party Solana oracles; Chainlink reaches Solana over
    /// CCIP and is weighted slightly lower to reflect bridge latency.
    pub fn default_weight(&self) -> u32 {
        match self {
            SourceKind::Pyth => 40,
            SourceKind::Switchboard => 40,
            SourceKind::Chainlink => 20,
        }
    }
}

/// A single source observation tagged with its provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    /// Which oracle produced this price.
    pub source: SourceKind,
    /// The normalised price.
    pub price: NormalizedPrice,
    /// Trust weight (defaults from [`SourceKind::default_weight`]).
    pub weight: u32,
}

impl Observation {
    /// Build an observation with the source's default weight.
    pub fn new(source: SourceKind, price: NormalizedPrice) -> Self {
        Self {
            source,
            price,
            weight: source.default_weight(),
        }
    }

    /// Override the trust weight.
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
}

/// Static configuration for a logical feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedConfig {
    /// Human readable symbol, e.g. `USDC/USD`.
    pub symbol: String,
    /// Common exponent all sources are normalised to before comparison.
    pub target_expo: i32,
    /// Freshness / confidence budget.
    pub staleness: StalenessPolicy,
    /// Cross-source divergence threshold in basis points. Exceeding this both
    /// invalidates the aggregate and arms an `OracleCover` signal.
    pub divergence_threshold_bps: u64,
    /// Minimum number of healthy sources required to publish an aggregate.
    pub min_sources: usize,
}

impl FeedConfig {
    /// A sane default profile for a USD stablecoin feed.
    pub fn stablecoin(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            target_expo: -8,
            staleness: StalenessPolicy::new(50, 30),
            divergence_threshold_bps: 200, // 2%
            min_sources: 2,
        }
    }

    /// A default profile for a volatile asset feed (wider confidence band).
    pub fn volatile(symbol: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            target_expo: -8,
            staleness: StalenessPolicy::new(50, 80),
            divergence_threshold_bps: 300, // 3%
            min_sources: 2,
        }
    }

    /// Override the minimum-sources requirement.
    pub fn require_sources(mut self, n: usize) -> Self {
        self.min_sources = n;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_weights_sum_to_100() {
        let total = SourceKind::Pyth.default_weight()
            + SourceKind::Switchboard.default_weight()
            + SourceKind::Chainlink.default_weight();
        assert_eq!(total, 100);
    }

    #[test]
    fn stablecoin_profile_is_tighter_than_volatile() {
        let s = FeedConfig::stablecoin("USDC/USD");
        let v = FeedConfig::volatile("SOL/USD");
        assert!(s.staleness.max_confidence_bps < v.staleness.max_confidence_bps);
        assert!(s.divergence_threshold_bps < v.divergence_threshold_bps);
    }

    #[test]
    fn observation_defaults_weight_from_source() {
        let o = Observation::new(SourceKind::Pyth, NormalizedPrice::new(1, 0, 0, 0));
        assert_eq!(o.weight, 40);
        assert_eq!(o.with_weight(10).weight, 10);
    }
}

//! # berm-oracle-adapter
//!
//! Multi-source oracle aggregation for the Berm cover protocol.
//!
//! Parametric cover replaces slow, discretionary claim adjudication with a price
//! oracle that arms and settles policies automatically. That only works if the
//! price the protocol settles on is trustworthy, so this crate implements a
//! defence-in-depth oracle layer following the dual-oracle pattern recommended by
//! both Pyth Network (Lazer) and Switchboard, extended with a Chainlink OCR2 /
//! CCIP fallback:
//!
//! - [`pyth`] converts Pyth price feeds (and the on-chain v2 price account) into
//!   a normalised representation.
//! - [`switchboard`] decodes Switchboard On-Demand pull-feed results.
//! - [`chainlink`] decodes Chainlink OCR2 rounds delivered over CCIP.
//! - [`staleness`] enforces freshness and confidence budgets per feed.
//! - [`aggregator`] reconciles healthy sources into a single weighted-median
//!   price and raises an [`error::OracleError::Divergence`] when sources
//!   disagree beyond tolerance -- the parametric signal behind `OracleCover`.
//!
//! All settlement math runs on integer mantissas; `f64` is exposed only for
//! human-readable reporting.
//!
//! ## References
//! - Pyth Network price-feed semantics (signed exponent + confidence interval).
//! - Switchboard On-Demand pull-feed result format (18-decimal fixed point).
//! - Chainlink OCR2 report / CCIP `Any2SolanaMessage` round shape.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod aggregator;
pub mod chainlink;
pub mod error;
pub mod feed;
pub mod price;
pub mod pyth;
pub mod staleness;
pub mod switchboard;

pub use aggregator::{aggregate, detect_divergence, Aggregate};
pub use error::{OracleError, OracleResult};
pub use feed::{FeedConfig, Observation, SourceKind};
pub use price::{divergence_bps, NormalizedPrice};
pub use staleness::StalenessPolicy;

/// Convenience: aggregate a feed from raw per-source observations, returning
/// either the reconciled price or the typed failure (stale / divergent / etc).
///
/// This is the single entry point the cover engine calls each slot.
pub fn reconcile(
    cfg: &FeedConfig,
    observations: &[Observation],
    current_slot: u64,
) -> OracleResult<Aggregate> {
    aggregate(cfg, observations, current_slot)
}

/// Build the default Berm feed registry: the stablecoins and LSTs the protocol
/// underwrites cover for out of the box.
pub fn default_feeds() -> Vec<FeedConfig> {
    vec![
        FeedConfig::stablecoin("USDC/USD"),
        FeedConfig::stablecoin("USDT/USD"),
        FeedConfig::stablecoin("USDS/USD"),
        FeedConfig::volatile("SOL/USD"),
        FeedConfig::volatile("mSOL/USD"),
        FeedConfig::volatile("jitoSOL/USD"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_covers_core_assets() {
        let feeds = default_feeds();
        assert!(feeds.iter().any(|f| f.symbol == "USDC/USD"));
        assert!(feeds.iter().any(|f| f.symbol == "mSOL/USD"));
        assert_eq!(feeds.len(), 6);
    }

    #[test]
    fn reconcile_end_to_end() {
        let cfg = FeedConfig::stablecoin("USDC/USD");
        let o = vec![
            Observation::new(SourceKind::Pyth, NormalizedPrice::new(100_000_000, 1000, -8, 10)),
            Observation::new(
                SourceKind::Switchboard,
                NormalizedPrice::new(100_010_000, 1000, -8, 10),
            ),
        ];
        let agg = reconcile(&cfg, &o, 10).unwrap();
        assert_eq!(agg.healthy_sources, 2);
    }
}

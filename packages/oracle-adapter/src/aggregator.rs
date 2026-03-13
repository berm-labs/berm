//! Cross-source aggregation and divergence detection.
//!
//! Given a set of [`Observation`]s for one logical feed, the aggregator:
//!
//! 1. discards observations that fail the feed's staleness / confidence gates;
//! 2. requires at least `min_sources` survivors;
//! 3. normalises survivors to a common exponent;
//! 4. computes the maximum pairwise divergence and, if it exceeds the feed
//!    threshold, surfaces an [`OracleError::Divergence`] (the `OracleCover`
//!    signal);
//! 5. otherwise returns a weighted-median aggregate price.
//!
//! The dual/triple-oracle divergence check is the parametric trigger behind
//! `OracleCover`: when Pyth and Switchboard (and optionally Chainlink) disagree
//! beyond tolerance, dependent lending markets may mis-price collateral, so the
//! cover arms.

use serde::{Deserialize, Serialize};

use crate::error::{OracleError, OracleResult};
use crate::feed::{FeedConfig, Observation};
use crate::price::{divergence_bps, NormalizedPrice};

/// The outcome of aggregating a feed at a given slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aggregate {
    /// The aggregated price at the feed's target exponent.
    pub price: NormalizedPrice,
    /// Number of sources that survived the health gates.
    pub healthy_sources: usize,
    /// Maximum observed pairwise divergence in basis points.
    pub max_divergence_bps: u64,
}

/// Aggregate observations for `cfg` evaluated at `current_slot`.
pub fn aggregate(
    cfg: &FeedConfig,
    observations: &[Observation],
    current_slot: u64,
) -> OracleResult<Aggregate> {
    let total = observations.len();

    // 1+2: filter unhealthy and rescale survivors to the common exponent.
    let mut healthy: Vec<(NormalizedPrice, u32)> = Vec::new();
    for obs in observations {
        if cfg
            .staleness
            .validate(&cfg.symbol, &obs.price, current_slot)
            .is_err()
        {
            continue;
        }
        let scaled = obs.price.rescale(cfg.target_expo, &cfg.symbol)?;
        healthy.push((scaled, obs.weight));
    }

    if healthy.len() < cfg.min_sources {
        return Err(OracleError::NoHealthySource {
            feed: cfg.symbol.clone(),
            healthy: healthy.len(),
            total,
        });
    }

    // 4: maximum pairwise divergence across healthy sources.
    let max_div = max_pairwise_divergence(&healthy.iter().map(|(p, _)| p.price).collect::<Vec<_>>());
    if max_div > cfg.divergence_threshold_bps {
        return Err(OracleError::Divergence {
            feed: cfg.symbol.clone(),
            observed_bps: max_div,
            threshold_bps: cfg.divergence_threshold_bps,
        });
    }

    // 5: weighted median aggregate.
    let agg_price = weighted_median(&mut healthy);
    let agg_conf = healthy.iter().map(|(p, _)| p.conf).max().unwrap_or(0);
    Ok(Aggregate {
        price: NormalizedPrice::new(agg_price, agg_conf, cfg.target_expo, current_slot),
        healthy_sources: healthy.len(),
        max_divergence_bps: max_div,
    })
}

/// Maximum pairwise divergence (bps) over a slice of same-scale prices.
pub fn max_pairwise_divergence(prices: &[i64]) -> u64 {
    let mut max = 0;
    for i in 0..prices.len() {
        for j in (i + 1)..prices.len() {
            max = max.max(divergence_bps(prices[i], prices[j]));
        }
    }
    max
}

/// Weighted median over `(price, weight)` pairs.
///
/// Sorts by price, accumulates weight, and returns the price at which the
/// cumulative weight first crosses half of the total. The weighted median is
/// robust to a single corrupted source in a way the mean is not.
pub fn weighted_median(samples: &mut [(NormalizedPrice, u32)]) -> i64 {
    samples.sort_by_key(|(p, _)| p.price);
    let total: u64 = samples.iter().map(|(_, w)| *w as u64).sum();
    let half = total.div_ceil(2);
    let mut acc = 0u64;
    for (p, w) in samples.iter() {
        acc += *w as u64;
        if acc >= half {
            return p.price;
        }
    }
    samples.last().map(|(p, _)| p.price).unwrap_or(0)
}

/// Pure divergence check, exposed for the `OracleCover` trigger which only needs
/// the boolean signal and the observed magnitude (not a full aggregate).
pub fn detect_divergence(cfg: &FeedConfig, observations: &[Observation]) -> Option<u64> {
    let prices: Vec<i64> = observations
        .iter()
        .filter_map(|o| o.price.rescale(cfg.target_expo, &cfg.symbol).ok())
        .map(|p| p.price)
        .collect();
    let max = max_pairwise_divergence(&prices);
    (max > cfg.divergence_threshold_bps).then_some(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feed::{Observation, SourceKind};

    fn obs(source: SourceKind, price: i64, slot: u64) -> Observation {
        Observation::new(source, NormalizedPrice::new(price, 1000, -8, slot))
    }

    #[test]
    fn aggregates_agreeing_sources() {
        let cfg = FeedConfig::stablecoin("USDC/USD");
        let o = vec![
            obs(SourceKind::Pyth, 100_000_000, 100),
            obs(SourceKind::Switchboard, 100_050_000, 100),
            obs(SourceKind::Chainlink, 99_980_000, 100),
        ];
        let agg = aggregate(&cfg, &o, 100).unwrap();
        assert_eq!(agg.healthy_sources, 3);
        assert!(agg.max_divergence_bps < cfg.divergence_threshold_bps);
        assert_eq!(agg.price.price, 100_000_000); // weighted median
    }

    #[test]
    fn flags_divergence_as_oracle_cover_signal() {
        let cfg = FeedConfig::stablecoin("USDC/USD");
        let o = vec![
            obs(SourceKind::Pyth, 100_000_000, 100),
            obs(SourceKind::Switchboard, 90_000_000, 100), // 10% off
        ];
        let err = aggregate(&cfg, &o, 100).unwrap_err();
        assert!(err.is_divergence());
        assert!(detect_divergence(&cfg, &o).unwrap() >= 1000);
    }

    #[test]
    fn requires_minimum_sources_after_staleness() {
        let cfg = FeedConfig::stablecoin("USDC/USD");
        let o = vec![
            obs(SourceKind::Pyth, 100_000_000, 0), // stale
            obs(SourceKind::Switchboard, 100_000_000, 100),
        ];
        // Only one healthy at slot 100 (the other is 100 slots old > 50).
        let err = aggregate(&cfg, &o, 100).unwrap_err();
        assert!(matches!(err, OracleError::NoHealthySource { .. }));
    }
}

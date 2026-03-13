//! Chainlink adapter (OCR2 report format, delivered to Solana over CCIP).
//!
//! Chainlink price data reaches Solana through the Chainlink Data Store program,
//! whose rounds carry an OCR2 report: an `answer` integer scaled by a feed-level
//! `decimals`, the `observations_timestamp`, and the count of observers that
//! signed the round. When the feed is bridged cross-chain via CCIP the same
//! report is wrapped in a CCIP `Any2SolanaMessage`; we model the decoded report
//! plus the bridge freshness metadata and normalise into [`NormalizedPrice`].
//!
//! There is no maintained first-party Chainlink Solana crate, so the report
//! decoding here is an in-house implementation of the public OCR2 round shape.

use serde::{Deserialize, Serialize};

use crate::error::{OracleError, OracleResult};
use crate::feed::{Observation, SourceKind};
use crate::price::NormalizedPrice;

/// A decoded Chainlink OCR2 round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ocr2Round {
    /// Monotonic round identifier.
    pub round_id: u64,
    /// The reported answer, scaled by `decimals`.
    pub answer: i128,
    /// Number of decimal places applied to `answer`.
    pub decimals: u32,
    /// Number of distinct oracle signatures aggregated into the round.
    pub observer_count: u8,
    /// Slot at which the round (or its CCIP delivery) landed on Solana.
    pub landed_slot: u64,
}

/// Minimum signer set for a round to be considered trustworthy. OCR2 feeds use
/// an f+1 honest-majority assumption; we require a conservative quorum.
pub const MIN_OBSERVERS: u8 = 3;

impl Ocr2Round {
    /// Normalise an OCR2 round into the common price representation.
    pub fn normalize(&self, feed: &str) -> OracleResult<NormalizedPrice> {
        if self.observer_count < MIN_OBSERVERS {
            return Err(OracleError::Decode {
                feed: feed.into(),
                reason: format!(
                    "insufficient observers: {} < {}",
                    self.observer_count, MIN_OBSERVERS
                ),
            });
        }
        if self.answer <= 0 {
            return Err(OracleError::Decode {
                feed: feed.into(),
                reason: "non-positive chainlink answer".into(),
            });
        }
        let price = i64::try_from(self.answer).map_err(|_| OracleError::Decode {
            feed: feed.into(),
            reason: "chainlink answer exceeds i64".into(),
        })?;
        // Chainlink does not publish a confidence band; derive a conservative one
        // from the inverse of the observer count (more signers -> tighter band).
        let conf = derive_confidence(price, self.observer_count);
        Ok(NormalizedPrice::new(
            price,
            conf,
            -(self.decimals as i32),
            self.landed_slot,
        ))
    }

    /// Build a tagged [`Observation`].
    pub fn observe(&self, feed: &str) -> OracleResult<Observation> {
        Ok(Observation::new(SourceKind::Chainlink, self.normalize(feed)?))
    }
}

/// Derive a synthetic one-sigma confidence for a Chainlink round.
///
/// With no native confidence interval we map signer count onto a band: a fully
/// subscribed round (>= 16 signers) gets ~5 bps, scaling up toward ~40 bps at
/// the minimum quorum. This keeps Chainlink comparable to confidence-bearing
/// feeds during aggregation without overstating its precision.
pub fn derive_confidence(price: i64, observer_count: u8) -> u64 {
    let bps = match observer_count {
        n if n >= 16 => 5,
        n if n >= 8 => 12,
        n if n >= 5 => 25,
        _ => 40,
    };
    ((price.unsigned_abs() as u128 * bps) / 10_000) as u64
}

/// A CCIP cross-chain delivery wrapping an OCR2 round, carrying the extra hop's
/// freshness so the keeper can account for bridge latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CcipDelivery {
    /// The underlying OCR2 round.
    pub round: Ocr2Round,
    /// Source chain selector the report originated from.
    pub source_chain_selector: u64,
    /// Sequence number of the CCIP message (monotonic per lane).
    pub sequence_number: u64,
}

impl CcipDelivery {
    /// Normalise the wrapped round.
    pub fn normalize(&self, feed: &str) -> OracleResult<NormalizedPrice> {
        self.round.normalize(feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_healthy_round() {
        let r = Ocr2Round {
            round_id: 42,
            answer: 100_002_000,
            decimals: 8,
            observer_count: 16,
            landed_slot: 555,
        };
        let n = r.normalize("USDC/USD").unwrap();
        assert_eq!(n.price, 100_002_000);
        assert_eq!(n.expo, -8);
        assert!(n.conf > 0);
        assert_eq!(n.publish_slot, 555);
    }

    #[test]
    fn rejects_under_quorum_round() {
        let r = Ocr2Round {
            round_id: 1,
            answer: 100_000_000,
            decimals: 8,
            observer_count: 2,
            landed_slot: 1,
        };
        assert!(r.normalize("X").is_err());
    }

    #[test]
    fn confidence_tightens_with_more_observers() {
        let high = derive_confidence(100_000_000, 16);
        let low = derive_confidence(100_000_000, 5);
        assert!(high < low);
    }
}

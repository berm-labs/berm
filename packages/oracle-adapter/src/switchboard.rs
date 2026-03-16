//! Switchboard On-Demand adapter.
//!
//! Switchboard On-Demand "pull" feeds expose the latest result as a 18-decimal
//! signed fixed-point value (`SwitchboardDecimal`: a mantissa plus a `scale`
//! count of decimal places) together with the slot at which the oracle landed
//! the update and the spread between the responding oracles. This module mirrors
//! that representation off-chain and normalises it into [`NormalizedPrice`].
//!
//! The on-chain programs link the real `switchboard-on-demand` crate and read
//! `PullFeedAccountData`; off-chain the keeper receives the already-decoded value
//! over RPC, so we model the decoded shape here directly.

use serde::{Deserialize, Serialize};

use crate::error::{OracleError, OracleResult};
use crate::feed::{Observation, SourceKind};
use crate::price::NormalizedPrice;

/// Switchboard's 18-decimal fixed-point value as delivered by a pull feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchboardDecimal {
    /// Signed mantissa.
    pub mantissa: i128,
    /// Number of decimal places (the value is `mantissa / 10^scale`).
    pub scale: u32,
}

impl SwitchboardDecimal {
    /// Construct a decimal.
    pub fn new(mantissa: i128, scale: u32) -> Self {
        Self { mantissa, scale }
    }
}

/// The decoded latest-result view of a Switchboard On-Demand pull feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullFeedResult {
    /// Aggregated median value across the responding oracles.
    pub value: SwitchboardDecimal,
    /// Range (max - min) across responding oracles, used as a confidence proxy.
    pub range: SwitchboardDecimal,
    /// Slot at which the result landed on chain.
    pub result_slot: u64,
    /// Number of oracle samples in the result.
    pub sample_count: u32,
}

impl PullFeedResult {
    /// Normalise the result into the protocol's common representation.
    ///
    /// The Switchboard scale becomes the negative exponent. The oracle range is
    /// halved and used as the confidence interval (range as a full-width band,
    /// half-width sigma), matching how Switchboard documents result spread.
    pub fn normalize(&self, feed: &str) -> OracleResult<NormalizedPrice> {
        if self.sample_count == 0 {
            return Err(OracleError::Decode {
                feed: feed.into(),
                reason: "switchboard result has zero samples".into(),
            });
        }
        let expo = -(self.value.scale as i32);
        let price = i64::try_from(self.value.mantissa).map_err(|_| OracleError::Decode {
            feed: feed.into(),
            reason: "switchboard mantissa exceeds i64".into(),
        })?;
        // Align the range to the value's scale, then take half as one-sigma conf.
        let range_aligned = align_scale(self.range, self.value.scale);
        let conf = u64::try_from((range_aligned.max(0)) / 2).unwrap_or(u64::MAX);
        Ok(NormalizedPrice::new(price, conf, expo, self.result_slot))
    }

    /// Build a tagged [`Observation`].
    pub fn observe(&self, feed: &str) -> OracleResult<Observation> {
        Ok(Observation::new(SourceKind::Switchboard, self.normalize(feed)?))
    }
}

/// Re-express a [`SwitchboardDecimal`] mantissa at a target scale.
pub fn align_scale(dec: SwitchboardDecimal, target_scale: u32) -> i128 {
    if dec.scale == target_scale {
        dec.mantissa
    } else if dec.scale < target_scale {
        dec.mantissa * 10i128.pow(target_scale - dec.scale)
    } else {
        dec.mantissa / 10i128.pow(dec.scale - target_scale)
    }
}

/// Convenience constructor for a healthy single-value result.
pub fn result_from_value(mantissa: i128, scale: u32, slot: u64) -> PullFeedResult {
    PullFeedResult {
        value: SwitchboardDecimal::new(mantissa, scale),
        range: SwitchboardDecimal::new(0, scale),
        result_slot: slot,
        sample_count: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_pull_feed_result() {
        let r = PullFeedResult {
            value: SwitchboardDecimal::new(100_050_000, 8), // 1.0005 at 8 dp
            range: SwitchboardDecimal::new(20_000, 8),
            result_slot: 900,
            sample_count: 5,
        };
        let n = r.normalize("USDC/USD").unwrap();
        assert_eq!(n.price, 100_050_000);
        assert_eq!(n.expo, -8);
        assert_eq!(n.conf, 10_000); // half of range
        assert_eq!(n.publish_slot, 900);
    }

    #[test]
    fn rejects_mantissa_exceeding_i64() {
        // A raw 18-decimal mantissa larger than i64::MAX must be rejected.
        let r = PullFeedResult {
            value: SwitchboardDecimal::new(i128::from(u64::MAX) * 4, 18),
            range: SwitchboardDecimal::new(0, 18),
            result_slot: 1,
            sample_count: 3,
        };
        assert!(r.normalize("X").is_err());
    }

    #[test]
    fn rejects_zero_sample_result() {
        let r = PullFeedResult {
            value: SwitchboardDecimal::new(1, 8),
            range: SwitchboardDecimal::new(0, 8),
            result_slot: 1,
            sample_count: 0,
        };
        assert!(r.normalize("X").is_err());
    }

    #[test]
    fn aligns_scale_up_and_down() {
        assert_eq!(align_scale(SwitchboardDecimal::new(5, 2), 4), 500);
        assert_eq!(align_scale(SwitchboardDecimal::new(500, 4), 2), 5);
    }
}

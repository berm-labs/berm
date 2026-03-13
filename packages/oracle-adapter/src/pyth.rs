//! Pyth Network adapter.
//!
//! Off-chain the keeper consumes Pyth price updates as [`pyth_sdk::Price`] values
//! (the same struct Pyth's Hermes / Lazer endpoints serialise). This module
//! converts a `pyth_sdk::Price` into the protocol-internal [`NormalizedPrice`]
//! and applies the publish-time -> slot mapping the keeper tracks.
//!
//! Reference: Pyth Network price-feed semantics, where `price` carries a signed
//! exponent `expo` and `conf` is the one-sigma confidence interval.

use pyth_sdk::Price as PythPrice;

use crate::error::{OracleError, OracleResult};
use crate::feed::{Observation, SourceKind};
use crate::price::NormalizedPrice;

/// Convert a `pyth_sdk::Price` into a [`NormalizedPrice`].
///
/// `publish_slot` is supplied by the keeper because Pyth reports a unix publish
/// time, not a Solana slot; the keeper maintains the slot<->time mapping from the
/// RPC it polls.
pub fn from_pyth_price(p: &PythPrice, publish_slot: u64, feed: &str) -> OracleResult<NormalizedPrice> {
    if p.price == 0 && p.conf == 0 {
        return Err(OracleError::Decode {
            feed: feed.into(),
            reason: "pyth price and confidence are both zero".into(),
        });
    }
    // `pyth_sdk::Price::conf` is already a `u64`; carry it through directly.
    Ok(NormalizedPrice::new(p.price, p.conf, p.expo, publish_slot))
}

/// Build a tagged [`Observation`] from a Pyth price.
pub fn observe(p: &PythPrice, publish_slot: u64, feed: &str) -> OracleResult<Observation> {
    let price = from_pyth_price(p, publish_slot, feed)?;
    Ok(Observation::new(SourceKind::Pyth, price))
}

/// On-chain Pyth price-account (v2) field offsets.
///
/// The Anchor programs in this workspace cannot link `pyth-sdk-solana` (it
/// conflicts with `switchboard-on-demand` on `borsh`), so the canonical v2 price
/// account layout offsets are mirrored here and exercised by
/// [`parse_price_account`]. These offsets match the public Pyth `price_account`
/// C layout: magic(4) ver(4) atype(4) size(4) ptype(4) expo(4) ... then the
/// aggregate price/conf live in the `agg` sub-struct.
pub mod account_layout {
    /// Magic number prefixing every Pyth account.
    pub const MAGIC: u32 = 0xa1b2_c3d4;
    /// Offset of the 4-byte magic.
    pub const MAGIC_OFFSET: usize = 0;
    /// Offset of the signed `expo` field.
    pub const EXPO_OFFSET: usize = 20;
    /// Offset of the aggregate price (`i64`).
    pub const AGG_PRICE_OFFSET: usize = 208;
    /// Offset of the aggregate confidence (`u64`).
    pub const AGG_CONF_OFFSET: usize = 216;
    /// Offset of the last-published slot (`u64`).
    pub const VALID_SLOT_OFFSET: usize = 40;
}

/// Decode a Pyth v2 price account buffer into a [`NormalizedPrice`].
///
/// This is the routine the on-chain programs use (the same offsets are copied
/// into the Anchor crates). It validates the magic, then reads the aggregate
/// price, confidence, exponent, and the valid slot.
pub fn parse_price_account(data: &[u8], feed: &str) -> OracleResult<NormalizedPrice> {
    use account_layout::*;
    let need = AGG_CONF_OFFSET + 8;
    if data.len() < need {
        return Err(OracleError::Decode {
            feed: feed.into(),
            reason: format!("account too small: {} < {}", data.len(), need),
        });
    }
    let magic = u32::from_le_bytes(read4(data, MAGIC_OFFSET));
    if magic != MAGIC {
        return Err(OracleError::Decode {
            feed: feed.into(),
            reason: format!("bad magic {magic:#x}"),
        });
    }
    let expo = i32::from_le_bytes(read4(data, EXPO_OFFSET));
    let price = i64::from_le_bytes(read8(data, AGG_PRICE_OFFSET));
    let conf = u64::from_le_bytes(read8(data, AGG_CONF_OFFSET));
    let slot = u64::from_le_bytes(read8(data, VALID_SLOT_OFFSET));
    Ok(NormalizedPrice::new(price, conf, expo, slot))
}

fn read4(data: &[u8], off: usize) -> [u8; 4] {
    let mut b = [0u8; 4];
    b.copy_from_slice(&data[off..off + 4]);
    b
}

fn read8(data: &[u8], off: usize) -> [u8; 8] {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[off..off + 8]);
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_pyth_price() {
        let p = PythPrice {
            price: 100_000_000,
            conf: 50_000,
            expo: -8,
            publish_time: 1_700_000_000,
        };
        let n = from_pyth_price(&p, 250, "USDC/USD").unwrap();
        assert_eq!(n.price, 100_000_000);
        assert_eq!(n.expo, -8);
        assert_eq!(n.publish_slot, 250);
    }

    #[test]
    fn rejects_empty_pyth_price() {
        let p = PythPrice {
            price: 0,
            conf: 0,
            expo: -8,
            publish_time: 0,
        };
        assert!(from_pyth_price(&p, 1, "X").is_err());
    }

    #[test]
    fn parses_synthetic_price_account() {
        use account_layout::*;
        let mut buf = vec![0u8; AGG_CONF_OFFSET + 8];
        buf[MAGIC_OFFSET..MAGIC_OFFSET + 4].copy_from_slice(&MAGIC.to_le_bytes());
        buf[EXPO_OFFSET..EXPO_OFFSET + 4].copy_from_slice(&(-8i32).to_le_bytes());
        buf[AGG_PRICE_OFFSET..AGG_PRICE_OFFSET + 8].copy_from_slice(&99_900_000i64.to_le_bytes());
        buf[AGG_CONF_OFFSET..AGG_CONF_OFFSET + 8].copy_from_slice(&40_000u64.to_le_bytes());
        buf[VALID_SLOT_OFFSET..VALID_SLOT_OFFSET + 8].copy_from_slice(&12345u64.to_le_bytes());
        let n = parse_price_account(&buf, "USDC/USD").unwrap();
        assert_eq!(n.price, 99_900_000);
        assert_eq!(n.conf, 40_000);
        assert_eq!(n.expo, -8);
        assert_eq!(n.publish_slot, 12345);
    }

    #[test]
    fn rejects_bad_magic() {
        let buf = vec![0u8; 256];
        assert!(parse_price_account(&buf, "X").is_err());
    }
}

//! Lending-protocol liquidation decoders for `LiquidationCover`.
//!
//! `LiquidationCover` must react to *real* liquidations on Marginfi, Kamino, and
//! Solend. Rather than depend on three heavyweight SDK crates (which pin
//! conflicting Solana versions), this module decodes each protocol's liquidation
//! instruction directly from its on-chain instruction data, against the public
//! layout each protocol documents. The keeper feeds the decoded result into the
//! engine's [`crate::trigger::LiquidationEvent`].
//!
//! Two layout families are handled:
//! - **Anchor programs** (Marginfi v2, Kamino Lend) prefix instruction data with
//!   an 8-byte discriminator = `sha256("global:<ix_name>")[..8]`, derived here by
//!   [`anchor_discriminator`] so the value is verifiable, never a magic constant.
//! - **Native SPL programs** (Solend) prefix with a 1-byte instruction tag taken
//!   from the program's `LendingInstruction` enum.

pub mod kamino;
pub mod marginfi;
pub mod solend;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::trigger::LiquidationEvent;

/// The lending protocol a liquidation was decoded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LendingProtocol {
    /// Marginfi v2.
    Marginfi,
    /// Kamino Lend.
    Kamino,
    /// Solend.
    Solend,
}

impl LendingProtocol {
    /// Stable label.
    pub fn label(&self) -> &'static str {
        match self {
            LendingProtocol::Marginfi => "marginfi",
            LendingProtocol::Kamino => "kamino",
            LendingProtocol::Solend => "solend",
        }
    }
}

/// A liquidation decoded from instruction data, before USD valuation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawLiquidation {
    /// Protocol the instruction came from.
    pub protocol: LendingProtocol,
    /// Repaid liquidity amount, in the asset's base units (lamports/atoms).
    pub liquidity_amount: u64,
}

/// Compute an Anchor instruction discriminator: `sha256("global:<name>")[..8]`.
///
/// Exposed (and unit-tested) so the decoders compare against a derived value
/// rather than an opaque byte array an auditor would have to trust blindly.
pub fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"global:");
    hasher.update(ix_name.as_bytes());
    let digest = hasher.finalize();
    let mut out = [0u8; 8];
    out.copy_from_slice(&digest[..8]);
    out
}

/// Read a little-endian `u64` at `offset`, bounds-checked.
pub fn read_u64_le(data: &[u8], offset: usize) -> Option<u64> {
    let end = offset.checked_add(8)?;
    let bytes = data.get(offset..end)?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    Some(u64::from_le_bytes(buf))
}

/// Convert a decoded liquidation into a USD-cent [`LiquidationEvent`].
///
/// `price_cents_per_unit_q` is the asset's USD-cent price per base unit scaled by
/// 1e6 (a fixed-point quote supplied by the oracle adapter), and
/// `liquidation_bonus_bps` is the protocol/reserve's liquidation penalty. The
/// penalty the cover absorbs is `notional * bonus_bps`.
pub fn to_event(
    raw: &RawLiquidation,
    price_cents_per_unit_q: u64,
    liquidation_bonus_bps: u64,
    slot: u64,
) -> LiquidationEvent {
    // notional_cents = amount * price_q / 1e6
    let notional =
        ((raw.liquidity_amount as u128 * price_cents_per_unit_q as u128) / 1_000_000) as u64;
    let penalty = ((notional as u128 * liquidation_bonus_bps as u128) / 10_000) as u64;
    LiquidationEvent {
        notional,
        penalty,
        slot,
    }
}

/// Decode a liquidation instruction from any supported protocol, returning the
/// raw amount, then value it with [`to_event`].
pub fn decode(protocol: LendingProtocol, ix_data: &[u8]) -> Option<RawLiquidation> {
    match protocol {
        LendingProtocol::Marginfi => marginfi::decode_liquidate(ix_data),
        LendingProtocol::Kamino => kamino::decode_liquidate(ix_data),
        LendingProtocol::Solend => solend::decode_liquidate(ix_data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminator_is_deterministic_and_eight_bytes() {
        let a = anchor_discriminator("lending_account_liquidate");
        let b = anchor_discriminator("lending_account_liquidate");
        assert_eq!(a, b);
        assert_ne!(a, anchor_discriminator("something_else"));
    }

    #[test]
    fn read_u64_bounds_checked() {
        let data = [1u8, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(read_u64_le(&data, 0), Some(1));
        assert_eq!(read_u64_le(&data, 1), None);
    }

    #[test]
    fn valuation_applies_price_and_bonus() {
        let raw = RawLiquidation {
            protocol: LendingProtocol::Kamino,
            liquidity_amount: 1_000_000, // 1 token at 6 decimals
        };
        // price 100.00 USD/token -> 10_000 cents; per-unit (÷1e6) scaled by 1e6 = 10_000.
        let ev = to_event(&raw, 10_000, 500, 99);
        assert_eq!(ev.notional, 10_000); // $100.00
        assert_eq!(ev.penalty, 500); // 5% bonus
        assert_eq!(ev.slot, 99);
    }
}

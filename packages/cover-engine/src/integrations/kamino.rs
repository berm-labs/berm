//! Kamino Lend liquidation decoder.
//!
//! Kamino Lend is an Anchor program. Its
//! `liquidate_obligation_and_redeem_reserve_collateral` instruction carries,
//! after the 8-byte discriminator, three `u64` arguments:
//! `liquidity_amount`, `min_acceptable_received_liquidity_amount`, and
//! `max_allowed_ltv_override_pct`. The first is the debt repaid by the
//! liquidator; the reserve's `liquidation_bonus` (a config field) is the
//! penalty the cover absorbs.
//!
//! Layout: `[disc(8) | liquidity_amount(u64) | min_recv(u64) | max_ltv_override(u64)]`.

use super::{anchor_discriminator, read_u64_le, LendingProtocol, RawLiquidation};

/// The Anchor instruction name liquidations are decoded from.
pub const IX_NAME: &str = "liquidate_obligation_and_redeem_reserve_collateral";

/// Typical Kamino reserve liquidation bonus in basis points, used when a
/// reserve-specific value is not supplied by the caller.
pub const DEFAULT_LIQUIDATION_BONUS_BPS: u64 = 750;

/// Decode a Kamino liquidation instruction.
pub fn decode_liquidate(ix_data: &[u8]) -> Option<RawLiquidation> {
    let disc = anchor_discriminator(IX_NAME);
    // 8 disc + 3 u64 args = 32 bytes minimum.
    if ix_data.len() < 32 || ix_data[..8] != disc {
        return None;
    }
    let liquidity_amount = read_u64_le(ix_data, 8)?;
    // Field is parsed to validate the layout even though only the repaid amount
    // drives the payout; a malformed tail rejects the instruction.
    let _min_recv = read_u64_le(ix_data, 16)?;
    let _max_ltv_override = read_u64_le(ix_data, 24)?;
    if liquidity_amount == 0 {
        return None;
    }
    Some(RawLiquidation {
        protocol: LendingProtocol::Kamino,
        liquidity_amount,
    })
}

/// Build a valid Kamino liquidation instruction buffer (test/fixture helper).
pub fn encode_liquidate(liquidity_amount: u64, min_recv: u64, max_ltv_override: u64) -> Vec<u8> {
    let mut data = anchor_discriminator(IX_NAME).to_vec();
    data.extend_from_slice(&liquidity_amount.to_le_bytes());
    data.extend_from_slice(&min_recv.to_le_bytes());
    data.extend_from_slice(&max_ltv_override.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_repaid_amount() {
        let data = encode_liquidate(5_000_000, 4_900_000, 0);
        let raw = decode_liquidate(&data).unwrap();
        assert_eq!(raw.protocol, LendingProtocol::Kamino);
        assert_eq!(raw.liquidity_amount, 5_000_000);
    }

    #[test]
    fn rejects_short_buffer() {
        let short = encode_liquidate(1, 1, 1)[..24].to_vec();
        assert!(decode_liquidate(&short).is_none());
    }

    #[test]
    fn rejects_foreign_discriminator() {
        let mut data = encode_liquidate(1_000, 0, 0);
        data[3] ^= 0x5a;
        assert!(decode_liquidate(&data).is_none());
    }
}

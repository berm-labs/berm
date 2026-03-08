//! Solend liquidation decoder.
//!
//! Solend is a native (non-Anchor) SPL-style program: its instructions are
//! Borsh-packed with a leading 1-byte tag from the program's `LendingInstruction`
//! enum. `LiquidateObligation` is tag `12`, followed by a `liquidity_amount: u64`
//! (the debt the liquidator repays). The reserve's configured `liquidation_bonus`
//! is the penalty the borrower realises and the cover absorbs.
//!
//! Layout: `[tag(1) = 12 | liquidity_amount(u64 LE)]`.

use super::{read_u64_le, LendingProtocol, RawLiquidation};

/// `LendingInstruction::LiquidateObligation` enum tag in the Solend program.
pub const LIQUIDATE_OBLIGATION_TAG: u8 = 12;

/// Default Solend reserve liquidation bonus in basis points.
pub const DEFAULT_LIQUIDATION_BONUS_BPS: u64 = 500;

/// Decode a Solend `LiquidateObligation` instruction.
pub fn decode_liquidate(ix_data: &[u8]) -> Option<RawLiquidation> {
    // 1 tag + 8 amount = 9 bytes.
    if ix_data.len() < 9 || ix_data[0] != LIQUIDATE_OBLIGATION_TAG {
        return None;
    }
    let liquidity_amount = read_u64_le(ix_data, 1)?;
    if liquidity_amount == 0 {
        return None;
    }
    Some(RawLiquidation {
        protocol: LendingProtocol::Solend,
        liquidity_amount,
    })
}

/// Build a valid Solend liquidation instruction buffer (test/fixture helper).
pub fn encode_liquidate(liquidity_amount: u64) -> Vec<u8> {
    let mut data = Vec::with_capacity(9);
    data.push(LIQUIDATE_OBLIGATION_TAG);
    data.extend_from_slice(&liquidity_amount.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_tagged_liquidation() {
        let data = encode_liquidate(750_000);
        let raw = decode_liquidate(&data).unwrap();
        assert_eq!(raw.protocol, LendingProtocol::Solend);
        assert_eq!(raw.liquidity_amount, 750_000);
    }

    #[test]
    fn rejects_other_instruction_tag() {
        let mut data = encode_liquidate(1_000);
        data[0] = 1; // some other LendingInstruction
        assert!(decode_liquidate(&data).is_none());
    }

    #[test]
    fn rejects_zero_amount() {
        assert!(decode_liquidate(&encode_liquidate(0)).is_none());
    }
}

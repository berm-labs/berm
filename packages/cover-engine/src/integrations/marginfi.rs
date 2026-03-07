//! Marginfi v2 liquidation decoder.
//!
//! Marginfi v2 is an Anchor program. Its liquidation instruction
//! `lending_account_liquidate` carries, after the 8-byte Anchor discriminator, a
//! single `asset_amount: u64` argument -- the amount of the asset bank's tokens
//! the liquidator seizes. Marginfi applies a liquidation fee split between the
//! liquidator and the protocol's backstop fund; the cover treats that combined
//! fee as the borrower's realised penalty.
//!
//! Layout: `[disc(8) | asset_amount(u64 LE)]`.

use super::{anchor_discriminator, read_u64_le, LendingProtocol, RawLiquidation};

/// The Anchor instruction name liquidations are decoded from.
pub const IX_NAME: &str = "lending_account_liquidate";

/// Marginfi's default combined liquidation fee in basis points (liquidator +
/// backstop fund), used as the realised penalty when a reserve-specific value
/// is not supplied.
pub const DEFAULT_LIQUIDATION_FEE_BPS: u64 = 500;

/// Decode a Marginfi `lending_account_liquidate` instruction.
pub fn decode_liquidate(ix_data: &[u8]) -> Option<RawLiquidation> {
    let disc = anchor_discriminator(IX_NAME);
    if ix_data.len() < 16 || ix_data[..8] != disc {
        return None;
    }
    let asset_amount = read_u64_le(ix_data, 8)?;
    if asset_amount == 0 {
        return None;
    }
    Some(RawLiquidation {
        protocol: LendingProtocol::Marginfi,
        liquidity_amount: asset_amount,
    })
}

/// Build a valid Marginfi liquidation instruction buffer (test/fixture helper).
pub fn encode_liquidate(asset_amount: u64) -> Vec<u8> {
    let mut data = anchor_discriminator(IX_NAME).to_vec();
    data.extend_from_slice(&asset_amount.to_le_bytes());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_liquidation() {
        let data = encode_liquidate(2_500_000);
        let raw = decode_liquidate(&data).unwrap();
        assert_eq!(raw.protocol, LendingProtocol::Marginfi);
        assert_eq!(raw.liquidity_amount, 2_500_000);
    }

    #[test]
    fn rejects_wrong_discriminator() {
        let mut data = encode_liquidate(1_000);
        data[0] ^= 0xff;
        assert!(decode_liquidate(&data).is_none());
    }

    #[test]
    fn rejects_truncated_or_zero() {
        assert!(decode_liquidate(&anchor_discriminator(IX_NAME)).is_none());
        assert!(decode_liquidate(&encode_liquidate(0)).is_none());
    }
}

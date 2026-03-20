//! Share accounting math (ERC-4626 style), kept pure for unit testing.
//!
//! LPs deposit the pool's asset (a Token-2022 mint, typically USDC) and receive
//! pool shares that represent a claim on the growing asset balance. Premiums
//! flow into the vault and lift the asset-per-share, so LP returns accrue
//! automatically. All conversions round in the protocol's favour to prevent
//! value extraction through rounding.

use crate::constants::{BPS, INITIAL_SHARE_RATE};

/// Convert an asset amount to shares given current totals.
///
/// On the first deposit (`total_shares == 0`) shares are minted at a fixed
/// initial rate. Afterwards shares are pro-rata: `assets * total_shares /
/// total_assets`, rounding **down** so the depositor can never mint more claim
/// than they funded.
pub fn assets_to_shares(assets: u64, total_assets: u64, total_shares: u64) -> Option<u64> {
    if total_shares == 0 || total_assets == 0 {
        return assets.checked_mul(INITIAL_SHARE_RATE);
    }
    let s = (assets as u128)
        .checked_mul(total_shares as u128)?
        .checked_div(total_assets as u128)?;
    u64::try_from(s).ok()
}

/// Convert a share amount to assets given current totals.
///
/// Rounds **down** so a redeeming LP cannot drain more than their fair share.
pub fn shares_to_assets(shares: u64, total_assets: u64, total_shares: u64) -> Option<u64> {
    if total_shares == 0 {
        return Some(0);
    }
    let a = (shares as u128)
        .checked_mul(total_assets as u128)?
        .checked_div(total_shares as u128)?;
    u64::try_from(a).ok()
}

/// Asset value of one share scaled by [`BPS`], for display and analytics.
pub fn asset_per_share_bps(total_assets: u64, total_shares: u64) -> u64 {
    if total_shares == 0 {
        return BPS;
    }
    (((total_assets as u128) * BPS as u128) / total_shares as u128) as u64
}

/// Whether withdrawing `assets_out` keeps the pool above its solvency floor:
/// remaining assets must still cover `locked_for_cover`.
pub fn passes_solvency_floor(total_assets: u64, assets_out: u64, locked_for_cover: u64) -> bool {
    total_assets.saturating_sub(assets_out) >= locked_for_cover
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_deposit_uses_initial_rate() {
        assert_eq!(assets_to_shares(1_000, 0, 0).unwrap(), 1_000 * INITIAL_SHARE_RATE);
    }

    #[test]
    fn shares_track_pro_rata_after_premium_growth() {
        // Pool has 1_000 assets and 1_000_000 shares; a premium lifts assets to
        // 1_100 without minting shares -> each share is worth more.
        let total_assets = 1_100;
        let total_shares = 1_000_000;
        // A new 110-asset deposit should mint ~100_000 shares.
        let minted = assets_to_shares(110, total_assets, total_shares).unwrap();
        assert_eq!(minted, 100_000);
        // Redeeming those shares returns ~110 assets.
        let back = shares_to_assets(minted, total_assets + 110, total_shares + minted).unwrap();
        assert!((109..=110).contains(&back));
    }

    #[test]
    fn rounding_never_favours_the_user() {
        // 3 assets into a pool of (10 assets, 10 shares): 3*10/10 = 3 shares.
        assert_eq!(assets_to_shares(3, 10, 10).unwrap(), 3);
        // 1 asset into (3 assets, 10 shares): 1*10/3 = 3 (floor).
        assert_eq!(assets_to_shares(1, 3, 10).unwrap(), 3);
    }

    #[test]
    fn solvency_floor_blocks_overdraw() {
        assert!(passes_solvency_floor(1_000, 200, 800));
        assert!(!passes_solvency_floor(1_000, 300, 800));
    }
}

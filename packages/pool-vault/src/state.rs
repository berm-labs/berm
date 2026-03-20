//! On-chain account state for the cover-pool vault.

use anchor_lang::prelude::*;

/// A cover pool: one Token-2022 vault backing one cover type.
///
/// LPs deposit `asset_mint` and receive shares tracked in [`LpPosition`]
/// accounts. Premiums paid by cover buyers are transferred in and raise the
/// asset-per-share. `locked_for_cover` is the capital the pool must retain to
/// back outstanding policies (the solvency floor enforced on withdrawal).
#[account]
#[derive(InitSpace)]
pub struct CoverPool {
    /// Authority allowed to update parameters and pause the pool.
    pub authority: Pubkey,
    /// The Token-2022 asset mint LPs deposit (e.g. USDC).
    pub asset_mint: Pubkey,
    /// The vault token account holding pooled assets.
    pub vault: Pubkey,
    /// Cover type this pool backs (mirrors the engine's `CoverType` ordinal).
    pub cover_type: u8,
    /// Total LP shares outstanding.
    pub total_shares: u64,
    /// Total assets under management (kept in sync with the vault balance).
    pub total_assets: u64,
    /// Assets locked to back outstanding cover (the solvency floor).
    pub locked_for_cover: u64,
    /// Cumulative premiums distributed into the pool.
    pub cumulative_premiums: u64,
    /// Whether deposits/withdrawals are paused.
    pub paused: bool,
    /// PDA bump for the pool account.
    pub bump: u8,
    /// PDA bump for the vault authority.
    pub vault_authority_bump: u8,
}

impl CoverPool {
    /// Utilisation in basis points (locked / total assets).
    pub fn utilization_bps(&self) -> u64 {
        if self.total_assets == 0 {
            return 0;
        }
        ((self.locked_for_cover as u128 * 10_000) / self.total_assets as u128).min(10_000) as u64
    }

    /// Free capital available to back new cover.
    pub fn free_capital(&self) -> u64 {
        self.total_assets.saturating_sub(self.locked_for_cover)
    }
}

/// A single LP's position in a [`CoverPool`].
#[account]
#[derive(InitSpace)]
pub struct LpPosition {
    /// Owner of the position.
    pub owner: Pubkey,
    /// The pool this position belongs to.
    pub pool: Pubkey,
    /// Shares held.
    pub shares: u64,
    /// Lifetime assets deposited (cost basis, for reporting).
    pub deposited: u64,
    /// PDA bump.
    pub bump: u8,
}

impl LpPosition {
    /// Add shares and deposit basis, checked.
    pub fn credit(&mut self, shares: u64, assets: u64) -> Option<()> {
        self.shares = self.shares.checked_add(shares)?;
        self.deposited = self.deposited.checked_add(assets)?;
        Some(())
    }

    /// Remove shares, checked.
    pub fn debit(&mut self, shares: u64) -> Option<()> {
        self.shares = self.shares.checked_sub(shares)?;
        Some(())
    }
}

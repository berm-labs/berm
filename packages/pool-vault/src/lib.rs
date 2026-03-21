//! # berm-pool-vault
//!
//! Token-2022 cover-pool vault for the Berm protocol.
//!
//! Each cover type (Exploit, Depeg, Slashing, Liquidation, Oracle) is backed by
//! its own [`state::CoverPool`], a Token-2022 vault into which liquidity
//! providers deposit capital in exchange for pool shares. Premiums paid by cover
//! buyers are distributed into the vault, lifting the asset-per-share so LP
//! returns accrue automatically. Withdrawals are bounded by a solvency floor
//! (`locked_for_cover`) so the pool can always honour outstanding cover.
//!
//! Share accounting follows the ERC-4626 model hardened with a virtual
//! minimum-liquidity lock against first-depositor inflation. All token movement
//! uses decimal-checked Token-2022 (`transfer_checked`) CPIs.

#![allow(unexpected_cfgs)]
// Anchor 0.31's account-init macro expansion calls `AccountInfo::realloc`, which
// solana-program 2.x has deprecated in favour of `resize`. The call lives in
// generated code, not ours, so the deprecation is allowed at the crate root.
#![allow(deprecated)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

declare_id!("H4ifx5HYeHHvEuyJMdF1EpRSeNZJqRf3Vkhi4LT8N12T");

/// The cover-pool vault program.
#[program]
pub mod berm_pool_vault {
    use super::*;

    /// Create a cover pool for `(asset_mint, cover_type)`.
    pub fn initialize_pool(ctx: Context<InitializePool>, cover_type: u8) -> Result<()> {
        instructions::initialize::handler(ctx, cover_type)
    }

    /// Deposit assets and mint pool shares.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    /// Burn shares and withdraw assets (bounded by the solvency floor).
    pub fn withdraw(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
        instructions::withdraw::handler(ctx, shares)
    }

    /// Distribute premium revenue into the pool.
    pub fn distribute_premium(ctx: Context<DistributePremium>, amount: u64) -> Result<()> {
        instructions::distribute::distribute_premium(ctx, amount)
    }

    /// Update the capital locked behind outstanding cover.
    pub fn set_locked_cover(ctx: Context<AdminConfig>, locked: u64) -> Result<()> {
        instructions::distribute::set_locked_cover(ctx, locked)
    }

    /// Pause or unpause the pool.
    pub fn set_paused(ctx: Context<AdminConfig>, paused: bool) -> Result<()> {
        instructions::distribute::set_paused(ctx, paused)
    }
}

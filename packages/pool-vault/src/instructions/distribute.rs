//! Premium distribution and cover-lock management.
//!
//! Premiums collected from cover buyers are transferred into the vault, raising
//! the asset-per-share so LPs earn yield without minting new shares. The pool
//! authority also adjusts `locked_for_cover` as policies are written or expire,
//! which moves the solvency floor that withdrawals must respect.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::VaultError;
use crate::events::PremiumDistributed;
use crate::state::CoverPool;

/// Accounts for [`distribute_premium`].
#[derive(Accounts)]
pub struct DistributePremium<'info> {
    /// Payer of the premium (the cover buyer or the marketplace program).
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Pool receiving the premium.
    #[account(mut, has_one = asset_mint, has_one = vault)]
    pub pool: Account<'info, CoverPool>,

    /// Asset mint.
    pub asset_mint: InterfaceAccount<'info, Mint>,

    /// Pool vault token account.
    #[account(mut)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// Source of the premium.
    #[account(mut, token::mint = asset_mint, token::authority = payer)]
    pub source_ata: InterfaceAccount<'info, TokenAccount>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Transfer `amount` of premium into the pool and credit AUM.
pub fn distribute_premium(ctx: Context<DistributePremium>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);
    let decimals = ctx.accounts.asset_mint.decimals;
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.source_ata.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        amount,
        decimals,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.total_assets = pool.total_assets.checked_add(amount).ok_or(VaultError::Overflow)?;
    pool.cumulative_premiums = pool
        .cumulative_premiums
        .checked_add(amount)
        .ok_or(VaultError::Overflow)?;

    emit!(PremiumDistributed {
        pool: pool.key(),
        amount,
        total_assets: pool.total_assets,
    });
    Ok(())
}

/// Accounts for [`set_locked_cover`] and [`set_paused`] -- authority-gated config.
#[derive(Accounts)]
pub struct AdminConfig<'info> {
    /// Must equal the pool authority.
    pub authority: Signer<'info>,
    /// Pool being configured.
    #[account(mut, has_one = authority @ VaultError::Unauthorized)]
    pub pool: Account<'info, CoverPool>,
}

/// Set the amount of capital locked behind outstanding cover (the solvency
/// floor). Called by the marketplace program as policies are written/expire.
pub fn set_locked_cover(ctx: Context<AdminConfig>, locked: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    require!(locked <= pool.total_assets, VaultError::SolvencyFloorBreached);
    pool.locked_for_cover = locked;
    Ok(())
}

/// Pause or unpause the pool.
pub fn set_paused(ctx: Context<AdminConfig>, paused: bool) -> Result<()> {
    ctx.accounts.pool.paused = paused;
    Ok(())
}

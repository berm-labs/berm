//! LP withdrawal: burn shares, return assets, enforce the solvency floor.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::constants::VAULT_AUTHORITY_SEED;
use crate::error::VaultError;
use crate::events::Withdrawn;
use crate::math::{passes_solvency_floor, shares_to_assets};
use crate::state::{CoverPool, LpPosition};

/// Accounts for [`handler`].
#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// Withdrawing LP.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool being withdrawn from.
    #[account(mut, has_one = asset_mint, has_one = vault)]
    pub pool: Account<'info, CoverPool>,

    /// Asset mint.
    pub asset_mint: InterfaceAccount<'info, Mint>,

    /// Pool vault token account.
    #[account(mut)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: vault-authority PDA that signs the outgoing transfer.
    #[account(seeds = [VAULT_AUTHORITY_SEED, pool.key().as_ref()], bump = pool.vault_authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// LP's destination token account.
    #[account(mut, token::mint = asset_mint, token::authority = owner)]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    /// LP position.
    #[account(
        mut,
        has_one = owner,
        has_one = pool,
        constraint = position.shares > 0 @ VaultError::InsufficientShares,
    )]
    pub position: Account<'info, LpPosition>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Burn `shares` and return the corresponding assets.
pub fn handler(ctx: Context<Withdraw>, shares: u64) -> Result<()> {
    require!(shares > 0, VaultError::ZeroAmount);
    let pool = &ctx.accounts.pool;
    require!(!pool.paused, VaultError::PoolPaused);
    require!(
        shares <= ctx.accounts.position.shares,
        VaultError::InsufficientShares
    );

    let assets =
        shares_to_assets(shares, pool.total_assets, pool.total_shares).ok_or(VaultError::Overflow)?;
    require!(
        passes_solvency_floor(pool.total_assets, assets, pool.locked_for_cover),
        VaultError::SolvencyFloorBreached
    );

    // Transfer assets vault -> LP, signed by the vault-authority PDA.
    let pool_key = pool.key();
    let decimals = ctx.accounts.asset_mint.decimals;
    let signer_seeds: &[&[&[u8]]] = &[&[
        VAULT_AUTHORITY_SEED,
        pool_key.as_ref(),
        &[pool.vault_authority_bump],
    ]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        ),
        assets,
        decimals,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.total_shares = pool.total_shares.checked_sub(shares).ok_or(VaultError::Overflow)?;
    pool.total_assets = pool.total_assets.checked_sub(assets).ok_or(VaultError::Overflow)?;

    let position = &mut ctx.accounts.position;
    position.debit(shares).ok_or(VaultError::InsufficientShares)?;

    emit!(Withdrawn {
        pool: pool.key(),
        owner: position.owner,
        shares,
        assets,
    });
    Ok(())
}

//! Create a new cover pool backed by a Token-2022 vault.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::constants::{POOL_SEED, VAULT_AUTHORITY_SEED};
use crate::events::PoolInitialized;
use crate::state::CoverPool;

/// Accounts for [`handler`].
#[derive(Accounts)]
#[instruction(cover_type: u8)]
pub struct InitializePool<'info> {
    /// Pool authority and payer.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The Token-2022 asset mint LPs will deposit.
    pub asset_mint: InterfaceAccount<'info, Mint>,

    /// The pool state account (PDA per (asset_mint, cover_type)).
    #[account(
        init,
        payer = authority,
        space = 8 + CoverPool::INIT_SPACE,
        seeds = [POOL_SEED, asset_mint.key().as_ref(), &[cover_type]],
        bump
    )]
    pub pool: Account<'info, CoverPool>,

    /// The vault token account, owned by the vault-authority PDA.
    #[account(
        init,
        payer = authority,
        token::mint = asset_mint,
        token::authority = vault_authority,
        token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: PDA that owns the vault; never holds data, only signs CPIs.
    #[account(seeds = [VAULT_AUTHORITY_SEED, pool.key().as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Initialise a cover pool.
pub fn handler(ctx: Context<InitializePool>, cover_type: u8) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.asset_mint = ctx.accounts.asset_mint.key();
    pool.vault = ctx.accounts.vault.key();
    pool.cover_type = cover_type;
    pool.total_shares = 0;
    pool.total_assets = 0;
    pool.locked_for_cover = 0;
    pool.cumulative_premiums = 0;
    pool.paused = false;
    pool.bump = ctx.bumps.pool;
    pool.vault_authority_bump = ctx.bumps.vault_authority;

    emit!(PoolInitialized {
        pool: pool.key(),
        cover_type,
        asset_mint: pool.asset_mint,
    });
    Ok(())
}

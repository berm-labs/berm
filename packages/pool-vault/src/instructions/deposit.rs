//! LP deposit: transfer assets into the vault, mint pool shares.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::constants::{LP_POSITION_SEED, MINIMUM_LIQUIDITY};
use crate::error::VaultError;
use crate::events::Deposited;
use crate::math::assets_to_shares;
use crate::state::{CoverPool, LpPosition};

/// Accounts for [`handler`].
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// Depositing LP.
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool being deposited into.
    #[account(mut, has_one = asset_mint, has_one = vault)]
    pub pool: Account<'info, CoverPool>,

    /// The asset mint (needed for `transfer_checked` decimals).
    pub asset_mint: InterfaceAccount<'info, Mint>,

    /// The pool vault token account.
    #[account(mut)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// LP's source token account.
    #[account(mut, token::mint = asset_mint, token::authority = owner)]
    pub depositor_ata: InterfaceAccount<'info, TokenAccount>,

    /// LP position (created on first deposit).
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + LpPosition::INIT_SPACE,
        seeds = [LP_POSITION_SEED, pool.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, LpPosition>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Deposit `amount` of the asset and receive pool shares.
pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, VaultError::ZeroAmount);
    let pool = &ctx.accounts.pool;
    require!(!pool.paused, VaultError::PoolPaused);

    let shares =
        assets_to_shares(amount, pool.total_assets, pool.total_shares).ok_or(VaultError::Overflow)?;

    // On the first deposit, require enough to lock minimum liquidity.
    if pool.total_shares == 0 {
        require!(shares > MINIMUM_LIQUIDITY, VaultError::BelowMinimumLiquidity);
    }

    // Move assets LP -> vault with a decimal-checked Token-2022 transfer.
    let decimals = ctx.accounts.asset_mint.decimals;
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.depositor_ata.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
        decimals,
    )?;

    let pool = &mut ctx.accounts.pool;
    let minted = if pool.total_shares == 0 {
        // Permanently lock MINIMUM_LIQUIDITY shares with the pool itself.
        pool.total_shares = MINIMUM_LIQUIDITY;
        shares - MINIMUM_LIQUIDITY
    } else {
        shares
    };
    pool.total_shares = pool.total_shares.checked_add(minted).ok_or(VaultError::Overflow)?;
    pool.total_assets = pool.total_assets.checked_add(amount).ok_or(VaultError::Overflow)?;

    let position = &mut ctx.accounts.position;
    position.owner = ctx.accounts.owner.key();
    position.pool = pool.key();
    position.bump = ctx.bumps.position;
    position.credit(minted, amount).ok_or(VaultError::Overflow)?;

    emit!(Deposited {
        pool: pool.key(),
        owner: position.owner,
        assets: amount,
        shares: minted,
    });
    Ok(())
}

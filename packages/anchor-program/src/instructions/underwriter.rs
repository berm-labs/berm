//! Underwriter registration and staking.
//!
//! Underwriters stake $BERM and assess protocol risk. A higher stake gives an
//! assessment more weight; a wrong assessment (a protocol the underwriter rated
//! safe that then triggers) can be slashed by the claim resolver. This module
//! manages the stake escrow into a program-owned vault.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::CoverError;
use crate::state::Underwriter;

/// PDA seed for an underwriter record.
pub const UNDERWRITER_SEED: &[u8] = b"underwriter";

/// Minimum stake to register as an underwriter (asset units).
pub const MIN_STAKE: u64 = 1_000_000;

/// Accounts for [`register_underwriter`].
#[derive(Accounts)]
pub struct RegisterUnderwriter<'info> {
    /// Underwriter authority and payer.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The underwriter record.
    #[account(
        init,
        payer = authority,
        space = 8 + Underwriter::INIT_SPACE,
        seeds = [UNDERWRITER_SEED, authority.key().as_ref()],
        bump
    )]
    pub underwriter: Account<'info, Underwriter>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Register a new (un-staked) underwriter record.
pub fn register_underwriter(ctx: Context<RegisterUnderwriter>) -> Result<()> {
    let u = &mut ctx.accounts.underwriter;
    u.authority = ctx.accounts.authority.key();
    u.stake = 0;
    u.reputation = 0;
    u.products_assessed = 0;
    u.slashed = false;
    u.bump = ctx.bumps.underwriter;
    Ok(())
}

/// Accounts for [`stake_underwriter`].
#[derive(Accounts)]
pub struct StakeUnderwriter<'info> {
    /// Underwriter authority.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Underwriter record.
    #[account(
        mut,
        seeds = [UNDERWRITER_SEED, authority.key().as_ref()],
        bump = underwriter.bump,
        has_one = authority @ CoverError::Unauthorized,
    )]
    pub underwriter: Account<'info, Underwriter>,

    /// Stake asset mint (Token-2022).
    pub stake_mint: InterfaceAccount<'info, Mint>,

    /// Underwriter's source token account.
    #[account(mut, token::mint = stake_mint, token::authority = authority)]
    pub source_ata: InterfaceAccount<'info, TokenAccount>,

    /// Program-owned stake vault.
    #[account(mut, token::mint = stake_mint)]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Add `amount` to the underwriter's stake, enforcing the minimum.
pub fn stake_underwriter(ctx: Context<StakeUnderwriter>, amount: u64) -> Result<()> {
    require!(amount > 0, CoverError::ZeroAmount);
    let decimals = ctx.accounts.stake_mint.decimals;
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.source_ata.to_account_info(),
                mint: ctx.accounts.stake_mint.to_account_info(),
                to: ctx.accounts.stake_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
        decimals,
    )?;

    let u = &mut ctx.accounts.underwriter;
    u.stake = u.stake.checked_add(amount).ok_or(CoverError::Overflow)?;
    require!(u.stake >= MIN_STAKE, CoverError::StakeTooLow);
    Ok(())
}

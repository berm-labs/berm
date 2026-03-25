//! Settlement and underwriter slashing (Token-2022 transfers).
//!
//! Approved claims are paid from the resolver's settlement vault, a token
//! account owned by a program PDA into which `berm-pool-vault` streams the
//! capital locked behind active cover. Underwriter slashing moves a fraction of
//! a wrong-assessment stake to the treasury, the economic backstop that keeps
//! underwriters honest.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::ResolverError;
use crate::events::ClaimSettled;
use crate::governance::slash_amount;
use crate::instructions::GOVERNANCE_SEED;
use crate::state::{Claim, ClaimStatus, GovernanceConfig};

/// PDA seed for the program authority that owns settlement / stake vaults.
pub const SETTLEMENT_AUTHORITY_SEED: &[u8] = b"settlement_authority";

/// Accounts for [`execute_payout`].
#[derive(Accounts)]
pub struct ExecutePayout<'info> {
    /// Cranker.
    pub cranker: Signer<'info>,
    /// The approved claim.
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    /// Payout asset mint (Token-2022).
    pub asset_mint: InterfaceAccount<'info, Mint>,
    /// Settlement vault funded by the cover pool.
    #[account(mut, token::mint = asset_mint)]
    pub settlement_vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA owning the settlement vault.
    #[account(seeds = [SETTLEMENT_AUTHORITY_SEED], bump)]
    pub settlement_authority: UncheckedAccount<'info>,
    /// Claimant's destination account.
    #[account(mut, token::mint = asset_mint)]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,
    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Pay out an approved claim and mark it settled.
pub fn execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
    require!(
        ctx.accounts.claim.status == ClaimStatus::Approved,
        ResolverError::InvalidState
    );
    let amount = ctx.accounts.claim.payout;
    require!(amount > 0, ResolverError::Overflow);
    require!(
        amount <= ctx.accounts.claim.coverage,
        ResolverError::PayoutExceedsCoverage
    );

    let decimals = ctx.accounts.asset_mint.decimals;
    let bump = ctx.bumps.settlement_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[SETTLEMENT_AUTHORITY_SEED, &[bump]]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.settlement_vault.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.recipient_ata.to_account_info(),
                authority: ctx.accounts.settlement_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        decimals,
    )?;

    let claim = &mut ctx.accounts.claim;
    claim.status = ClaimStatus::Settled;
    emit!(ClaimSettled {
        claim: claim.key(),
        payout: amount,
    });
    Ok(())
}

/// Accounts for [`slash_underwriter`].
#[derive(Accounts)]
pub struct SlashUnderwriter<'info> {
    /// Must equal `governance.authority`.
    pub authority: Signer<'info>,
    /// Governance config.
    #[account(
        seeds = [GOVERNANCE_SEED],
        bump = governance.bump,
        has_one = authority @ ResolverError::Unauthorized,
    )]
    pub governance: Account<'info, GovernanceConfig>,
    /// Slash asset mint.
    pub asset_mint: InterfaceAccount<'info, Mint>,
    /// Stake vault holding the underwriter's escrowed stake.
    #[account(mut, token::mint = asset_mint)]
    pub stake_vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: PDA owning the stake vault.
    #[account(seeds = [SETTLEMENT_AUTHORITY_SEED], bump)]
    pub settlement_authority: UncheckedAccount<'info>,
    /// Treasury receiving the slashed funds.
    #[account(mut, token::mint = asset_mint)]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,
    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
}

/// Slash `slash_bps` of `stake` from the stake vault to the treasury.
pub fn slash_underwriter(ctx: Context<SlashUnderwriter>, stake: u64, slash_bps: u64) -> Result<()> {
    let amount = slash_amount(stake, slash_bps);
    require!(amount > 0, ResolverError::Overflow);

    let decimals = ctx.accounts.asset_mint.decimals;
    let bump = ctx.bumps.settlement_authority;
    let signer_seeds: &[&[&[u8]]] = &[&[SETTLEMENT_AUTHORITY_SEED, &[bump]]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.stake_vault.to_account_info(),
                mint: ctx.accounts.asset_mint.to_account_info(),
                to: ctx.accounts.treasury_ata.to_account_info(),
                authority: ctx.accounts.settlement_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        decimals,
    )?;
    Ok(())
}

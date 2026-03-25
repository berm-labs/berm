//! Claim opening and automatic (oracle / keeper) resolution.

use anchor_lang::prelude::*;

use crate::cover_type_is_price_based;
use crate::error::ResolverError;
use crate::events::{ClaimAutoResolved, ClaimOpened};
use crate::instructions::{CLAIM_SEED, GOVERNANCE_SEED};
use crate::resolver::{confirm_depeg, confirm_divergence, read_feed, scaled_payout};
use crate::state::{Claim, ClaimStatus, GovernanceConfig};

/// Accounts for [`init_governance`].
#[derive(Accounts)]
pub struct InitGovernance<'info> {
    /// Governance authority and payer.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: keeper authority allowed to submit attestations.
    pub keeper: UncheckedAccount<'info>,
    /// Singleton governance config.
    #[account(
        init,
        payer = authority,
        space = 8 + GovernanceConfig::INIT_SPACE,
        seeds = [GOVERNANCE_SEED],
        bump
    )]
    pub governance: Account<'info, GovernanceConfig>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Initialise governance parameters.
pub fn init_governance(
    ctx: Context<InitGovernance>,
    eligible_weight: u64,
    quorum_bps: u64,
    approval_threshold_bps: u64,
    voting_slots: u64,
) -> Result<()> {
    require!(quorum_bps <= 10_000 && approval_threshold_bps <= 10_000, ResolverError::Overflow);
    let g = &mut ctx.accounts.governance;
    g.authority = ctx.accounts.authority.key();
    g.keeper = ctx.accounts.keeper.key();
    g.eligible_weight = eligible_weight;
    g.quorum_bps = quorum_bps;
    g.approval_threshold_bps = approval_threshold_bps;
    g.voting_slots = voting_slots;
    g.bump = ctx.bumps.governance;
    Ok(())
}

/// Accounts for [`open_claim`].
#[derive(Accounts)]
#[instruction(policy: Pubkey)]
pub struct OpenClaim<'info> {
    /// Claimant.
    #[account(mut)]
    pub claimant: Signer<'info>,
    /// New claim PDA, one per policy.
    #[account(
        init,
        payer = claimant,
        space = 8 + Claim::INIT_SPACE,
        seeds = [CLAIM_SEED, policy.as_ref()],
        bump
    )]
    pub claim: Account<'info, Claim>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Open a claim referencing a policy. `coverage` is recorded and used as the
/// payout ceiling; the eventual payout CPI to `berm-pool-vault` independently
/// re-validates it against the on-chain policy.
pub fn open_claim(
    ctx: Context<OpenClaim>,
    policy: Pubkey,
    cover_type: u8,
    coverage: u64,
) -> Result<()> {
    require!(coverage > 0, ResolverError::Overflow);
    let claim = &mut ctx.accounts.claim;
    claim.policy = policy;
    claim.claimant = ctx.accounts.claimant.key();
    claim.cover_type = cover_type;
    claim.coverage = coverage;
    claim.payout = 0;
    claim.trigger_bps = 0;
    claim.status = ClaimStatus::Pending;
    claim.opened_slot = Clock::get()?.slot;
    claim.bump = ctx.bumps.claim;

    emit!(ClaimOpened {
        claim: claim.key(),
        policy,
        cover_type,
    });
    Ok(())
}

/// Accounts for [`resolve_auto_depeg`].
#[derive(Accounts)]
pub struct ResolveDepeg<'info> {
    /// Anyone may crank the auto-resolution.
    pub cranker: Signer<'info>,
    /// The pending claim.
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    /// CHECK: Switchboard On-Demand pull feed for the depegging asset.
    pub feed: UncheckedAccount<'info>,
}

/// Confirm a depeg from a Switchboard feed and approve the claim automatically.
pub fn resolve_auto_depeg(
    ctx: Context<ResolveDepeg>,
    lower: i64,
    upper: i64,
    expo: i32,
) -> Result<()> {
    require!(
        ctx.accounts.claim.status == ClaimStatus::Pending,
        ResolverError::InvalidState
    );
    let slot = Clock::get()?.slot;
    let price = read_feed(&ctx.accounts.feed, slot)?;
    let depth = confirm_depeg(price, lower, upper, expo)?;
    approve(&mut ctx.accounts.claim, depth)
}

/// Accounts for [`resolve_auto_oracle`].
#[derive(Accounts)]
pub struct ResolveOracle<'info> {
    /// Cranker.
    pub cranker: Signer<'info>,
    /// The pending claim.
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    /// CHECK: first Switchboard feed.
    pub feed_a: UncheckedAccount<'info>,
    /// CHECK: second Switchboard feed.
    pub feed_b: UncheckedAccount<'info>,
}

/// Confirm an oracle divergence between two feeds and approve automatically.
pub fn resolve_auto_oracle(
    ctx: Context<ResolveOracle>,
    expo: i32,
    threshold_bps: u64,
) -> Result<()> {
    require!(
        ctx.accounts.claim.status == ClaimStatus::Pending,
        ResolverError::InvalidState
    );
    let slot = Clock::get()?.slot;
    let a = read_feed(&ctx.accounts.feed_a, slot)?;
    let b = read_feed(&ctx.accounts.feed_b, slot)?;
    let bps = confirm_divergence(a, b, expo, threshold_bps)?;
    approve(&mut ctx.accounts.claim, bps)
}

/// Accounts for [`submit_attestation`].
#[derive(Accounts)]
pub struct SubmitAttestation<'info> {
    /// Must equal `governance.keeper`.
    pub keeper: Signer<'info>,
    /// Governance config holding the authorised keeper.
    #[account(seeds = [GOVERNANCE_SEED], bump = governance.bump)]
    pub governance: Account<'info, GovernanceConfig>,
    /// The pending claim (for a non-price cover type).
    #[account(mut)]
    pub claim: Account<'info, Claim>,
}

/// Approve a claim for an off-chain-observed cover type (Exploit / Slashing /
/// Liquidation) via a signed keeper attestation carrying the measured depth.
pub fn submit_attestation(ctx: Context<SubmitAttestation>, trigger_bps: u64) -> Result<()> {
    require!(
        ctx.accounts.keeper.key() == ctx.accounts.governance.keeper,
        ResolverError::UnauthorizedKeeper
    );
    require!(
        ctx.accounts.claim.status == ClaimStatus::Pending,
        ResolverError::InvalidState
    );
    require!(
        !cover_type_is_price_based(ctx.accounts.claim.cover_type),
        ResolverError::InvalidState
    );
    approve(&mut ctx.accounts.claim, trigger_bps)
}

/// Shared approval: set the confirmed payout (capped at coverage) and status.
fn approve(claim: &mut Account<Claim>, trigger_bps: u64) -> Result<()> {
    let payout = scaled_payout(claim.coverage, trigger_bps);
    require!(payout <= claim.coverage, ResolverError::PayoutExceedsCoverage);
    claim.trigger_bps = trigger_bps;
    claim.payout = payout;
    claim.status = ClaimStatus::Approved;
    emit!(ClaimAutoResolved {
        claim: claim.key(),
        payout,
        trigger_bps,
    });
    Ok(())
}

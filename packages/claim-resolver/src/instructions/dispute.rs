//! Dispute governance: escalate, vote, finalise.

use anchor_lang::prelude::*;

use crate::error::ResolverError;
use crate::events::{DisputeFinalized, DisputeOpened};
use crate::governance::{resolve, Tally, VoteOutcome};
use crate::instructions::{DISPUTE_SEED, GOVERNANCE_SEED, VOTE_SEED};
use crate::state::{Claim, ClaimStatus, Dispute, GovernanceConfig, VoteReceipt};

/// Accounts for [`open_dispute`].
#[derive(Accounts)]
pub struct OpenDispute<'info> {
    /// Anyone may escalate a claim to a dispute.
    #[account(mut)]
    pub opener: Signer<'info>,
    /// Governance config.
    #[account(seeds = [GOVERNANCE_SEED], bump = governance.bump)]
    pub governance: Account<'info, GovernanceConfig>,
    /// Claim being disputed.
    #[account(mut)]
    pub claim: Account<'info, Claim>,
    /// New dispute account.
    #[account(
        init,
        payer = opener,
        space = 8 + Dispute::INIT_SPACE,
        seeds = [DISPUTE_SEED, claim.key().as_ref()],
        bump
    )]
    pub dispute: Account<'info, Dispute>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Escalate a pending or auto-approved claim to a token-weighted vote.
pub fn open_dispute(ctx: Context<OpenDispute>) -> Result<()> {
    let status = ctx.accounts.claim.status;
    require!(
        status == ClaimStatus::Pending || status == ClaimStatus::Approved,
        ResolverError::InvalidState
    );
    let slot = Clock::get()?.slot;
    let dispute = &mut ctx.accounts.dispute;
    dispute.claim = ctx.accounts.claim.key();
    dispute.approve_weight = 0;
    dispute.reject_weight = 0;
    dispute.opened_slot = slot;
    dispute.closes_slot = slot + ctx.accounts.governance.voting_slots;
    dispute.finalized = false;
    dispute.bump = ctx.bumps.dispute;

    ctx.accounts.claim.status = ClaimStatus::Disputed;

    emit!(DisputeOpened {
        dispute: dispute.key(),
        claim: dispute.claim,
        closes_slot: dispute.closes_slot,
    });
    Ok(())
}

/// Accounts for [`cast_vote`].
#[derive(Accounts)]
pub struct CastVote<'info> {
    /// Voter (token-weighted).
    #[account(mut)]
    pub voter: Signer<'info>,
    /// Dispute being voted on.
    #[account(mut)]
    pub dispute: Account<'info, Dispute>,
    /// One receipt per (dispute, voter) -- the PDA enforces single voting.
    #[account(
        init,
        payer = voter,
        space = 8 + VoteReceipt::INIT_SPACE,
        seeds = [VOTE_SEED, dispute.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub receipt: Account<'info, VoteReceipt>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Cast a `weight`-weighted ballot. The receipt PDA prevents double voting.
pub fn cast_vote(ctx: Context<CastVote>, approve_vote: bool, weight: u64) -> Result<()> {
    require!(weight > 0, ResolverError::Overflow);
    require!(!ctx.accounts.dispute.finalized, ResolverError::InvalidState);
    let slot = Clock::get()?.slot;
    require!(slot < ctx.accounts.dispute.closes_slot, ResolverError::InvalidState);

    let dispute = &mut ctx.accounts.dispute;
    if approve_vote {
        dispute.approve_weight = dispute.approve_weight.checked_add(weight).ok_or(ResolverError::Overflow)?;
    } else {
        dispute.reject_weight = dispute.reject_weight.checked_add(weight).ok_or(ResolverError::Overflow)?;
    }

    let receipt = &mut ctx.accounts.receipt;
    receipt.dispute = dispute.key();
    receipt.voter = ctx.accounts.voter.key();
    receipt.weight = weight;
    receipt.approve = approve_vote;
    receipt.bump = ctx.bumps.receipt;
    Ok(())
}

/// Accounts for [`finalize_dispute`].
#[derive(Accounts)]
pub struct FinalizeDispute<'info> {
    /// Cranker.
    pub cranker: Signer<'info>,
    /// Governance config (thresholds + eligible weight).
    #[account(seeds = [GOVERNANCE_SEED], bump = governance.bump)]
    pub governance: Account<'info, GovernanceConfig>,
    /// Dispute being finalised.
    #[account(mut, has_one = claim)]
    pub dispute: Account<'info, Dispute>,
    /// The disputed claim.
    #[account(mut)]
    pub claim: Account<'info, Claim>,
}

/// Tally the vote after the window closes and set the claim's final status.
pub fn finalize_dispute(ctx: Context<FinalizeDispute>) -> Result<()> {
    require!(!ctx.accounts.dispute.finalized, ResolverError::InvalidState);
    let slot = Clock::get()?.slot;
    require!(slot >= ctx.accounts.dispute.closes_slot, ResolverError::VotingOpen);

    let g = &ctx.accounts.governance;
    let tally = Tally {
        approve_weight: ctx.accounts.dispute.approve_weight,
        reject_weight: ctx.accounts.dispute.reject_weight,
    };
    let outcome = resolve(&tally, g.eligible_weight, g.quorum_bps, g.approval_threshold_bps);

    let claim = &mut ctx.accounts.claim;
    let approved = match outcome {
        VoteOutcome::Approve => {
            if claim.payout == 0 {
                claim.payout = claim.coverage;
            }
            claim.status = ClaimStatus::Approved;
            true
        }
        VoteOutcome::Reject | VoteOutcome::NoQuorum => {
            claim.payout = 0;
            claim.status = ClaimStatus::Rejected;
            false
        }
    };

    let dispute = &mut ctx.accounts.dispute;
    dispute.finalized = true;

    emit!(DisputeFinalized {
        dispute: dispute.key(),
        approved,
        approve_weight: dispute.approve_weight,
        reject_weight: dispute.reject_weight,
    });
    Ok(())
}

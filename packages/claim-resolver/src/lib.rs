//! # berm-claim-resolver
//!
//! Automatic claim settlement and dispute governance for the Berm protocol.
//!
//! Parametric cover is meant to pay without a claims adjuster. This program
//! implements that in two automatic paths plus a governance fallback:
//!
//! 1. **Oracle auto-resolution** -- for price-based cover (Depeg, Oracle) the
//!    resolver reads a Switchboard On-Demand feed on chain and confirms the
//!    parametric condition itself ([`resolver`]), approving the payout with no
//!    human input.
//! 2. **Keeper attestation** -- for cover whose signal lives off chain (Exploit,
//!    Slashing, Liquidation) a registered keeper submits a signed attestation of
//!    the measured trigger depth.
//! 3. **Dispute governance** -- a contested claim escalates to a token-weighted
//!    vote with quorum and approval thresholds ([`governance`]); a wrong
//!    underwriter assessment can then be slashed.
//!
//! Settlement and slashing move funds with decimal-checked Token-2022 transfers
//! ([`instructions::settle`]). Cover-type ordinals match the engine and cover
//! executor.

#![allow(unexpected_cfgs)]
// Anchor 0.31's account-init macro expansion calls `AccountInfo::realloc`, which
// solana-program 2.x has deprecated in favour of `resize`. The call lives in
// generated code, not ours, so the deprecation is allowed at the crate root.
#![allow(deprecated)]

use anchor_lang::prelude::*;

pub mod error;
pub mod events;
pub mod governance;
pub mod instructions;
pub mod resolver;
pub mod state;

use instructions::*;

declare_id!("GnS9Sii7PpELXQLyKwZRgrEpqma3GQwcSxtqNdCMmkk3");

/// Whether a cover-type ordinal settles against a price oracle (Depeg=1, Oracle=4).
pub fn cover_type_is_price_based(ordinal: u8) -> bool {
    ordinal == 1 || ordinal == 4
}

/// The Berm claim-resolver program.
#[program]
pub mod berm_claim_resolver {
    use super::*;

    /// Initialise governance parameters and the authorised keeper.
    pub fn init_governance(
        ctx: Context<InitGovernance>,
        eligible_weight: u64,
        quorum_bps: u64,
        approval_threshold_bps: u64,
        voting_slots: u64,
    ) -> Result<()> {
        instructions::claim::init_governance(
            ctx,
            eligible_weight,
            quorum_bps,
            approval_threshold_bps,
            voting_slots,
        )
    }

    /// Open a claim against a policy.
    pub fn open_claim(
        ctx: Context<OpenClaim>,
        policy: Pubkey,
        cover_type: u8,
        coverage: u64,
    ) -> Result<()> {
        instructions::claim::open_claim(ctx, policy, cover_type, coverage)
    }

    /// Auto-resolve a depeg claim from a Switchboard feed.
    pub fn resolve_auto_depeg(
        ctx: Context<ResolveDepeg>,
        lower: i64,
        upper: i64,
        expo: i32,
    ) -> Result<()> {
        instructions::claim::resolve_auto_depeg(ctx, lower, upper, expo)
    }

    /// Auto-resolve an oracle-divergence claim from two Switchboard feeds.
    pub fn resolve_auto_oracle(
        ctx: Context<ResolveOracle>,
        expo: i32,
        threshold_bps: u64,
    ) -> Result<()> {
        instructions::claim::resolve_auto_oracle(ctx, expo, threshold_bps)
    }

    /// Approve an off-chain cover claim via a signed keeper attestation.
    pub fn submit_attestation(ctx: Context<SubmitAttestation>, trigger_bps: u64) -> Result<()> {
        instructions::claim::submit_attestation(ctx, trigger_bps)
    }

    /// Escalate a claim to a governance dispute.
    pub fn open_dispute(ctx: Context<OpenDispute>) -> Result<()> {
        instructions::dispute::open_dispute(ctx)
    }

    /// Cast a token-weighted ballot on a dispute.
    pub fn cast_vote(ctx: Context<CastVote>, approve_vote: bool, weight: u64) -> Result<()> {
        instructions::dispute::cast_vote(ctx, approve_vote, weight)
    }

    /// Finalise a dispute after its voting window closes.
    pub fn finalize_dispute(ctx: Context<FinalizeDispute>) -> Result<()> {
        instructions::dispute::finalize_dispute(ctx)
    }

    /// Pay out an approved claim from the settlement vault.
    pub fn execute_payout(ctx: Context<ExecutePayout>) -> Result<()> {
        instructions::settle::execute_payout(ctx)
    }

    /// Slash a wrong-assessment underwriter's stake to the treasury.
    pub fn slash_underwriter(
        ctx: Context<SlashUnderwriter>,
        stake: u64,
        slash_bps: u64,
    ) -> Result<()> {
        instructions::settle::slash_underwriter(ctx, stake, slash_bps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_based_ordinals() {
        assert!(cover_type_is_price_based(1)); // Depeg
        assert!(cover_type_is_price_based(4)); // Oracle
        assert!(!cover_type_is_price_based(0)); // Exploit
        assert!(!cover_type_is_price_based(2)); // Slashing
    }
}

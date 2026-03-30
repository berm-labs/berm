//! # berm-cover
//!
//! The Berm cover executor -- the marketplace and policy-issuance program. It
//! deploys to Solana devnet; mainnet promotion requires separate authorisation.
//!
//! It manages the lifecycle that turns pooled LP capital into purchasable
//! parametric cover:
//!
//! - a singleton [`state::Protocol`] config with a fee and treasury;
//! - [`state::CoverProduct`]s, one per `(cover_type, subject)`, carrying the
//!   risk parameters used to price premiums;
//! - [`state::Underwriter`]s who stake and score protocol risk;
//! - [`state::CoverPolicy`]s issued to buyers, priced on chain via [`math`] so
//!   the premium can never be spoofed by the client.
//!
//! Price-based cover (Depeg, Oracle) reads both Pyth and Switchboard On-Demand
//! through [`oracle`], implementing the dual-oracle divergence guard. Settlement
//! and claim adjudication live in the sibling `berm-claim-resolver` program; the
//! capital itself is custodied by `berm-pool-vault`.
//!
//! Cover-type ordinals match `berm_cover_engine::CoverType` so off-chain keeper
//! triggers line up with on-chain policies.

#![allow(unexpected_cfgs)]
// Anchor 0.31's account-init macro expansion calls `AccountInfo::realloc`, which
// solana-program 2.x has deprecated in favour of `resize`. The call lives in
// generated code, not ours, so the deprecation is allowed at the crate root.
#![allow(deprecated)]

use anchor_lang::prelude::*;

pub mod cover_type;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod oracle;
pub mod state;

use instructions::*;

declare_id!("AMenBCW8sgtx2VriEYzdJkTCsUBF6FGQy8PhcNh9p7pH");

/// The Berm cover executor program.
#[program]
pub mod berm_cover {
    use super::*;

    /// Initialise the singleton protocol config.
    pub fn initialize_protocol(ctx: Context<InitializeProtocol>, fee_bps: u16) -> Result<()> {
        instructions::initialize::handler(ctx, fee_bps)
    }

    /// Create a cover product for a `(cover_type, subject)` pair.
    pub fn create_product(ctx: Context<CreateProduct>, args: CreateProductArgs) -> Result<()> {
        instructions::product::create_product(ctx, args)
    }

    /// Record an underwriter's safety score for a product.
    pub fn assess_risk(ctx: Context<AssessRisk>, safety_score: u64) -> Result<()> {
        instructions::product::assess_risk(ctx, safety_score)
    }

    /// Purchase cover: price the premium on chain, collect it, issue a policy.
    pub fn buy_cover(ctx: Context<BuyCover>, coverage: u64, duration_days: u32) -> Result<()> {
        instructions::buy_cover::handler(ctx, coverage, duration_days)
    }

    /// Register a new underwriter record.
    pub fn register_underwriter(ctx: Context<RegisterUnderwriter>) -> Result<()> {
        instructions::underwriter::register_underwriter(ctx)
    }

    /// Stake $BERM to back an underwriter's assessments.
    pub fn stake_underwriter(ctx: Context<StakeUnderwriter>, amount: u64) -> Result<()> {
        instructions::underwriter::stake_underwriter(ctx, amount)
    }
}

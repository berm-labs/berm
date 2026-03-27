//! Cover purchase: price the premium on chain, collect it, and issue a policy.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::CoverError;
use crate::events::CoverPurchased;
use crate::instructions::initialize::PROTOCOL_SEED;
use crate::math::{annual_rate_bps, premium};
use crate::state::{CoverPolicy, CoverProduct, PolicyStatus, Protocol};

/// PDA seed for a policy.
pub const POLICY_SEED: &[u8] = b"policy";

/// Seconds per day, for duration -> timestamp conversion.
pub const SECS_PER_DAY: i64 = 86_400;

/// Accounts for [`handler`].
#[derive(Accounts)]
pub struct BuyCover<'info> {
    /// Buyer (becomes the policy holder).
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Protocol account.
    #[account(mut, seeds = [PROTOCOL_SEED], bump = protocol.bump)]
    pub protocol: Account<'info, Protocol>,

    /// Product being purchased.
    #[account(mut)]
    pub product: Account<'info, CoverProduct>,

    /// New policy PDA, keyed by (product, protocol.total_policies).
    #[account(
        init,
        payer = buyer,
        space = 8 + CoverPolicy::INIT_SPACE,
        seeds = [POLICY_SEED, product.key().as_ref(), &protocol.total_policies.to_le_bytes()],
        bump
    )]
    pub policy: Account<'info, CoverPolicy>,

    /// Premium asset mint (Token-2022).
    pub premium_mint: InterfaceAccount<'info, Mint>,

    /// Buyer's premium source account.
    #[account(mut, token::mint = premium_mint, token::authority = buyer)]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,

    /// Treasury token account that receives the premium.
    #[account(mut, token::mint = premium_mint)]
    pub treasury_ata: InterfaceAccount<'info, TokenAccount>,

    /// SPL Token-2022 program.
    pub token_program: Interface<'info, TokenInterface>,
    /// System program.
    pub system_program: Program<'info, System>,
}

/// Buy `coverage` of cover for `duration_days`. The premium is recomputed on
/// chain from the product's risk parameters and collected before the policy is
/// issued, so the price cannot be spoofed by the client.
pub fn handler(ctx: Context<BuyCover>, coverage: u64, duration_days: u32) -> Result<()> {
    require!(coverage > 0, CoverError::ZeroAmount);
    let product = &ctx.accounts.product;
    require!(product.active, CoverError::ProductInactive);
    require!(
        duration_days >= product.min_duration_days && duration_days <= product.max_duration_days,
        CoverError::DurationOutOfRange
    );
    require!(
        coverage <= product.remaining_capacity(),
        CoverError::CapacityExceeded
    );

    // Price the premium at the utilisation *including* this policy.
    let rate = annual_rate_bps(
        product.base_rate_bps,
        product.safety_score,
        product.committed + coverage,
        product.capacity,
    );
    let owed = premium(coverage, rate, duration_days as u64).ok_or(CoverError::Overflow)?;

    // Collect the premium with a decimal-checked Token-2022 transfer.
    let decimals = ctx.accounts.premium_mint.decimals;
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.buyer_ata.to_account_info(),
                mint: ctx.accounts.premium_mint.to_account_info(),
                to: ctx.accounts.treasury_ata.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        owed,
        decimals,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let index = ctx.accounts.protocol.total_policies;

    let policy = &mut ctx.accounts.policy;
    policy.product = product.key();
    policy.holder = ctx.accounts.buyer.key();
    policy.coverage = coverage;
    policy.premium_paid = owed;
    policy.start_ts = now;
    policy.end_ts = now + duration_days as i64 * SECS_PER_DAY;
    policy.status = PolicyStatus::Active;
    policy.index = index;
    policy.bump = ctx.bumps.policy;

    let product = &mut ctx.accounts.product;
    product.committed = product.committed.checked_add(coverage).ok_or(CoverError::Overflow)?;

    let protocol = &mut ctx.accounts.protocol;
    protocol.total_policies = protocol.total_policies.checked_add(1).ok_or(CoverError::Overflow)?;

    emit!(CoverPurchased {
        policy: policy.key(),
        product: policy.product,
        holder: policy.holder,
        coverage,
        premium: owed,
    });
    Ok(())
}

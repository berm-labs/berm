//! Cover-product creation and underwriter risk assessment.

use anchor_lang::prelude::*;

use crate::cover_type::CoverType;
use crate::error::CoverError;
use crate::events::{ProductCreated, RiskAssessed};
use crate::instructions::initialize::PROTOCOL_SEED;
use crate::math::RISK_MAX;
use crate::state::{CoverProduct, Protocol, Underwriter};

/// PDA seed for a cover product.
pub const PRODUCT_SEED: &[u8] = b"product";

/// Arguments for creating a product.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateProductArgs {
    /// Cover-type ordinal.
    pub cover_type: u8,
    /// Subject identifier (<= 48 bytes).
    pub subject: String,
    /// Category base rate (bps).
    pub base_rate_bps: u64,
    /// Sellable capacity (asset units).
    pub capacity: u64,
    /// Minimum policy duration (days).
    pub min_duration_days: u32,
    /// Maximum policy duration (days).
    pub max_duration_days: u32,
}

/// Accounts for [`create_product`].
#[derive(Accounts)]
#[instruction(args: CreateProductArgs)]
pub struct CreateProduct<'info> {
    /// Protocol authority.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Protocol account.
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol.bump,
        has_one = authority @ CoverError::Unauthorized,
    )]
    pub protocol: Account<'info, Protocol>,

    /// New product PDA, keyed by (cover_type, subject).
    #[account(
        init,
        payer = authority,
        space = 8 + CoverProduct::INIT_SPACE,
        seeds = [PRODUCT_SEED, &[args.cover_type], args.subject.as_bytes()],
        bump
    )]
    pub product: Account<'info, CoverProduct>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Create a cover product.
pub fn create_product(ctx: Context<CreateProduct>, args: CreateProductArgs) -> Result<()> {
    require!(
        CoverType::from_ordinal(args.cover_type).is_some(),
        CoverError::Overflow
    );
    require!(args.subject.len() <= 48, CoverError::Overflow);
    require!(args.capacity > 0, CoverError::ZeroAmount);
    require!(
        args.min_duration_days > 0 && args.min_duration_days <= args.max_duration_days,
        CoverError::DurationOutOfRange
    );

    let product = &mut ctx.accounts.product;
    product.protocol = ctx.accounts.protocol.key();
    product.cover_type = args.cover_type;
    product.subject = args.subject.clone();
    product.base_rate_bps = args.base_rate_bps;
    product.safety_score = 500; // neutral until an underwriter scores it
    product.capacity = args.capacity;
    product.committed = 0;
    product.min_duration_days = args.min_duration_days;
    product.max_duration_days = args.max_duration_days;
    product.active = true;
    product.bump = ctx.bumps.product;

    let protocol = &mut ctx.accounts.protocol;
    protocol.total_products = protocol.total_products.checked_add(1).ok_or(CoverError::Overflow)?;

    emit!(ProductCreated {
        product: product.key(),
        cover_type: args.cover_type,
        subject: args.subject,
        capacity: args.capacity,
    });
    Ok(())
}

/// Accounts for [`assess_risk`].
#[derive(Accounts)]
pub struct AssessRisk<'info> {
    /// Underwriter authority.
    pub authority: Signer<'info>,

    /// The underwriter record (must be staked and not slashed).
    #[account(
        has_one = authority @ CoverError::Unauthorized,
        constraint = !underwriter.slashed @ CoverError::Unauthorized,
    )]
    pub underwriter: Account<'info, Underwriter>,

    /// Product being scored.
    #[account(mut)]
    pub product: Account<'info, CoverProduct>,
}

/// Record an underwriter's safety score (0..=1000) for a product.
pub fn assess_risk(ctx: Context<AssessRisk>, safety_score: u64) -> Result<()> {
    require!(safety_score <= RISK_MAX, CoverError::RiskScoreOutOfRange);
    let product = &mut ctx.accounts.product;
    product.safety_score = safety_score;

    let underwriter = &mut ctx.accounts.underwriter;
    underwriter.products_assessed = underwriter
        .products_assessed
        .checked_add(1)
        .ok_or(CoverError::Overflow)?;

    emit!(RiskAssessed {
        product: product.key(),
        underwriter: underwriter.key(),
        safety_score,
    });
    Ok(())
}

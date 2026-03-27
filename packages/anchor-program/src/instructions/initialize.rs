//! Protocol initialisation.

use anchor_lang::prelude::*;

use crate::error::CoverError;
use crate::events::ProtocolInitialized;
use crate::state::Protocol;

/// PDA seed for the singleton protocol account.
pub const PROTOCOL_SEED: &[u8] = b"protocol";

/// Accounts for [`handler`].
#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    /// Governance authority and payer.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: treasury that receives protocol fees; validated only as a pubkey.
    pub treasury: UncheckedAccount<'info>,

    /// The singleton protocol account.
    #[account(
        init,
        payer = authority,
        space = 8 + Protocol::INIT_SPACE,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol: Account<'info, Protocol>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Initialise the protocol with a fee (bps, capped at 10%).
pub fn handler(ctx: Context<InitializeProtocol>, fee_bps: u16) -> Result<()> {
    require!(fee_bps <= 1_000, CoverError::Overflow);
    let protocol = &mut ctx.accounts.protocol;
    protocol.authority = ctx.accounts.authority.key();
    protocol.treasury = ctx.accounts.treasury.key();
    protocol.fee_bps = fee_bps;
    protocol.total_products = 0;
    protocol.total_policies = 0;
    protocol.paused = false;
    protocol.bump = ctx.bumps.protocol;

    emit!(ProtocolInitialized {
        protocol: protocol.key(),
        authority: protocol.authority,
    });
    Ok(())
}

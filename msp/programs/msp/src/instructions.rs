use anchor_lang::prelude::*;
use anchor_spl::token::*;

use crate::errors::*;
use crate::treasury::*;
use crate::stream::*;
use crate::fee_treasury;
use crate::msp;

#[derive(Accounts, Clone)]
#[
    instruction(
        name: String,
        start_utc: u64,   
        rate_amount_units: u64,
        rate_interval_in_seconds: u64,
        allocation_assigned_units: u64,
        allocation_reserved_units: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount_units: u64,
        cliff_vest_percent: f64
    )
]
pub struct CreateStream<'info> {
    pub treasurer: Signer<'info>,
    #[account(
        seeds = [
            treasurer.key().as_ref(),
            &treasury.slot.to_le_bytes()
        ],
        bump = treasury.bump
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        constraint = associated_token.key() == treasury.associated_token_address @ ErrorCode::InvalidAssociatedToken
    )]
    pub associated_token: Account<'info, Mint>,
    #[account()]
    pub beneficiary: SystemAccount<'info>,
    #[account(
        init,
        payer = treasurer,
        space = 500, // TBD,
        constraint = stream.initialized == false,
        constraint = stream.version == 2
    )]
    pub stream: ProgramAccount<'info, StreamV2>,
    #[account(
        mut, 
        constraint = fee_tresury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_tresury: SystemAccount<'info>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
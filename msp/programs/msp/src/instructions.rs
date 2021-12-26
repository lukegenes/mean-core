use anchor_lang::prelude::*;
use anchor_spl::token::*;
use anchor_spl::associated_token::*;

use crate::constants::*;
use crate::errors::*;
use crate::treasury::*;
use crate::stream::*;
use crate::fee_treasury;
use crate::msp;

// Create Treasury
#[derive(Accounts, Clone)]
#[instruction(
    slot: u64,
    bump: u8,
    label: String,
    treasury_type: u8,
    auto_close: bool
)]
pub struct CreateTreasury<'info> {
    pub treasurer: Signer<'info>,
    #[account(
        init,
        payer = treasurer,
        seeds = [treasurer.key().as_ref(), &slot.to_le_bytes()],
        bump = bump,
        space = 300, // TBD
        constraint = treasury.initialized == false @ ErrorCode::TreasuryAlreadyInitialized,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        init,
        payer = treasurer,
        seeds = [treasurer.key().as_ref(), treasury.key().as_ref(), &slot.to_le_bytes()],
        bump = bump,
        space = Mint::LEN,
        constraint = treasury_mint.decimals == TREASURY_POOL_MINT_DECIMALS @ ErrorCode::InvalidTreasuryMintDecimals
    )]
    pub treasury_mint: ProgramAccount<'info, Mint>,
    #[account(
        mut, 
        constraint = fee_tresury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_tresury: SystemAccount<'info>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Create Stream
#[derive(Accounts, Clone)]
#[instruction(
    name: String,
    start_utc: u64,   
    rate_amount_units: u64,
    rate_interval_in_seconds: u64,
    allocation_assigned_units: u64,
    allocation_reserved_units: u64,
    rate_cliff_in_seconds: u64,
    cliff_vest_amount_units: u64,
    cliff_vest_percent: f64
)]
pub struct CreateStream<'info> {
    pub treasurer: Signer<'info>,
    #[account(
        seeds = [treasurer.key().as_ref(), &treasury.slot.to_le_bytes()],
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
        constraint = stream.initialized == false @ ErrorCode::StreamAlreadyInitialized,
        constraint = stream.version == 2 @ ErrorCode::InvalidStreamVersion
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

// Add Funds
#[derive(Accounts)]
#[instruction(
    amount: u64,
    allocation_type: u8,
    allocation_stream: Option<Pubkey>
)]
pub struct AddFunds<'info> {
    pub contributor: Signer<'info>,
    #[account(
        mut,
        constraint = contributor_token.owner == contributor.key() @ ErrorCode::InvalidOwner,
        constraint = (
            contributor_token.mint == treasury.associated_token_address &&
            contributor_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub contributor_token: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = contributor,
        token::mint = treasury_mint,
        token::authority = contributor
    )]
    pub contributor_treasury_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryAlreadyInitialized
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        mut,
        constraint = treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner,
        constraint = treasury_token.mint == treasury.associated_token_address @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasury_token: Account<'info, TokenAccount>,
    #[account(
        constraint = (
            associated_token.key() == stream.beneficiary_associated_token &&
            associated_token.key() == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken,
    )]
    pub associated_token: Account<'info, Mint>,
    #[account(
        constraint = treasury_mint.decimals == TREASURY_POOL_MINT_DECIMALS @ ErrorCode::InvalidTreasuryMintDecimals,
        constraint = treasury_mint.key() == treasury.mint_address @ ErrorCode::InvalidTreasuryMint
    )]
    pub treasury_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint = stream.to_account_info().data_len() == 500 @ ErrorCode::InvalidStreamSize,
        constraint = stream.treasury_address == treasury.key() @ ErrorCode::InvalidTreasury,
        constraint = stream.beneficiary_associated_token == associated_token.key() @ ErrorCode::InvalidAssociatedToken
    )]
    pub stream: ProgramAccount<'info, StreamV2>,
    #[account(
        mut, 
        constraint = fee_tresury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_tresury: SystemAccount<'info>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}


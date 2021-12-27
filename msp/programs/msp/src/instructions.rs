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
    name: String,
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
    pub treasury_mint: Account<'info, Mint>,
    #[account(
        mut, 
        constraint = fee_treasury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_treasury: SystemAccount<'info>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Create Stream
#[derive(Accounts, Clone)]
#[instruction(
    treasury_bump: u8,
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
        bump = treasury_bump
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
        constraint = (
            contributor_token.mint == treasury.associated_token_address &&
            contributor_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub contributor_token: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = contributor,
        token::mint = treasury_mint,
        token::authority = contributor
    )]
    pub contributor_treasury_token: Box<Account<'info, TokenAccount>>,
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
    pub treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = (
            associated_token.key() == stream.beneficiary_associated_token &&
            associated_token.key() == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken,
    )]
    pub associated_token: Box<Account<'info, Mint>>,
    #[account(
        constraint = treasury_mint.decimals == TREASURY_POOL_MINT_DECIMALS @ ErrorCode::InvalidTreasuryMintDecimals,
        constraint = treasury_mint.key() == treasury.mint_address @ ErrorCode::InvalidTreasuryMint
    )]
    pub treasury_mint: Box<Account<'info, Mint>>,
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

// Withdraw
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Withdraw<'info> {
    pub beneficiary: Signer<'info>,
    #[account(
        mut,
        constraint = beneficiary_token.owner == beneficiary.key() @ ErrorCode::InvalidOwner,
        constraint = (
            beneficiary_token.mint == associated_token.key() &&
            beneficiary_token.mint == stream.beneficiary_associated_token &&
            beneficiary_token.mint == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub beneficiary_token: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = (
            associated_token.key() == treasury.associated_token_address &&
            associated_token.key() == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidTreasuryMint
    )]
    pub associated_token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = treasury.key() == stream.treasurer_address @ ErrorCode::InvalidTreasury,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryAlreadyInitialized,
        constraint = treasury.to_account_info().data_len() == 300 @ ErrorCode::InvalidTreasurySize,
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        mut,
        constraint = treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            treasury_token.mint == associated_token.key() &&
            treasury_token.mint == treasury.associated_token_address &&
            treasury_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = stream.treasury_address == treasury.key() @ ErrorCode::InvalidTreasury,
        constraint = stream.beneficiary_address == beneficiary.key() @ ErrorCode::InvalidBeneficiary,
        constraint = stream.beneficiary_associated_token == associated_token.key() @ ErrorCode::InvalidAssociatedToken,
        constraint = stream.to_account_info().data_len() == 500 @ ErrorCode::InvalidStreamSize,
    )]
    pub stream: ProgramAccount<'info, StreamV2>,
    #[account(
        mut, 
        constraint = fee_treasury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_treasury: SystemAccount<'info>,
    #[account(
        mut,
        constraint = fee_treasury_token.owner == fee_treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            fee_treasury_token.mint == associated_token.key() &&
            fee_treasury_token.mint == treasury.associated_token_address &&
            fee_treasury_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub fee_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Pause or Resume Stream
#[derive(Accounts)]
pub struct PauseOrResumeStream<'info> {
    #[account(
        constraint = (
            initializer.key() == stream.treasurer_address || 
            initializer.key() == stream.beneficiary_address
        ) @ ErrorCode::NotAuthorized
    )]
    pub initializer: Signer<'info>,
    #[account(
        mut,
        constraint = treasury.key() == stream.treasury_address @ ErrorCode::InvalidTreasury,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryNotInitialized
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        constraint = (
            associated_token.key() == stream.beneficiary_associated_token &&
            associated_token.key() == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken,
    )]
    pub associated_token: Account<'info, Mint>,
    #[account(
        mut,
        constraint = stream.to_account_info().data_len() == 500 @ ErrorCode::InvalidStreamSize,
        constraint = stream.treasury_address == treasury.key() @ ErrorCode::InvalidTreasury,
        constraint = stream.beneficiary_associated_token == associated_token.key() @ ErrorCode::InvalidAssociatedToken,
        constraint = (
            stream.treasurer_address == initializer.key() || 
            stream.beneficiary_address == initializer.key()
        ) @ ErrorCode::NotAuthorized
    )]
    pub stream: ProgramAccount<'info, StreamV2>,
    #[account(
        mut, 
        constraint = fee_tresury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_tresury: SystemAccount<'info>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
}

// Close Stream
#[derive(Accounts)]
#[instruction(auto_close_treasury: bool)]
pub struct CloseStream<'info> {
    #[account(
        constraint = ((
            initializer.key() == stream.treasurer_address && initializer.key() == treasury.treasurer_address) || 
            initializer.key() == stream.beneficiary_address
        ) @ ErrorCode::NotAuthorized
    )]
    pub initializer: Signer<'info>,
    #[account(
        constraint = (
            treasurer.key() == stream.treasurer_address &&
            treasurer.key() == treasury.treasurer_address
        ) @ ErrorCode::InvalidTreasurer
    )]
    pub treasurer: SystemAccount<'info>,
    #[account(
        mut,
        constraint = (
            treasurer_token.mint == treasury.associated_token_address &&
            treasurer_token.mint == stream.beneficiary_associated_token &&
            treasurer_token.mint == associated_token.key()
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasurer_token: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = treasurer_treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner
    )]
    pub treasurer_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryAlreadyInitialized
    )]
    pub beneficiary: SystemAccount<'info>,
    #[account(
        mut,
        constraint = beneficiary_token.owner == beneficiary.key() @ ErrorCode::InvalidOwner,
        constraint = (
            beneficiary_token.mint == associated_token.key() &&
            beneficiary_token.mint == stream.beneficiary_associated_token &&
            beneficiary_token.mint == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub beneficiary_token: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = (
            associated_token.key() == stream.beneficiary_associated_token &&
            associated_token.key() == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken,
    )]
    pub associated_token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = treasury.key() == stream.treasury_address @ ErrorCode::InvalidTreasury,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryNotInitialized
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        mut,
        constraint = treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            treasury_token.mint == associated_token.key() &&
            treasury_token.mint == treasury.associated_token_address &&
            treasury_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = treasury_mint.decimals == TREASURY_POOL_MINT_DECIMALS @ ErrorCode::InvalidTreasuryMintDecimals,
        constraint = treasury_mint.key() == treasury.mint_address @ ErrorCode::InvalidTreasuryMint
    )]
    pub treasury_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = stream.treasury_address == treasury.key() @ ErrorCode::InvalidTreasury,
        constraint = stream.beneficiary_address == beneficiary.key() @ ErrorCode::InvalidBeneficiary,
        constraint = stream.beneficiary_associated_token == associated_token.key() @ ErrorCode::InvalidAssociatedToken,
        constraint = stream.to_account_info().data_len() == 500 @ ErrorCode::InvalidStreamSize,
    )]
    pub stream: ProgramAccount<'info, StreamV2>,
    #[account(
        mut, 
        constraint = fee_treasury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_treasury: SystemAccount<'info>,
    #[account(
        mut,
        constraint = fee_treasury_token.owner == fee_treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            fee_treasury_token.mint == associated_token.key() &&
            fee_treasury_token.mint == treasury.associated_token_address &&
            fee_treasury_token.mint == stream.beneficiary_associated_token
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub fee_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// Close Treasury
#[derive(Accounts)]
pub struct CloseTreasury<'info> {
    #[account(constraint = treasurer.key() == treasury.treasurer_address @ ErrorCode::InvalidTreasurer)]
    pub treasurer: SystemAccount<'info>,
    #[account(
        mut,
        constraint = (
            treasurer_token.mint == treasury.associated_token_address &&
            treasurer_token.mint == associated_token.key()
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasurer_token: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = treasurer_treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner
    )]
    pub treasurer_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(constraint = associated_token.key() == treasury.associated_token_address @ ErrorCode::InvalidAssociatedToken)]
    pub associated_token: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = treasurer.key() == treasury.treasurer_address @ ErrorCode::NotAuthorized,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryNotInitialized
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        mut,
        constraint = treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            treasury_token.mint == associated_token.key() &&
            treasury_token.mint == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = treasury_mint.decimals == TREASURY_POOL_MINT_DECIMALS @ ErrorCode::InvalidTreasuryMintDecimals,
        constraint = treasury_mint.key() == treasury.mint_address @ ErrorCode::InvalidTreasuryMint
    )]
    pub treasury_mint: Account<'info, Mint>,
    #[account(
        mut, 
        constraint = fee_treasury.key() == fee_treasury::ID @ ErrorCode::InvalidFeeTreasuryAccount
    )]
    pub fee_treasury: SystemAccount<'info>,
    #[account(
        mut,
        constraint = fee_treasury_token.owner == fee_treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            fee_treasury_token.mint == associated_token.key() &&
            fee_treasury_token.mint == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub fee_treasury_token: Box<Account<'info, TokenAccount>>,
    #[account(constraint = msp.key() == msp::ID @ ErrorCode::InvalidProgramId)]
    pub msp: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RefreshTreasuryBalance<'info> {
    #[account(constraint = treasurer.key() == treasury.treasurer_address @ ErrorCode::InvalidTreasurer)]
    pub treasurer: Signer<'info>,
    #[account(constraint = associated_token.key() == treasury.associated_token_address @ ErrorCode::InvalidAssociatedToken)]
    pub associated_token: Account<'info, Mint>,
    #[account(
        mut,
        constraint = treasurer.key() == treasury.treasurer_address @ ErrorCode::NotAuthorized,
        constraint = treasury.version == 2 @ ErrorCode::InvalidTreasuryVersion,
        constraint = treasury.initialized == true @ ErrorCode::TreasuryNotInitialized
    )]
    pub treasury: ProgramAccount<'info, TreasuryV2>,
    #[account(
        mut,
        constraint = treasury_token.owner == treasury.key() @ ErrorCode::InvalidOwner,
        constraint = (
            treasury_token.mint == associated_token.key() &&
            treasury_token.mint == treasury.associated_token_address
        ) @ ErrorCode::InvalidAssociatedToken
    )]
    pub treasury_token: Account<'info, TokenAccount>
}
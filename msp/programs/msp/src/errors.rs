use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("Invalid Money Streaming Program ID")]
    InvalidProgramId,
    #[msg("Invalid account owner")]
    InvalidOwner,
    #[msg("Not Authorized")]
    NotAuthorized,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid associated token address")]
    InvalidAssociatedToken,
    #[msg("Invalid fee treasury account")]
    InvalidFeeTreasuryAccount,
    #[msg("Invalid treasury mint decimals")]
    InvalidTreasuryMintDecimals,
    #[msg("Stream is already initialized")]
    StreamAlreadyInitialized,
    #[msg("Treasury is already initialized")]
    TreasuryAlreadyInitialized,
    #[msg("Treasury is not initialized")]
    TreasuryNotInitialized,
    #[msg("Invalid stream version")]
    InvalidStreamVersion,
    #[msg("Invalid treasury version")]
    InvalidTreasuryVersion,
    #[msg("Invalid treasury mint address")]
    InvalidTreasuryMint,
    #[msg("Invalid treasury account")]
    InvalidTreasury,
    #[msg("Invalid stream size")]
    InvalidStreamSize,
    #[msg("Invalid treasury size")]
    InvalidTreasurySize,
    #[msg("Invalid treasurer")]
    InvalidTreasurer,
    #[msg("Invalid beneficiary")]
    InvalidBeneficiary,
}
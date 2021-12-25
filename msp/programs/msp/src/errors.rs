use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("Invalid Money Streaming Program ID")]
    InvalidProgramId,
    #[msg("Not Authorized")]
    NotAuthorized,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid associated token address")]
    InvalidAssociatedToken,
    #[msg("Invalid fee treasury account")]
    InvalidFeeTreasuryAccount
}
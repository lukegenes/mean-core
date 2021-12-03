// Program specific errors

use solana_program::{
    msg,
    decode_error::DecodeError,
    program_error::{ ProgramError, PrintProgramError } 
};

use num_derive::FromPrimitive;
use thiserror::Error;

/// Stream errors
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum StreamError {

    #[error("Invalid streaming program id")]
    IncorrectProgramId,

    #[error("Invalid instruction for the streaming program")]
    InvalidStreamInstruction,

    #[error("Stream account is already initialized")]
    StreamAlreadyInitialized,

    #[error("Stream terms account is already initialized")]
    StreamTermsAlreadyInitialized,

    #[error("Invalid stream data")]
    InvalidStreamData,

    #[error("Invalid treasury account")]
    InvalidTreasuryAccount,

    #[error("Invalid treasury mint")]
    InvalidTreasuryPoolMint,

    #[error("Invalid treasury token")]
    InvalidTreasuryToken,    

    #[error("Invalid treasury data")]
    InvalidTreasuryData,

    #[error("Invalid treasury pool")]
    InvalidTreasuryPool,    

    #[error("Invalid treasury pool ATA")]
    InvalidTreasuryPoolAddress,

    #[error("Invalid MSP Operations token")]
    InvalidMspOpsToken,   

    #[error("Instruction signature is missing")]
    MissingInstructionSignature,

    #[error("Account balance below rent-exempt threshold")]
    InvalidRentException,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Not authorized to perform this action")]
    InstructionNotAuthorized,

    #[error("Invalid argument")]
    InvalidArgument,

    #[error("NotAllowedRecoverableAmount")]
    NotAllowedRecoverableAmount,

    #[error("NotAllowedWithdrawalAmount")]
    NotAllowedWithdrawalAmount,

    #[error("NotAuthorizedToWithdraw")]
    NotAuthorizedToWithdraw,

    #[error("InvalidWithdrawalDate")]
    InvalidWithdrawalDate,

    #[error("InvalidSignerAuthority")]
    InvalidSignerAuthority,

    #[error("Overflow")]
    Overflow,
    //
    #[error("InvalidPdaAccount")]
    InvalidPdaAccount,
    
    #[error("AvailableTreasuryReserveExceeded")]
    AvailableTreasuryReserveExceeded,

    #[error("InvalidTreasuryAssociatedToken")]
    InvalidTreasuryAssociatedToken,

    #[error("CloseTreasuryWithStreams")]
    CloseTreasuryWithStreams,
}

impl From<StreamError> for ProgramError {
    fn from(e: StreamError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<E> DecodeError<E> for StreamError {
    fn type_of() -> &'static str {
        "StreamError"
    }
}

impl PrintProgramError for StreamError {
    fn print<E>(&self) where E: 'static + std::error::Error + DecodeError<E> + PrintProgramError {
        match self {
            Self::IncorrectProgramId => msg!("Error: IncorrectProgramId"),
            Self::InvalidStreamInstruction => msg!("Error: InvalidStreamInstruction"),
            Self::StreamAlreadyInitialized => msg!("Error: StreamAlreadyInitialized"),
            Self::StreamTermsAlreadyInitialized => msg!("Error: StreamTermsAlreadyInitialized"),
            Self::InvalidStreamData => msg!("Error: InvalidStreamData"),
            Self::InvalidTreasuryAccount => msg!("Error: Treasury associated token address does not match seed derivation"),
            Self::InvalidTreasuryPoolMint => msg!("Error: Treasury pool mint address does not match seed derivation"),
            Self::InvalidTreasuryToken => msg!("Error: Treasury associated token address does not match seed derivation"),
            Self::InvalidTreasuryData => msg!("Error: Treasury data in not valid"),
            Self::InvalidMspOpsToken => msg!("Error: MSP Operations associated token address does not match seed derivation"),
            Self::InvalidTreasuryPool => msg!("Error: Treasury pool address does not match seed derivation"),
            Self::InvalidTreasuryPoolAddress => msg!("Error: Treasury ATA does not match seed derivation"),
            Self::MissingInstructionSignature => msg!("Error: MissingInstructionSignature"),
            Self::InvalidRentException => msg!("Error: Account balance below rent-exempt threshold"),
            Self::InsufficientFunds => msg!("Error: InsufficientFunds"),
            Self::InstructionNotAuthorized => msg!("Error: InstructionNotAuthorized"),
            Self::InvalidArgument => msg!("Error: InvalidArgument"),
            Self::NotAllowedRecoverableAmount => msg!("Error: Can not recover more that the unvested amount"),            
            Self::NotAllowedWithdrawalAmount => msg!("Error: Can not withdraw more that the vested amount"),
            Self::NotAuthorizedToWithdraw => msg!("Error: Not authorized to withdraw from the stream"),
            Self::InvalidWithdrawalDate => msg!("Error: The date to withdraw your money has not been reached yet"),
            Self::InvalidSignerAuthority => msg!("Error: InvalidSignerAuthority"),
            Self::Overflow => msg!("Error: Overflow"),
            Self::InvalidPdaAccount => msg!("Error: PDA account doesn't match the seed derivation"),
            Self::AvailableTreasuryReserveExceeded => msg!("Error: The amount to reserve in not available"),
            Self::InvalidTreasuryAssociatedToken => msg!("Error: The associated token is not valid for the treasury"),
            Self::CloseTreasuryWithStreams => msg!("Error: Treasuries with active streams can not be closed"),
        }
    }
}

/// Treasury errors
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TreasuryError {

    #[error("Invalid treasury data")]
    InvalidTreasuryData
}

impl From<TreasuryError> for ProgramError {
    fn from(e: TreasuryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<E> DecodeError<E> for TreasuryError {
    fn type_of() -> &'static str {
        "TreasuryError"
    }
}

impl PrintProgramError for TreasuryError {
    fn print<E>(&self) where E: 'static + std::error::Error + DecodeError<E> + PrintProgramError {
        match self {
            Self::InvalidTreasuryData => msg!("Error: InvalidTreasuryData")
        }
    }
}
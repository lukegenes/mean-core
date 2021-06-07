// Program specific errors

use solana_program::{
    msg,
    decode_error::DecodeError,
    program_error::{ ProgramError, PrintProgramError } 
};

use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum StreamError {

    #[error("Invalid streaming program id")]
    IncorrectProgramId,

    #[error("Invalid instruction for the streaming program")]
    InvalidStreamInstruction,

    #[error("Stream account is already initialized")]
    StreamAlreadyInitialized,

    #[error("Invalid stream data")]
    InvalidStreamData,

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

    #[error("NotAllowedWithdrawalAmount")]
    NotAllowedWithdrawalAmount,

    #[error("NotAuthorizedToWithdraw")]
    NotAuthorizedToWithdraw,

    #[error("InvalidWithdrawalDate")]
    InvalidWithdrawalDate,

    #[error("InvalidSignerAuthority")]
    InvalidSignerAuthority
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
            Self::InvalidStreamData => msg!("Error: InvalidStreamData"),
            Self::MissingInstructionSignature => msg!("Error: MissingInstructionSignature"),
            Self::InvalidRentException => msg!("Error: Account balance below rent-exempt threshold"),
            Self::InsufficientFunds => msg!("Error: InsufficientFunds"),
            Self::InstructionNotAuthorized => msg!("Error: InstructionNotAuthorized"),
            Self::InvalidArgument => msg!("Error: InvalidArgument"),
            Self::NotAllowedWithdrawalAmount => msg!("Error: Can not withdraw more that the vested amount"),
            Self::NotAuthorizedToWithdraw => msg!("Error: Not authorized to withdraw from the stream"),
            Self::InvalidWithdrawalDate => msg!("Error: The date to withdraw your money has not been reached yet"),
            Self::InvalidSignerAuthority => msg!("Error: InvalidSignerAuthority")
        }
    }
}
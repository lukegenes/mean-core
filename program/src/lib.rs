// Register modules
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub use solana_program;

use solana_program::{
    declare_id,
    entrypoint::ProgramResult,
    pubkey::Pubkey
};

use crate::error::StreamError;

declare_id!("H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko");

pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(StreamError::IncorrectProgramId.into());
    }
    Ok(())
}
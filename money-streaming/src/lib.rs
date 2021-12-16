// Register modules
pub mod error;
pub mod utils;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod constants;
pub mod account_validations;
pub mod extensions;
pub mod backwards_comp;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub use solana_program;

use solana_program::{
    declare_id,
    entrypoint::ProgramResult,
    pubkey::Pubkey
};

use crate::error::StreamError;

declare_id!("9yMq7x4LstWYWi14pr8BEBsEX33L3HnugpiM2PT96x4k");

pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(StreamError::IncorrectProgramId.into());
    }
    Ok(())
}
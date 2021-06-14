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

declare_id!("37z61WhJCAaDADwcpJRHgr66FUhHB9TfkS49Ssvp3Cdb");

pub fn check_program_account(program_id: &Pubkey) -> ProgramResult {
    if program_id != &id() {
        return Err(StreamError::IncorrectProgramId.into());
    }
    Ok(())
}
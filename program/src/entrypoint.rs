// Entrypoint to the program

use solana_program::{
    entrypoint,
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::PrintProgramError
};

use crate::{ 
    processor::Processor,
    error::StreamError
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],

) -> ProgramResult {

    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        error.print::<StreamError>();
        return Err(error);
    }
    Ok(())
}
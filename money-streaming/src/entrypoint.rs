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
    error::StreamError,
    constants::FEE_TREASURY_ACCOUNT_ADDRESS
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]

) -> ProgramResult {

    let msp_ops_account = FEE_TREASURY_ACCOUNT_ADDRESS.parse().unwrap();
    
    if let Err(error) = verify_msp_ops_account(&msp_ops_account, accounts) {
        error.print::<StreamError>();
        return Err(error);
    }

    if let Err(error) = Processor::process(program_id, accounts, instruction_data) {
        error.print::<StreamError>();
        return Err(error);
    }

    Ok(())
}

fn verify_msp_ops_account(
    account_to_verify: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {

    let msp_account_valid = accounts.iter().any(|a| a.key.eq(account_to_verify));

    if !msp_account_valid {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    Ok(())
}
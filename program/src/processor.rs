// Program

use solana_program::{
    msg,
    system_program,
    system_instruction,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    instruction::{ AccountMeta, Instruction },
    account_info::{ next_account_info, AccountInfo },
    program_pack::{ IsInitialized, Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar }    
};

use spl_token::instruction;

use crate::{
    error::StreamError,
    instruction::StreamInstruction,
    state::{ Stream, StreamTerms, LAMPORTS_PER_SOL }
};

pub struct Processor {}

impl Processor {

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]

    ) -> ProgramResult {

        let instruction = StreamInstruction::unpack(instruction_data)?;

        match instruction {

            StreamInstruction::CreateStream {
                stream_name,
                treasury_address,
                stream_address,
                beneficiary_address,
                stream_associated_token,
                funding_amount,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent

            } => {

                msg!("Instruction: CreateStream");

                Self::process_create_stream(
                    accounts, 
                    program_id,
                    stream_name,
                    treasury_address,
                    stream_address,
                    beneficiary_address,
                    stream_associated_token,
                    funding_amount,
                    rate_amount,
                    rate_interval_in_seconds,
                    start_utc,
                    rate_cliff_in_seconds,
                    cliff_vest_amount,
                    cliff_vest_percent
                )
            },

            StreamInstruction::AddFunds {
                contribution_token_address,
                contribution_amount

            } => {

                msg!("Instruction: AddFunds");

                Self::process_add_funds(
                    accounts, 
                    program_id,
                    contribution_token_address,
                    contribution_amount
                )
            },

            StreamInstruction::Withdraw {
                withdrawal_amount
            } => {
                msg!("Instruction: Withdraw");
                
                Self::process_withdraw(
                    accounts, 
                    program_id, 
                    withdrawal_amount
                )
            },

            StreamInstruction::ProposeUpdate {
                proposed_by,
                stream_name,
                treasurer_address,
                treasury_address,
                beneficiary_address,
                stream_associated_token,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds

            } => {

                msg!("Instruction: ProposeUpdate");
                
                Self::process_propose_update(
                    accounts, 
                    program_id,
                    proposed_by,
                    stream_name,
                    treasurer_address,
                    treasury_address,
                    beneficiary_address,
                    stream_associated_token,
                    rate_amount,
                    rate_interval_in_seconds,
                    start_utc,
                    rate_cliff_in_seconds
                )                
            },

            StreamInstruction::AnswerUpdate {
                answer
            } => {
                msg!("Instruction: AnswerUpdate");
                
                Self::process_answer_update(
                    accounts, 
                    program_id, 
                    answer
                )
            },

            StreamInstruction::CloseStream => {
                msg!("Instruction: CloseStream");

                Self::process_close_stream(
                    accounts, 
                    program_id
                )
            },

            StreamInstruction::CloseTreasury => {
                msg!("Instruction: CloseTreasury");

                Self::process_close_treasury(
                    accounts, 
                    program_id
                )
            }
        }
    }

    fn process_create_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        stream_name: String,
        treasury_address: Pubkey,
        stream_address: Pubkey,
        beneficiary_address: Pubkey,
        stream_associated_token: Pubkey,
        funding_amount: f64,
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64,
        cliff_vest_percent: f64
        
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasurer_atoken_account_info = next_account_info(account_info_iter)?;

        if !treasurer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let treasurer_owner_account_info = next_account_info(account_info_iter)?;

        if treasurer_account_info.owner != treasurer_owner_account_info.key {
            return Err(StreamError::InvalidSignerAuthority.into());
        }

        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_atoken_account_info = next_account_info(account_info_iter)?;

        if treasury_account_info.owner != program_id {
            return Err(StreamError::InvalidStreamInstruction.into());
        }

        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InvalidStreamInstruction.into());
        }

        let money_streaming_program_account_info = next_account_info(account_info_iter)?;
        // Rent excemption checks
        let rent = Rent::get()?;

        if !rent.is_exempt(treasury_account_info.lamports(), 0) {
            return Err(StreamError::InvalidRentException.into());
        }

        if !rent.is_exempt(stream_account_info.lamports(), Stream::LEN) {
            return Err(StreamError::InvalidRentException.into());
        }

        // Debit Mean Fees from treasurer
        let meanfi_account_info = next_account_info(account_info_iter)?;
        let meanfi_owner_account_info = next_account_info(account_info_iter)?;
        let flat_fee = 0.025f64;
        let fees_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let total_deposits = funding_amount;

        let fees_transfer_ix = system_instruction::transfer(
            treasurer_account_info.key,
            meanfi_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            treasurer_account_info.clone(),
            meanfi_account_info.clone(),
            meanfi_owner_account_info.clone()
        ]);

        // Transfer tokens
        let token_program_account_info = next_account_info(account_info_iter)?;
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasurer_atoken_account_info.key,
            treasury_atoken_account_info.key,
            treasurer_account_info.key,
            &[],
            funding_amount as u64
        )?;

        invoke(&transfer_ix, &[
            token_program_account_info.clone(),
            treasurer_atoken_account_info.clone(),
            treasury_atoken_account_info.clone(),
            treasurer_account_info.clone()
        ]);

        // Update stream contract terms
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        stream.stream_name = stream_name;
        stream.treasurer_address = *treasurer_account_info.key;
        stream.rate_amount = rate_amount;
        stream.rate_interval_in_seconds = rate_interval_in_seconds;
        stream.start_utc = start_utc;
        stream.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream.cliff_vest_amount = cliff_vest_amount;
        stream.cliff_vest_percent = cliff_vest_percent;
        stream.beneficiary_address = beneficiary_address;
        stream.stream_associated_token = stream_associated_token;
        stream.treasury_address = *treasury_account_info.key;
        stream.treasury_estimated_depletion_utc = 0;
        stream.total_deposits = total_deposits;
        stream.total_withdrawals = 0.0;
        stream.initialized = true;
                
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut()); 

        Ok(())
    }

    fn process_add_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        contribution_token_address: Pubkey,
        contribution_amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;

        if !contributor_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let contribution_lamports = contribution_amount;

        if contributor_account_info.owner == &spl_token::id() { // the contribution is in some token so need to swap those tokens to lamports

        }

        let treasury_account_info = next_account_info(account_info_iter)?;
        let contributor_account_authority_info = next_account_info(account_info_iter)?;
        let flat_fee = 0.03f64 * contribution_lamports;
        let contributor_balance = (contributor_account_info.lamports() as f64);

        if contribution_lamports > contributor_balance {
            return Err(StreamError::InsufficientFunds.into());
        }

        // Credit the treasury account
        let transfer_ix = system_instruction::transfer(
            contributor_account_info.key,
            treasury_account_info.key,
            (contribution_lamports - flat_fee) as u64
        );

        invoke(&transfer_ix, &[
            contributor_account_info.clone(),
            treasury_account_info.clone(),
            contributor_account_authority_info.clone()
        ]);

        let meanfi_account_info = next_account_info(account_info_iter)?;
        let meanfi_auth_account_info = next_account_info(account_info_iter)?;

        // Debit Mean fees from contributor and credit MeanFi account
        let meanfi_transfer_ix = system_instruction::transfer(
            contributor_account_info.key,
            meanfi_account_info.key,
            (flat_fee as u64)
        );

        invoke(&meanfi_transfer_ix, &[
            contributor_account_info.clone(),
            meanfi_account_info.clone(),
            meanfi_auth_account_info.clone()
        ]);

        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Update the stream data
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        stream.total_deposits += contribution_lamports;
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        withdrawal_amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let beneficiary_acount_info = next_account_info(account_info_iter)?;
        let beneficiary_atoken_account_info = next_account_info(account_info_iter)?;

        if !beneficiary_acount_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        // Check the treasury token account authority
        let treasury_atoken_account_info = next_account_info(account_info_iter)?;
        let treasury_atoken_account_owner_info = next_account_info(account_info_iter)?;

        if treasury_atoken_account_info.owner != treasury_atoken_account_owner_info.key {
            return Err(StreamError::NotAuthorizedToWithdraw.into());
        }
        
        let stream_account_info = next_account_info(account_info_iter)?;
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let clock = Clock::get()?;
        let rate = stream.rate_amount * (stream.rate_interval_in_seconds as f64);
        let start_block_height = stream.start_utc + stream.rate_cliff_in_seconds;
        let escrow_vested_amount = rate * ((clock.slot - start_block_height) as f64);

        if withdrawal_amount > escrow_vested_amount {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        // Withdraw
        let token_program_account_info = next_account_info(account_info_iter)?;
        let withdraw_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_atoken_account_info.key,
            beneficiary_atoken_account_info.key,
            treasury_atoken_account_owner_info.key,
            &[],
            withdrawal_amount as u64
        )?;

        invoke(&withdraw_ix, &[
            token_program_account_info.clone(),
            treasury_atoken_account_info.clone(),
            beneficiary_atoken_account_info.clone(),
            treasury_atoken_account_owner_info.clone()
        ]);

        // Mean fees
        let meanfi_account_info = next_account_info(account_info_iter)?;
        let meanfi_owner_account_info = next_account_info(account_info_iter)?;
        let fees_lamports = (0.3f64 * withdrawal_amount / 100) * (LAMPORTS_PER_SOL as f64);
        let fees_transfer_ix = system_instruction::transfer(
            beneficiary,
            meanfi_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            initializer_account_info.clone(),
            meanfi_account_info.clone(),
            meanfi_owner_account_info.clone()
        ]);

        // Update stream account data
        stream.total_withdrawals += withdrawal_amount;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        
        Ok(())
    }

    fn process_propose_update(
        accounts: &[AccountInfo], 
        program_id:  &Pubkey,
        proposed_by: Pubkey,
        stream_name: String,
        treasurer_address: Pubkey,
        treasury_address: Pubkey,
        beneficiary_address: Pubkey,
        stream_associated_token: Pubkey,
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }
        
        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasurer_address.ne(&initializer_account_info.key) || 
           stream.beneficiary_address.ne(&initializer_account_info.key) {

            return Err(StreamError::InstructionNotAuthorized.into()); // Only the treasurer or the beneficiary of the stream can propose an update
        }

        let counterparty_account_info = next_account_info(account_info_iter)?;
        
        // if stream.treasurer_address == *initializer_account_info.key {
        //     treasurer_account_info = initializer_account_info;
        //     beneficiary_account_info = counterparty_account_info;
        // } else if stream.treasurer_address == *counterparty_account_info.key {
        //     treasurer_account_info = counterparty_account_info;
        //     beneficiary_account_info = initializer_account_info;
        // } else {
        //     return Err(StreamError::InstructionNotAuthorized.into());
        // }

        let stream_terms_account_info = next_account_info(account_info_iter)?;

        if stream_terms_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into()); // The stream terms' account should be owned by the streaming program
        }

        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.is_initialized() {
            return Err(StreamError::StreamAlreadyInitialized.into());
        }

        stream_terms.proposed_by = *initializer_account_info.key;
        stream_terms.stream_name = stream_name;
        stream_terms.treasurer_address = treasurer_address;
        stream_terms.rate_amount = rate_amount;
        stream_terms.rate_interval_in_seconds = rate_interval_in_seconds;
        stream_terms.start_utc = start_utc;
        stream_terms.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream_terms.beneficiary_address = beneficiary_address;
        stream_terms.initialized = true;

        // Save
        StreamTerms::pack_into_slice(&stream_terms, &mut stream_terms_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_answer_update(
        accounts: &[AccountInfo], 
        program_id: &Pubkey,
        answer: bool

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let stream_terms_account_info = next_account_info(account_info_iter)?;

        if stream_terms_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into()); // The stream terms' account should be owned by the streaming program
        }
        
        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.proposed_by.eq(&initializer_account_info.key) && answer == true {
            return Err(StreamError::InstructionNotAuthorized.into()); // Only the counterparty of a previous of the stream terms can approve it
        }

        let counterparty_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        
        if stream.treasurer_address == *initializer_account_info.key {
            treasurer_account_info = initializer_account_info;
            beneficiary_account_info = counterparty_account_info;
        } else if stream.treasurer_address == *counterparty_account_info.key {
            treasurer_account_info = counterparty_account_info;
            beneficiary_account_info = initializer_account_info;
        } else {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if answer == false { // Rejected: Close stream terms account
            **stream_terms_account_info.lamports.borrow_mut() = 0;
            stream_terms = StreamTerms::default();

        } else { // Approved: Update stream data and close stream terms account

            if stream_terms.stream_name.ne(&stream.stream_name) {
                stream.stream_name = stream.stream_name
            }

            if stream_terms.treasurer_address.ne(&Pubkey::default()) && 
                stream_terms.treasurer_address.ne(&stream.treasurer_address) {

                stream.treasurer_address = stream_terms.treasurer_address;
            }

            if stream_terms.beneficiary_address.ne(&Pubkey::default()) && 
                stream_terms.beneficiary_address.ne(&stream.beneficiary_address) {
                    
                stream.beneficiary_address = stream_terms.beneficiary_address;
            }

            if stream_terms.stream_associated_token.ne(&Pubkey::default()) && 
                stream_terms.stream_associated_token.ne(&stream.stream_associated_token) {
                    
                stream.stream_associated_token = stream_terms.stream_associated_token;
            }

            if stream_terms.treasury_address.ne(&Pubkey::default()) && 
                stream_terms.treasury_address.ne(&stream.treasury_address) {
                    
                stream.treasury_address = stream_terms.treasury_address;
            }

            if stream_terms.rate_amount != 0.0 && stream_terms.rate_amount != stream.rate_amount {       
                stream.rate_amount = stream_terms.rate_amount;
            }

            if stream_terms.rate_interval_in_seconds != 0 && 
               stream.rate_interval_in_seconds != stream_terms.rate_interval_in_seconds { 

                stream.rate_interval_in_seconds = stream_terms.rate_interval_in_seconds;
            }

            if stream_terms.start_utc != 0 && stream_terms.start_utc != stream.start_utc {
                stream.start_utc = stream_terms.start_utc;
            }

            if stream_terms.rate_cliff_in_seconds != 0 && 
                stream_terms.rate_cliff_in_seconds != stream.rate_cliff_in_seconds {

                stream.rate_cliff_in_seconds = stream_terms.rate_cliff_in_seconds;
            }

            // Save stream
            Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        }

        // Save
        StreamTerms::pack_into_slice(&stream_terms, &mut stream_terms_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_close_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }
        
        let stream_account_info = next_account_info(account_info_iter)?;
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasurer_address.ne(&initializer_account_info.key) || 
           stream.beneficiary_address.ne(&initializer_account_info.key) {

            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let counterpart_account_info = next_account_info(account_info_iter)?;
        
        if stream.treasurer_address == *initializer_account_info.key {
            treasurer_account_info = initializer_account_info;
            beneficiary_account_info = counterpart_account_info;
        } else {
            treasurer_account_info = counterpart_account_info;
            beneficiary_account_info = initializer_account_info;
        }
        
        // Stoping the stream and updating data
        let clock = Clock::get()?; 
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64);
        let escrow_vested_amount = rate * ((clock.slot - stream.start_utc) as f64);        
        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        let fees = 0.05f64;
        let fees_lamports = fees * (LAMPORTS_PER_SOL as f64)
        stream.rate_amount = 0.0;

        // Distributing escrow vested amount to the beneficiary
        let treasury_atoken_account_info = next_account_info(account_info_iter)?;
        let treasury_atoken_owner_account_info = next_account_info(account_info_iter)?;
        let beneficiary_atoken_account_info = next_account_info(account_info_iter)?;

        // Let's transfer the escrow vested amounts to the beneficiary
        let token_program_account_info = next_account_info(account_info_iter)?;
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_atoken_account_info.key,
            beneficiary_atoken_account_info.key,
            treasury_atoken_owner_account_info.key,
            &[],
            (escrow_vested_amount - fees) as u64
        )?;

        invoke(&transfer_ix, &[
            token_program_account_info.clone(),
            treasury_atoken_account_info.clone(),
            beneficiary_atoken_account_info.clone(),
            treasury_atoken_owner_account_info.clone()
        ]);

        let meanfi_account_info = next_account_info(account_info_iter)?;
        let meanfi_owner_account_info = next_account_info(account_info_iter)?;

        // Mean fees
        let fees_transfer_ix = system_instruction::transfer(
            initializer_account_info.key,
            meanfi_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            initializer_account_info.clone(),
            meanfi_account_info.clone(),
            meanfi_owner_account_info.clone()
        ]);

        // Close stream account
        **stream_account_info.lamports.borrow_mut() = 0;
        stream = Stream::default();

        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_close_treasury(
        accounts: &[AccountInfo], 
        program_id: &Pubkey

    ) -> ProgramResult {
        
        // let account_info_iter = &mut accounts.iter();
        // let treasurer_account_info = next_account_info(account_info_iter)?;

        // if !treasurer_account_info.is_signer {
        //     return Err(StreamError::MissingInstructionSignature.into());
        // }

        // let treasury_account_info = next_account_info(account_info_iter)?;
        // // From here all accounts passed through the accounts iterator are the stream accounts to be closed

        // while let some_stream_account_info = account_info_iter.next() {
        //     let stream_account_info = some_stream_account_info.unwrap();
        //     let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        //     let instruction = close_stream(
        //         initializer_account_info.key,
        //         stream_account_info.key,
        //         &stream.beneficiary_withdrawal_address,
        //         treasurer_account_info.key,
        //         program_id
        //     );

        //     invoke(&instruction)?;
        // }

        // All streams should be closed now
        
        Ok(())
    }
}

// Program

use solana_program::{
    msg,
    system_instruction,
    program::invoke,
    pubkey::Pubkey,
    // stake_history::Epoch,
    entrypoint::ProgramResult,
    // program_error::ProgramError,
    instruction::{ AccountMeta, Instruction },
    account_info::{ next_account_info, AccountInfo },
    program_pack::{ IsInitialized, Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar }    
};

use crate::{
    // instruction,
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
                treasurer_address,
                beneficiary_withdrawal_address,
                escrow_token_address,
                treasury_address,
                funding_amount,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent

            } => {

                msg!("Instruction: Create Stream");
                msg!("Funding amount: {:?}", funding_amount);

                Self::process_create_stream(
                    accounts, 
                    program_id,
                    stream_name,
                    treasurer_address,
                    beneficiary_withdrawal_address,
                    escrow_token_address,
                    treasury_address,
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

                msg!("Instruction: Add Funds");

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
                beneficiary_withdrawal_address,
                escrow_token_address,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds

            } => {

                msg!("Instruction: Propose Update");
                
                Self::process_propose_update(
                    accounts, 
                    program_id,
                    proposed_by,
                    stream_name,
                    treasurer_address,
                    treasury_address,
                    beneficiary_withdrawal_address,
                    escrow_token_address,
                    rate_amount,
                    rate_interval_in_seconds,
                    start_utc,
                    rate_cliff_in_seconds
                )                
            },

            StreamInstruction::AnswerUpdate {
                answer
            } => {
                msg!("Instruction: Answer Update");
                
                Self::process_answer_update(
                    accounts, 
                    program_id, 
                    answer
                )
            },

            StreamInstruction::CloseStream => {
                msg!("Instruction: Close Stream");

                Self::process_close_stream(
                    accounts, 
                    program_id
                )
            },

            StreamInstruction::CloseTreasury => {
                msg!("Instruction: Close Treasury");

                Self::process_close_treasury(
                    accounts, 
                    program_id
                )
            }

            // _ => Err(StreamError::InvalidStreamInstruction.into())
        }
    }

    fn process_create_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        stream_name: String,
        treasurer_address: Pubkey,
        beneficiary_withdrawal_address: Pubkey,
        escrow_token_address: Pubkey,
        treasury_address: Pubkey,
        funding_amount: u64,
        rate_amount: u64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: u64,
        cliff_vest_percent: u64
        
    ) -> ProgramResult {

        // msg!("Stream name: {:?}", stream_name);
        msg!("Stream name: {:?}", stream_name.to_string());
        msg!("Treasurer address: {:?}", treasurer_address.to_string());
        msg!("Benericiary address: {:?}", beneficiary_withdrawal_address.to_string());
        msg!("Escrow token: {:?}", escrow_token_address.to_string());
        msg!("Treasury address: {:?}", treasury_address.to_string());
        msg!("Funding amount: {:?}", funding_amount);
        msg!("Rate amount: {:?}", rate_amount);
        msg!("Rate interval seconds: {:?}", rate_interval_in_seconds);  

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;

        if !treasurer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let _beneficiary_account_info = next_account_info(account_info_iter)?;
        // Rent excemption checks
        let rent = Rent::get()?;
        let treasury_account_info = next_account_info(account_info_iter)?;

        if !rent.is_exempt(treasury_account_info.lamports(), 0) {
            return Err(StreamError::InvalidRentException.into());
        }

        let stream_account_info = next_account_info(account_info_iter)?;

        if !rent.is_exempt(stream_account_info.lamports(), Stream::LEN) {
            return Err(StreamError::InvalidRentException.into());
        }

        if treasury_account_info.owner != program_id || stream_account_info.owner != program_id {
            return Err(StreamError::InvalidStreamInstruction.into());
        }

        // Update the stream data
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.is_initialized() {
            return Err(StreamError::StreamAlreadyInitialized.into()); // already initialized
        }

        let total_deposits = funding_amount * LAMPORTS_PER_SOL;

        if funding_amount > treasurer_account_info.lamports() {
            return Err(StreamError::InsufficientFunds.into());
        }

        if total_deposits > 0 {
            msg!("Total deposit: {:?}", total_deposits);

            // **treasurer_account_info.lamports.borrow_mut() -= total_deposits;
            // **treasury_account_info.lamports.borrow_mut() += total_deposits;

            // let transfer_ix = system_instruction::transfer(
            //     treasurer_account_info.key,
            //     treasury_account_info.key,
            //     total_deposits
            // );

            // invoke(&transfer_ix, &[
            //     treasurer_account_info.clone(),
            //     treasury_account_info.clone()
            // ]);
        }

        // stream.stream_name = stream_name;
        stream.treasurer_address = treasurer_address;
        stream.rate_amount = rate_amount;
        stream.rate_interval_in_seconds = rate_interval_in_seconds;
        stream.start_utc = start_utc;
        stream.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream.cliff_vest_amount = cliff_vest_amount;
        stream.cliff_vest_percent = cliff_vest_percent;
        stream.beneficiary_withdrawal_address = beneficiary_withdrawal_address;
        stream.escrow_token_address = escrow_token_address;
        stream.treasury_address = treasury_address;
        stream.treasury_estimated_depletion_utc = 0;
        stream.total_deposits = total_deposits;
        stream.total_withdrawals = 0;
        stream.initialized = true;
                
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut()); 

        Ok(())
    }

    fn process_add_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        contribution_token_address: Pubkey,
        contribution_amount: u64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;

        if !contributor_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        // let stream_account_info = next_account_info(account_info_iter)?;

        // if stream_account_info.owner != program_id {
        //     return Err(StreamError::InstructionNotAuthorized.into());
        // }

        let contribution_lamports = contribution_amount;

        if contributor_account_info.owner == &spl_token::id() { // the contribution is in some token so need to swap those tokens to lamports

        }

        let treasury_account_info = next_account_info(account_info_iter)?;

        // Credit the treasury account
        **contributor_account_info.lamports.borrow_mut() -= contribution_lamports;
        **treasury_account_info.lamports.borrow_mut() += contribution_lamports;

        // // Update the stream data
        // let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        // let clock = Clock::get()?;
        // let rate = stream.rate_amount / stream.rate_interval_in_seconds;
        // stream.total_deposits += contribution_lamports;
        // stream.escrow_vested_amount = rate * (clock.slot - stream.start_utc);        
        // let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - stream.escrow_vested_amount;
        // stream.escrow_unvested_amount = escrow_unvested_amount;

        // Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        withdrawal_amount: u64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let beneficiary_account_info = next_account_info(account_info_iter)?;

        if !beneficiary_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }
        
        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Update stream account data
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        let clock = Clock::get()?; 
        let rate = stream.rate_amount / stream.rate_interval_in_seconds;
        let start_block_height = stream.start_utc + stream.rate_cliff_in_seconds;

        if start_block_height > clock.slot {
            return Err(StreamError::InvalidWithdrawalDate.into());
        }

        let escrow_vested_amount = rate * (clock.slot - start_block_height);
        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;

        if withdrawal_amount > escrow_vested_amount {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        let treasury_account_info = next_account_info(account_info_iter)?;

        if stream.escrow_token_address.ne(&Pubkey::default()) {
            // The beneficiary has a token account (escrow_token_address) to swap the withdrawals

        } else {
            // Cretit the beneficiary account
            **treasury_account_info.lamports.borrow_mut() -= escrow_vested_amount;
            **beneficiary_account_info.lamports.borrow_mut() += escrow_vested_amount;
        }

        stream.total_withdrawals += withdrawal_amount;
        // escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;

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
        beneficiary_withdrawal_address: Pubkey,
        escrow_token_address: Pubkey,
        rate_amount: u64,
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
           stream.beneficiary_withdrawal_address.ne(&initializer_account_info.key) {

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
        stream_terms.beneficiary_withdrawal_address = beneficiary_withdrawal_address;
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

            // if stream_terms.stream_name.ne(&String::from("")) && 
            //    stream_terms.stream_name.ne(&stream.stream_name) {

            //     stream.stream_name = stream.stream_name
            // }

            if stream_terms.treasurer_address.ne(&Pubkey::default()) && 
                stream_terms.treasurer_address.ne(&stream.treasurer_address) {

                stream.treasurer_address = stream_terms.treasurer_address;
            }

            if stream_terms.beneficiary_withdrawal_address.ne(&Pubkey::default()) && 
                stream_terms.beneficiary_withdrawal_address.ne(&stream.beneficiary_withdrawal_address) {
                    
                stream.beneficiary_withdrawal_address = stream_terms.beneficiary_withdrawal_address;
            }

            if stream_terms.escrow_token_address.ne(&Pubkey::default()) && 
                stream_terms.escrow_token_address.ne(&stream.escrow_token_address) {
                    
                stream.escrow_token_address = stream_terms.escrow_token_address;
            }

            if stream_terms.treasury_address.ne(&Pubkey::default()) && 
                stream_terms.treasury_address.ne(&stream.treasury_address) {
                    
                stream.treasury_address = stream_terms.treasury_address;
            }

            if stream_terms.rate_amount != 0 && stream_terms.rate_amount != stream.rate_amount {       
                stream.treasury_address = stream_terms.treasury_address;
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
           stream.beneficiary_withdrawal_address.ne(&initializer_account_info.key) {

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
        
        // Stoping the stream adn updating data
        let clock = Clock::get()?; 
        let rate = stream.rate_amount / stream.rate_interval_in_seconds;
        let escrow_vested_amount = rate * (clock.slot - stream.start_utc);        
        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        stream.rate_amount = 0;

        // Distributing escrow vested amount to the beneficiary
        let escrow_vested_amount_in_tokens = 0;

        if stream.escrow_token_address.ne(&Pubkey::default()) { // swap lamports to the token and credit the token account
            // posiblemente para hacer el transfer necesito tener los 2 account sender and recipients porq hay q pasarselos al token swap o token program
        }

        let treasury_account_info = next_account_info(account_info_iter)?;

        **treasury_account_info.lamports.borrow_mut() -= escrow_vested_amount;
        **beneficiary_account_info.lamports.borrow_mut() = escrow_vested_amount;

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

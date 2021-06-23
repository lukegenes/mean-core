// Program

use num_traits;
use solana_program::{
    msg,
    system_program,
    system_instruction,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    system_instruction::SystemInstruction,
    instruction::{ AccountMeta, Instruction },
    account_info::{ next_account_info, AccountInfo },
    program_pack::{ IsInitialized, Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar }    
};

use spl_token::instruction;
// use spl_associated_token_program;

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
                beneficiary_address,
                stream_name,
                funding_amount,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent,
                auto_pause_in_seconds

            } => {

                msg!("Instruction: CreateStream");

                Self::process_create_stream(
                    accounts, 
                    program_id,
                    beneficiary_address,
                    stream_name,
                    funding_amount,
                    rate_amount,
                    rate_interval_in_seconds,
                    start_utc,
                    rate_cliff_in_seconds,
                    cliff_vest_amount,
                    cliff_vest_percent,
                    auto_pause_in_seconds
                )
            },

            StreamInstruction::AddFunds { 
                contribution_amount,
                resume
                
            } => {
                msg!("Instruction: AddFunds");

                Self::process_add_funds(
                    accounts, 
                    program_id,
                    contribution_amount,
                    resume,
                )
            },

            StreamInstruction::Withdraw { withdrawal_amount } => {
                msg!("Instruction: Withdraw");
                
                Self::process_withdraw(
                    accounts, 
                    program_id, 
                    withdrawal_amount
                )
            },

            StreamInstruction::PauseStream => {
                msg!("Instruction: PauseStream");

                Self::process_pause_stream(
                    accounts, 
                    program_id
                )
            },

            StreamInstruction::ResumeStream => {
                msg!("Instruction: ResumeStream");

                Self::process_resume_stream(
                    accounts, 
                    program_id
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

            StreamInstruction::AnswerUpdate { answer } => {
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
        beneficiary_address: Pubkey,
        stream_name: String,
        funding_amount: f64,
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64,
        cliff_vest_percent: f64,
        auto_pause_in_seconds: u64
        
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasurer_token_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_sysvar_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_account_info)?;

        if !treasurer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let clock = Clock::get()?;
        let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());

        if rate_amount == funding_amount && 
           start_utc / 1000 <= (clock.unix_timestamp as u64)
        {
            // One time payment. Transfer directly to beneficiary without creating the stream
            let amount = funding_amount * pow;
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasurer_token_account_info.key,
                beneficiary_token_account_info.key,
                treasurer_account_info.key,
                &[],
                amount as u64
            )?;

            invoke(&transfer_ix, &[
                token_program_account_info.clone(),
                treasurer_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                treasurer_account_info.clone()
            ]);

            msg!("Transfer {:?} tokens to: {:?}", 
                funding_amount, 
                (*beneficiary_token_account_info.key).to_string()
            );
        }
        else 
        {
            let msp_seed = String::from("MoneyStreamingProgram");
            // Creating treasury account (PDA)
            let treasury_seed: &[&[_]] = &[
                &stream_account_info.key.to_bytes(),
                &msp_account_info.key.to_bytes(),
                &msp_seed.as_bytes()
            ];

            let (treasury_address, treasury_bump_seed) = Pubkey::find_program_address(
                treasury_seed,
                program_id
            );

            if treasury_address != *treasury_account_info.key {
                msg!("Error: Treasury address does not match seed derivation");
                return Err(StreamError::InvalidStreamInstruction.into());
            }

            let treasury_signer_seed: &[&[_]] = &[
                &stream_account_info.key.to_bytes(),
                &msp_account_info.key.to_bytes(),
                &msp_seed.as_bytes(),
                &[treasury_bump_seed]
            ];

            let treasury_minimum_balance = rent.minimum_balance(0);
            let create_treasury_ix = system_instruction::create_account(
                treasurer_account_info.key,
                treasury_account_info.key,
                treasury_minimum_balance,
                0,
                program_id
            );

            invoke_signed(&create_treasury_ix, 
                &[
                    treasurer_account_info.clone(),
                    treasury_account_info.clone(),
                    msp_account_info.clone(),
                    system_account_info.clone()
                ], 
                &[treasury_signer_seed]
            );

            msg!("Create treasury account: {:?}", (*treasury_account_info.key).to_string());

            // Transfer tokens
            if funding_amount > 0.0 
            {
                let amount = funding_amount * pow;
                let transfer_ix = spl_token::instruction::transfer(
                    token_program_account_info.key,
                    treasurer_token_account_info.key,
                    treasury_token_account_info.key,
                    treasurer_account_info.key,
                    &[],
                    amount as u64
                )?;

                invoke(&transfer_ix, &[
                    token_program_account_info.clone(),
                    treasurer_token_account_info.clone(),
                    treasury_token_account_info.clone(),
                    treasurer_account_info.clone()
                ]);

                msg!("Transfer {:?} tokens to: {:?}", 
                    funding_amount, 
                    (*treasury_account_info.key).to_string()
                );
            }

            // Update stream contract terms
            msg!("Creating stream contract");
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
            stream.stream_associated_token = *mint_account_info.key;
            stream.treasury_address = treasury_address;
            stream.treasury_estimated_depletion_utc = 0;
            stream.total_deposits = funding_amount;
            stream.total_withdrawals = 0.0;
            stream.escrow_vested_amount_snap = 0.0;
            stream.escrow_vested_amount_snap_block_height = start_utc / 1000;
            stream.auto_pause_in_seconds = auto_pause_in_seconds;
            stream.is_streaming = true;
            stream.initialized = true;
                    
            Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
            msg!("Stream contract created with address: {:?}", (*stream_account_info.key).to_string());
        }

        // Debit Fees from treasurer
        let flat_fee = 0.025f64;
        let fees_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let fees_transfer_ix = system_instruction::transfer(
            treasurer_account_info.key,
            msp_ops_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            treasurer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fees_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

        Ok(())
    }

    fn process_add_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        contribution_amount: f64,
        resume: bool

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;
        let contributor_token_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !contributor_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let fees = 0.025f64;
        let fees_lamports = fees * (LAMPORTS_PER_SOL as f64);
        let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let amount = contribution_amount * pow;

        // Credit the treasury account
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            contributor_token_account_info.key,
            treasury_token_account_info.key,
            contributor_account_info.key,
            &[],
            amount as u64
        )?;

        invoke(&transfer_ix, &[
            token_program_account_info.clone(),
            contributor_token_account_info.clone(),
            treasury_token_account_info.clone(),
            contributor_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens to: {:?}", 
            contribution_amount, 
            (*treasury_account_info.key).to_string()
        );

        // Debit Fees from treasurer
        let flat_fee = 0.025f64;
        let fees_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let fees_transfer_ix = system_instruction::transfer(
            contributor_account_info.key,
            msp_ops_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            contributor_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fees_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

        // Update stream contract terms
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        stream.total_deposits += contribution_amount;

        if !stream.is_streaming && resume == true 
        {
            let is_streaming = match stream.is_streaming 
            {
                false => 0,
                true => 1,
                _ => return Err(StreamError::InvalidStreamData.into()),
            };

            let clock = Clock::get()?;
            let current_block_height = clock.unix_timestamp as u64;
            let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_streaming as f64);
            let elapsed_time = ((current_block_height - stream.escrow_vested_amount_snap_block_height) as f64);
            let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
            
            if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
            {
                escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
                stream.escrow_vested_amount_snap = escrow_vested_amount;
                stream.escrow_vested_amount_snap_block_height = current_block_height;
                stream.is_streaming = false;
            } 
            else 
            {
                stream.is_streaming = true
            }
        }

        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        withdrawal_amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let beneficiary_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !beneficiary_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let is_streaming = match stream.is_streaming 
        {
            false => 0,
            true => 1,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        let clock = Clock::get()?;
        let current_block_height = clock.unix_timestamp as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_streaming as f64);
        let elapsed_time = ((current_block_height - stream.escrow_vested_amount_snap_block_height) as f64);
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        if withdrawal_amount > escrow_vested_amount 
        {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;

        if withdrawal_amount > escrow_vested_amount 
        {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        // Withdraw
        let msp_seed = String::from("MoneyStreamingProgram");
        let treasury_seed: &[&[_]] = &[
            &stream_account_info.key.to_bytes(),
            &msp_account_info.key.to_bytes(),
            &msp_seed.as_bytes()
        ];

        let (treasury_address, treasury_bump_seed) = Pubkey::find_program_address(
            treasury_seed,
            program_id
        );

        if treasury_address != *treasury_account_info.key 
        {
            msg!("Error: Treasury address does not match seed derivation");
            return Err(StreamError::InvalidStreamInstruction.into());
        }

        let treasury_signer_seed: &[&[_]] = &[
            &stream_account_info.key.to_bytes(),
            &msp_account_info.key.to_bytes(),
            &msp_seed.as_bytes(),
            &[treasury_bump_seed]
        ];

        let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let amount = withdrawal_amount * pow;
        let withdraw_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            beneficiary_token_account_info.key,
            treasury_account_info.key,
            &[],
            amount as u64
        )?;

        invoke_signed(&withdraw_ix, 
            &[
                token_program_account_info.clone(),
                treasury_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                treasury_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_signer_seed]
        );

        msg!("Transfer {:?} tokens to: {:?}", 
            amount, 
            (*beneficiary_token_account_info.key).to_string()
        );

        // Update stream account data
        stream.total_withdrawals += withdrawal_amount;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Fees
        let fees = 0.025f64;
        let fees_lamports = fees * (LAMPORTS_PER_SOL as f64);
        let fees_transfer_ix = system_instruction::transfer(
            beneficiary_account_info.key,
            msp_ops_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            beneficiary_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fees_lamports, 
            (*msp_ops_account_info.key).to_string()
        );
        
        Ok(())
    }

    fn process_pause_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id ||
           (*initializer_account_info.key != stream.treasurer_address &&
            *initializer_account_info.key != stream.beneficiary_address &&
            initializer_account_info.key != program_id) // if auto pause then the MSP will be the initializer
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Pausing the stream and updating data
        let is_streaming = match stream.is_streaming 
        {
            false => 0,
            true => 1,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        let clock = Clock::get()?;
        let current_block_height = clock.unix_timestamp as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_streaming as f64);
        let elapsed_time = ((current_block_height - stream.escrow_vested_amount_snap_block_height) as f64);
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        stream.escrow_vested_amount_snap = escrow_vested_amount;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.is_streaming = false;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Pausing the stream");

        if initializer_account_info.key != program_id
        {
            // Debit fees if the initializer of the instruction is not the MSP (auto pause)
            let fees = 0.025f64;
            let fees_lamports = fees * (LAMPORTS_PER_SOL as f64);
            let fees_transfer_ix = system_instruction::transfer(
                initializer_account_info.key,
                msp_ops_account_info.key,
                fees_lamports as u64
            );

            invoke(&fees_transfer_ix, &[
                initializer_account_info.clone(),
                msp_ops_account_info.clone(),
                system_account_info.clone()
            ]);

            msg!("Transfer {:?} lamports of fee to: {:?}", 
                fees_lamports, 
                (*msp_ops_account_info.key).to_string()
            );
        }

        Ok(())
    }

    fn process_resume_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id || 
           (*initializer_account_info.key != stream.treasurer_address &&
            *initializer_account_info.key != stream.beneficiary_address &&
            initializer_account_info.key != program_id) // if auto resume then the MSP will be the initializer
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Resuming the stream and updating data
        stream.is_streaming = true;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Resuming the stream");

        if initializer_account_info.key != program_id
        {
            // Debit fees if the initializer of the instruction is not the MSP (auto pause)
            let fees = 0.025f64;
            let fees_lamports = fees * (LAMPORTS_PER_SOL as f64);
            let fees_transfer_ix = system_instruction::transfer(
                initializer_account_info.key,
                msp_ops_account_info.key,
                fees_lamports as u64
            );

            invoke(&fees_transfer_ix, &[
                initializer_account_info.clone(),
                msp_ops_account_info.clone(),
                system_account_info.clone()
            ]);

            msg!("Transfer {:?} lamports of fee to: {:?}", 
                fees_lamports, 
                (*msp_ops_account_info.key).to_string()
            );
        }

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

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }
        
        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasurer_address.ne(&initializer_account_info.key) || 
           stream.beneficiary_address.ne(&initializer_account_info.key) 
        {
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

        if stream_terms_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // The stream terms' account should be owned by the streaming program
        }

        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.is_initialized() 
        {
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

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let stream_terms_account_info = next_account_info(account_info_iter)?;

        if stream_terms_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // The stream terms' account should be owned by the streaming program
        }
        
        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.proposed_by.eq(&initializer_account_info.key) && answer == true 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // Only the counterparty of a previous of the stream terms can approve it
        }

        let counterparty_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        
        if stream.treasurer_address == *initializer_account_info.key 
        {
            treasurer_account_info = initializer_account_info;
            beneficiary_account_info = counterparty_account_info;
        } 
        else if stream.treasurer_address == *counterparty_account_info.key 
        {
            treasurer_account_info = counterparty_account_info;
            beneficiary_account_info = initializer_account_info;
        } 
        else 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if answer == false // Rejected: Close stream terms account 
        {
            **stream_terms_account_info.lamports.borrow_mut() = 0;
            stream_terms = StreamTerms::default();
        } 
        else // Approved: Update stream data and close stream terms account
        {
            if stream_terms.stream_name.ne(&stream.stream_name) 
            {
                stream.stream_name = stream.stream_name
            }

            if stream_terms.treasurer_address.ne(&Pubkey::default()) && 
                stream_terms.treasurer_address.ne(&stream.treasurer_address) 
            {
                stream.treasurer_address = stream_terms.treasurer_address;
            }

            if stream_terms.beneficiary_address.ne(&Pubkey::default()) && 
                stream_terms.beneficiary_address.ne(&stream.beneficiary_address) 
            {        
                stream.beneficiary_address = stream_terms.beneficiary_address;
            }

            if stream_terms.stream_associated_token.ne(&Pubkey::default()) && 
                stream_terms.stream_associated_token.ne(&stream.stream_associated_token) 
            {       
                stream.stream_associated_token = stream_terms.stream_associated_token;
            }

            if stream_terms.treasury_address.ne(&Pubkey::default()) && 
                stream_terms.treasury_address.ne(&stream.treasury_address) 
            {       
                stream.treasury_address = stream_terms.treasury_address;
            }

            if stream_terms.rate_amount != 0.0 && stream_terms.rate_amount != stream.rate_amount 
            {       
                stream.rate_amount = stream_terms.rate_amount;
            }

            if stream_terms.rate_interval_in_seconds != 0 && 
               stream.rate_interval_in_seconds != stream_terms.rate_interval_in_seconds 
            {
                stream.rate_interval_in_seconds = stream_terms.rate_interval_in_seconds;
            }

            if stream_terms.start_utc != 0 && stream_terms.start_utc != stream.start_utc 
            {
                stream.start_utc = stream_terms.start_utc;
            }

            if stream_terms.rate_cliff_in_seconds != 0 && 
                stream_terms.rate_cliff_in_seconds != stream.rate_cliff_in_seconds 
            {
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
        let counterparty_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;  
        let treasury_token_account_info = next_account_info(account_info_iter)?; 
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasurer_address != *initializer_account_info.key && 
           stream.beneficiary_address != *initializer_account_info.key 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }
        
        if stream.treasurer_address == *initializer_account_info.key 
        {
            treasurer_account_info = initializer_account_info;
            beneficiary_account_info = counterparty_account_info;
        } 
        else 
        {
            treasurer_account_info = counterparty_account_info;
            beneficiary_account_info = initializer_account_info;
        }
        
        // Stoping the stream and updating data
        msg!("Stopping the stream");

        let is_streaming = match stream.is_streaming 
        {
            false => 0,
            true => 1,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        let clock = Clock::get()?;
        let current_block_height = clock.unix_timestamp as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_streaming as f64);
        let elapsed_time = ((current_block_height - stream.escrow_vested_amount_snap_block_height) as f64);
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        stream.is_streaming = false;
        
        // Crediting escrow vested amount to the beneficiary
        if escrow_vested_amount > 0.0 
        {
            let msp_seed = String::from("MoneyStreamingProgram");
            let treasury_seed: &[&[_]] = &[
                &stream_account_info.key.to_bytes(),
                &msp_account_info.key.to_bytes(),
                &msp_seed.as_bytes()
            ];

            let (treasury_address, treasury_bump_seed) = Pubkey::find_program_address(
                treasury_seed,
                program_id
            );

            if treasury_address != *treasury_account_info.key 
            {
                msg!("Error: Treasury address does not match seed derivation");
                return Err(StreamError::InvalidStreamInstruction.into());
            }

            let treasury_signer_seed: &[&[_]] = &[
                &stream_account_info.key.to_bytes(),
                &msp_account_info.key.to_bytes(),
                &msp_seed.as_bytes(),
                &[treasury_bump_seed]
            ];

            let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
            let pow = num_traits::pow(10f64, mint.decimals.into());
            let amount = escrow_vested_amount * pow;
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasury_token_account_info.key,
                beneficiary_token_account_info.key,
                treasury_account_info.key,
                &[],
                amount as u64
            )?;

            invoke_signed(&transfer_ix, 
                &[
                    token_program_account_info.clone(),
                    treasury_token_account_info.clone(),
                    beneficiary_token_account_info.clone(),
                    treasury_account_info.clone(),
                    msp_account_info.clone()
                ],
                &[treasury_signer_seed]
            );

            msg!("Transfer {:?} tokens to: {:?}", 
                amount, 
                (*beneficiary_token_account_info.key).to_string()
            );
        }

        // Distributing escrow unvested amount to contributors
        if escrow_unvested_amount > 0.0
        {
            // get all contributors
            // calculate the amount for each one
            // credit each amount
        }

        // Close stream account
        let treasurer_lamports = treasurer_account_info.lamports();
        **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
            .checked_add(stream_account_info.lamports())
            .ok_or(StreamError::Overflow)?;

        **stream_account_info.lamports.borrow_mut() = 0;
        // Cleaning data
        stream.treasurer_address = Pubkey::default();
        stream.rate_amount = 0.0;
        stream.rate_interval_in_seconds = 0;
        stream.start_utc = 0;
        stream.rate_cliff_in_seconds = 0;
        stream.cliff_vest_amount = 0.0;
        stream.cliff_vest_percent = 0.0;
        stream.beneficiary_address = Pubkey::default();
        stream.stream_associated_token = Pubkey::default();
        stream.treasury_address = Pubkey::default();
        stream.treasury_estimated_depletion_utc = 0;
        stream.total_deposits = 0.0;
        stream.total_withdrawals = 0.0;
        stream.initialized = false;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Closing the stream");

        // Debit fees from the initializer of the instruction
        let fees = 0.05f64;
        let fees_lamports = fees * (LAMPORTS_PER_SOL as f64);
        let fees_transfer_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fees_lamports as u64
        );

        invoke(&fees_transfer_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fees_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

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

// Program

use std::cmp;
use num_traits;
use solana_program::{
    msg,
    system_program,
    system_instruction,
    program::{ invoke, invoke_signed },
    program_option::COption,
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
    instruction::{ StreamInstruction, transfer },
    state::{ Stream, StreamTerms, Treasury, LAMPORTS_PER_SOL, TREASURY_MINT_DECIMALS }
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

            StreamInstruction::RecoverFunds { recover_amount } => {
                msg!("Instruction: RecoverFunds");

                Self::process_recover_funds(
                    accounts, 
                    program_id,
                    recover_amount
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
                beneficiary_address,
                associated_token_address,
                rate_amount,
                rate_interval_in_seconds,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent,
                auto_pause_in_seconds

            } => {

                msg!("Instruction: ProposeUpdate");
                
                Self::process_propose_update(
                    accounts, 
                    program_id,
                    proposed_by,
                    stream_name,
                    treasurer_address,
                    beneficiary_address,
                    associated_token_address,
                    rate_amount,
                    rate_interval_in_seconds,
                    rate_cliff_in_seconds,
                    cliff_vest_amount,
                    cliff_vest_percent,
                    auto_pause_in_seconds
                )                
            },

            StreamInstruction::AnswerUpdate { approve } => {
                msg!("Instruction: AnswerUpdate");
                
                Self::process_answer_update(
                    accounts, 
                    program_id, 
                    approve
                )
            },

            StreamInstruction::CloseStream => {
                msg!("Instruction: CloseStream");

                Self::process_close_stream(
                    accounts, 
                    program_id
                )
            },

            StreamInstruction::CreateTreasury { nounce } => {
                msg!("Instruction: CreateTreasury");

                Self::process_create_treasury(
                    accounts, 
                    program_id,
                    nounce
                )
            },

            StreamInstruction::Transfer { amount } => {
                msg!("Instruction: Transfer");

                Self::process_transfer(
                    accounts, 
                    program_id,
                    amount
                )
            },
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
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !treasurer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let clock = Clock::get()?;
        let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let fee = 0.03f64 * funding_amount * 100f64;
        let amount = (funding_amount - fee);

        if funding_amount == rate_amount &&
           start_utc / 1000 <= (clock.unix_timestamp as u64)
        {
            // One time payment. Transfer directly to beneficiary without creating the stream
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasurer_token_account_info.key,
                beneficiary_token_account_info.key,
                treasurer_account_info.key,
                &[],
                (amount * pow) as u64
            )?;

            invoke(&transfer_ix, &[
                token_program_account_info.clone(),
                treasurer_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                treasurer_account_info.clone()
            ]);

            msg!("Transfer {:?} tokens to: {:?}", 
                amount, 
                (*beneficiary_token_account_info.key).to_string()
            );
        }
        else
        {
            if funding_amount > 0.0 
            {
                // Transfer tokens
                let transfer_ix = spl_token::instruction::transfer(
                    token_program_account_info.key,
                    treasurer_token_account_info.key,
                    treasury_token_account_info.key,
                    treasurer_account_info.key,
                    &[],
                    (amount * pow) as u64
                )?;

                invoke(&transfer_ix, &[
                    token_program_account_info.clone(),
                    treasurer_token_account_info.clone(),
                    treasury_token_account_info.clone(),
                    treasurer_account_info.clone()
                ]);

                msg!("Transfer {:?} tokens to: {:?}", 
                    amount, 
                    (*treasury_account_info.key).to_string()
                );
            }

            let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

            // Updating stream data
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
            stream.treasury_address = *treasury_account_info.key;
            stream.treasury_estimated_depletion_utc = 0;
            stream.total_deposits = 0.0;
            stream.total_withdrawals = 0.0;
            stream.escrow_vested_amount_snap = 0.0;
            stream.escrow_vested_amount_snap_block_height = 0;
            stream.escrow_vested_amount_snap_block_time = 0;
            stream.stream_resumed_block_height = clock.slot as u64;
            stream.stream_resumed_block_time = clock.unix_timestamp as u64;

            if auto_pause_in_seconds != 0 
            {
                stream.auto_pause_in_seconds = auto_pause_in_seconds;
            }
            else 
            {
                stream.auto_pause_in_seconds = (funding_amount * (rate_interval_in_seconds as f64) / rate_amount ) as u64;
            }

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
        let contributor_treasury_token_account_info = next_account_info(account_info_iter)?;
        let beneficiary_mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;        
        let treasury_mint_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !contributor_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let fee = 0.03f64 * contribution_amount / 100f64;
        let amount = contribution_amount - fee;

        // Mint treasury tokens to contributor
        let mint = spl_token::state::Mint::unpack_from_slice(&treasury_mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let mint_to_ix = spl_token::instruction::mint_to(
            token_program_account_info.key,
            treasury_mint_account_info.key,
            contributor_treasury_token_account_info.key,
            treasury_account_info.key,
            &[],
            (amount * pow) as u64
        )?;

        invoke(&mint_to_ix, &[
            treasury_mint_account_info.clone(),
            contributor_treasury_token_account_info.clone(),
            treasury_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Minting {:?} treasury tokens to: {:?}", 
            amount, 
            (*contributor_treasury_token_account_info.key).to_string()
        );

        // Transfer tokens to contributor
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            contributor_token_account_info.key,
            contributor_account_info.key,
            &[],
            (amount * pow) as u64
        )?;

        invoke(&transfer_ix, &[
            contributor_account_info.clone(),
            treasury_token_account_info.clone(),
            contributor_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens to: {:?}",
            amount, 
            (*contributor_token_account_info.key).to_string()
        );

        // Update and resume stream
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        stream.total_deposits += amount;

        if resume == true 
        {
            let clock = Clock::get()?;
            let current_block_height = clock.slot as u64;
            let current_block_time = clock.unix_timestamp as u64;

            stream.stream_resumed_block_height = current_block_height;
            stream.stream_resumed_block_time = current_block_time;
        }
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Pay fees
        let fees_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            contributor_token_account_info.key,
            msp_ops_token_account_info.key,
            contributor_account_info.key,
            &[],
            fee as u64
        )?;

        invoke(&fees_ix, &[
            contributor_account_info.clone(),
            contributor_token_account_info.clone(),
            msp_ops_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens of fee to: {:?}",
            fee, 
            (*msp_ops_token_account_info.key).to_string()
        );

        Ok(())
    }

    fn process_recover_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        recover_amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;
        let contributor_token_account_info = next_account_info(account_info_iter)?;
        let contributor_treasury_token_account_info = next_account_info(account_info_iter)?;
        let contributor_mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let treasury_mint_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !contributor_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }      

        if stream_account_info.owner != program_id 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let fee = 0.03f64 * recover_amount / 100f64;
        let amount = recover_amount - fee;
        let mint = spl_token::state::Mint::unpack_from_slice(&treasury_mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let burn_amount = amount * pow;

        // Burn treasury tokens from the contributor treasury token account       
        let burn_ix = spl_token::instruction::burn(
            token_program_account_info.key,
            contributor_treasury_token_account_info.key,
            treasury_mint_account_info.key,
            treasury_account_info.key,
            &[],
            burn_amount as u64
        )?;

        invoke(&burn_ix, &[
            contributor_treasury_token_account_info.clone(),
            treasury_mint_account_info.clone(),
            treasury_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Burning {:?} treasury tokens from: {:?}", 
            amount, 
            (*contributor_treasury_token_account_info.key).to_string()
        );
        
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?; 
        let clock = Clock::get()?;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time > stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;

        if recover_amount > escrow_vested_amount
        {
            return Err(StreamError::NotAllowedRecoverableAmount.into());
        }

        let transfer_amount = (escrow_unvested_amount * recover_amount * pow) / (mint.supply as f64) * 100f64;

        // Transfer tokens to contributor
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            contributor_token_account_info.key,
            treasury_account_info.key,
            &[],
            transfer_amount as u64
        )?;

        invoke(&transfer_ix, &[
            treasury_account_info.clone(),
            treasury_token_account_info.clone(),
            contributor_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens to: {:?}",
            amount, 
            (*contributor_token_account_info.key).to_string()
        );

        // Update the stream
        stream.total_deposits -= recover_amount;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Check the total supply of the treasury
        if mint.supply == 0
        {
            // Close the treasury
            let mut treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
            let msp_ops_lamports = msp_ops_account_info.lamports();

            **msp_ops_account_info.lamports.borrow_mut() = msp_ops_lamports
                .checked_add(treasury_account_info.lamports())
                .ok_or(StreamError::Overflow)?;

            **treasury_account_info.lamports.borrow_mut() = 0;

            treasury.mint = Pubkey::default();
            treasury.nounce = 0;
            treasury.initialized = false;

            Treasury::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());
        }

        // Pay fees
        let fees_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            contributor_token_account_info.key,
            msp_ops_token_account_info.key,
            contributor_account_info.key,
            &[],
            fee as u64
        )?;

        invoke(&fees_ix, &[
            contributor_account_info.clone(),
            contributor_token_account_info.clone(),
            msp_ops_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens of fee to: {:?}",
            fee, 
            (*msp_ops_token_account_info.key).to_string()
        );

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

        let clock = Clock::get()?;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time > stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
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

        let clock = Clock::get()?;
        let current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64);
        let elapsed_time = current_block_time - stream.stream_resumed_block_time;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * (elapsed_time as f64);
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        stream.escrow_vested_amount_snap = escrow_vested_amount;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
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
        let clock = Clock::get()?;

        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
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
        beneficiary_address: Pubkey,
        associated_token_address: Pubkey,
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64,
        cliff_vest_percent: f64,
        auto_pause_in_seconds: u64

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let stream_terms_account_info = next_account_info(account_info_iter)?;
        let counterparty_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }    

        if stream_terms_account_info.owner != program_id || stream_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasurer_address.ne(&initializer_account_info.key) || 
           stream.beneficiary_address.ne(&initializer_account_info.key) 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // Only the treasurer or the beneficiary of the stream can propose an update
        }

        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.is_initialized() 
        {
            return Err(StreamError::StreamAlreadyInitialized.into());
        }

        stream_terms.proposed_by = *initializer_account_info.key;
        stream_terms.stream_id = *stream_account_info.key;
        stream_terms.stream_name = stream_name;
        stream_terms.treasurer_address = treasurer_address;
        stream_terms.beneficiary_address = beneficiary_address;
        stream_terms.associated_token_address = associated_token_address;
        stream_terms.rate_amount = rate_amount;
        stream_terms.rate_interval_in_seconds = rate_interval_in_seconds;
        stream_terms.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream_terms.cliff_vest_amount = cliff_vest_amount;
        stream_terms.cliff_vest_percent = cliff_vest_percent;

        if auto_pause_in_seconds != 0 
        {
            stream_terms.auto_pause_in_seconds = auto_pause_in_seconds;
        }
        else 
        {
            let funding_amount = stream.total_deposits - stream.total_withdrawals;
            stream_terms.auto_pause_in_seconds = (funding_amount * (rate_interval_in_seconds as f64) / rate_amount ) as u64;
        }

        stream_terms.initialized = true;

        // Save
        StreamTerms::pack_into_slice(&stream_terms, &mut stream_terms_account_info.data.borrow_mut());

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

    fn process_answer_update(
        accounts: &[AccountInfo], 
        program_id: &Pubkey,
        approve: bool

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let stream_terms_account_info = next_account_info(account_info_iter)?;
        let counterparty_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if stream_terms_account_info.owner != program_id || stream_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // The stream terms' account should be owned by the streaming program
        }
        
        let mut stream_terms = StreamTerms::unpack_from_slice(&stream_terms_account_info.data.borrow())?;

        if stream_terms.proposed_by.eq(&initializer_account_info.key) && approve == true 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // Only the counterparty of a previous of the stream terms can approve it
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_terms.stream_id.ne(&stream_account_info.key) 
        {
            return Err(StreamError::InvalidStreamData.into());
        }
        
        if stream.treasurer_address == *initializer_account_info.key 
        {
            treasurer_account_info = initializer_account_info;
        } 
        else if stream.treasurer_address == *counterparty_account_info.key 
        {
            treasurer_account_info = counterparty_account_info;
        } 
        else 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if approve == false // Rejected: Close stream terms account 
        {
            let treasurer_lamports = treasurer_account_info.lamports();
            **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
                .checked_add(stream_terms_account_info.lamports())
                .ok_or(StreamError::Overflow)?;

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

            if stream_terms.associated_token_address.ne(&Pubkey::default()) && 
                stream_terms.associated_token_address.ne(&stream.stream_associated_token) 
            {       
                stream.stream_associated_token = stream_terms.associated_token_address;
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

            if stream_terms.rate_cliff_in_seconds != 0 && 
                stream_terms.rate_cliff_in_seconds != stream.rate_cliff_in_seconds 
            {
                stream.rate_cliff_in_seconds = stream_terms.rate_cliff_in_seconds;
            }

            if stream_terms.cliff_vest_amount != 0.0 && 
                stream_terms.cliff_vest_amount != stream.cliff_vest_amount 
            {
                stream.cliff_vest_amount = stream_terms.cliff_vest_amount;
            }

            if stream_terms.cliff_vest_percent != 100 as f64 && 
                stream_terms.cliff_vest_percent != stream.cliff_vest_percent 
            {
                stream.cliff_vest_percent = stream_terms.cliff_vest_percent;
            }

            if stream_terms.auto_pause_in_seconds != 0 && 
                stream_terms.auto_pause_in_seconds != stream.auto_pause_in_seconds 
            {
                stream.auto_pause_in_seconds = stream_terms.auto_pause_in_seconds;
            }

            // Save stream
            Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        }

        // Save stream terms
        StreamTerms::pack_into_slice(&stream_terms, &mut stream_terms_account_info.data.borrow_mut());

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
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
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
        
        let clock = Clock::get()?;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time > stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        let escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        let fee = 0.03f64 * escrow_vested_amount / 100f64;
        
        // Crediting escrow vested amount to the beneficiary
        if escrow_vested_amount > 0.0 
        {
            let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
            let pow = num_traits::pow(10f64, mint.decimals.into());
            let amount = (escrow_vested_amount - fee);
            // Crediting escrow vested amount to the beneficiary
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasury_token_account_info.key,
                beneficiary_token_account_info.key,
                treasury_account_info.key,
                &[],
                (amount * pow) as u64
            )?;

            invoke(&transfer_ix, &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                token_program_account_info.clone()
            ]);

            msg!("Transfer {:?} tokens to: {:?}",
                amount, 
                (*beneficiary_token_account_info.key).to_string()
            );
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
        stream.escrow_vested_amount_snap = 0.0;
        stream.escrow_vested_amount_snap_block_height = 0;
        stream.stream_resumed_block_height = 0;
        stream.stream_resumed_block_time = 0;
        stream.auto_pause_in_seconds = 0;
        stream.initialized = false;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Closing the stream");

        // Pay fees by the beneficiary
        let fees_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            beneficiary_token_account_info.key,
            msp_ops_token_account_info.key,
            beneficiary_account_info.key,
            &[],
            fee as u64
        )?;

        invoke(&fees_ix, &[
            beneficiary_account_info.clone(),
            beneficiary_token_account_info.clone(),
            msp_ops_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens of fee to: {:?}",
            fee, 
            (*beneficiary_token_account_info.key).to_string()
        );

        // Debit fees from the initializer of the instruction
        let flat_fee = 0.025f64;
        let fees_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
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

    fn process_create_treasury(
        accounts: &[AccountInfo], 
        program_id: &Pubkey,
        nounce: u8

    ) -> ProgramResult {
        
        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if !treasurer_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        // Initialize mint
        let init_mint_ix = spl_token::instruction::initialize_mint(
            token_program_account_info.key,
            mint_account_info.key,
            treasury_account_info.key,
            None,
            TREASURY_MINT_DECIMALS
        )?;

        invoke(&init_mint_ix, &[
            token_program_account_info.clone(),
            mint_account_info.clone(),
            treasury_account_info.clone()
        ]);

        msg!("Initialize treasury mint: {:?}", (*mint_account_info.key).to_string());

        // Update treasury data
        let mut treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;

        treasury.mint = *mint_account_info.key;
        treasury.nounce = nounce;
        treasury.initialized = true;
        // Save
        Treasury::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

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

    fn process_transfer(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount: f64
        
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?;
        let source_token_account_info = next_account_info(account_info_iter)?;
        let destination_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;

        if !source_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mint = spl_token::state::Mint::unpack_from_slice(&mint_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, mint.decimals.into());
        let transfer_amount = amount * pow;

        // Transfer
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            source_token_account_info.key,
            destination_token_account_info.key,
            source_account_info.key,
            &[],
            transfer_amount as u64
        )?;

        invoke(&transfer_ix, &[
            source_account_info.clone(),
            source_token_account_info.clone(),
            destination_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens to: {:?}",
            amount, 
            (*destination_token_account_info.key).to_string()
        );

        Ok(())
    }
}

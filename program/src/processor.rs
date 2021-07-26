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


// use spl_associated_token_program;

use crate::{
    error::StreamError,
    instruction::{ StreamInstruction, transfer },
    state::{ Stream, StreamTerms, Treasury, MSP_ACCOUNT_ADDRESS, LAMPORTS_PER_SOL, TREASURY_MINT_DECIMALS }
};

pub struct Processor {}

impl Processor {

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8]

    ) -> ProgramResult {

        // let msp_account_key = MSP_ACCOUNT_ADDRESS.parse().unwrap();
        // let msp_account_valid = accounts.iter().any(|a| a.key.eq(&msp_account_key));
        
        // if !msp_account_valid {
        //     return Err(StreamError::InstructionNotAuthorized.into());
        // }

        let instruction = StreamInstruction::unpack(instruction_data)?;

        match instruction {

            StreamInstruction::CreateStream {
                beneficiary_address,
                stream_name,
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
                funded_on_utc,
                resume

            } => {
                msg!("Instruction: AddFunds");

                Self::process_add_funds(
                    accounts, 
                    program_id,
                    contribution_amount,
                    funded_on_utc,
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

            StreamInstruction::CreateTreasury { 
                treasury_block_height,
                treasury_base_address

            } => {
                msg!("Instruction: CreateTreasury");

                Self::process_create_treasury(
                    accounts, 
                    program_id,
                    treasury_block_height,
                    treasury_base_address
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
        let treasury_account_info = next_account_info(account_info_iter)?;
        let beneficiary_mint_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;
        let clock = Clock::get()?;

        if !treasurer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if treasury_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let stream_balance = rent.minimum_balance(Stream::LEN);
        let create_stream_ix = system_instruction::create_account(
            treasurer_account_info.key,
            stream_account_info.key,
            stream_balance,
            u64::from_le_bytes(Stream::LEN.to_le_bytes()),
            msp_account_info.key
        );

        invoke(&create_stream_ix, &[
            treasurer_account_info.clone(),
            stream_account_info.clone(),
            msp_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Stream account created with address: {:?}", (*stream_account_info.key).to_string());

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        // Updating stream data
        stream.stream_name = stream_name;
        stream.treasurer_address = *treasurer_account_info.key;
        stream.rate_amount = rate_amount;
        stream.rate_interval_in_seconds = rate_interval_in_seconds;
        stream.funded_on_utc = 0;
        stream.start_utc = start_utc;
        stream.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream.cliff_vest_amount = cliff_vest_amount;
        stream.cliff_vest_percent = cliff_vest_percent;
        stream.beneficiary_address = beneficiary_address;
        stream.beneficiary_associated_token = *beneficiary_mint_account_info.key;
        stream.treasury_address = *treasury_account_info.key;
        stream.treasury_estimated_depletion_utc = 0;
        stream.total_deposits = 0.0;
        stream.total_withdrawals = 0.0;
        stream.escrow_vested_amount_snap = 0.0;
        stream.escrow_vested_amount_snap_block_height = clock.slot as u64;
        stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;
        stream.stream_resumed_block_height = 0;
        stream.stream_resumed_block_time = 0;

        if auto_pause_in_seconds != 0 
        {
            stream.auto_pause_in_seconds = auto_pause_in_seconds;
        }

        stream.initialized = true;
                
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Stream contract successfully created");

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
        funded_on_utc: u64,
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
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if !contributor_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if treasury_account_info.owner != program_id || stream_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Check is the stream needs to be paused because of lacks of funds
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        let current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        let no_funds = (escrow_vested_amount >= stream.total_deposits - stream.total_withdrawals) as u64;

        // Pause if no funds and it is running before
        if no_funds == 1
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
            stream.escrow_vested_amount_snap = escrow_vested_amount;
            stream.escrow_vested_amount_snap_block_height = current_block_height;
            stream.escrow_vested_amount_snap_block_time = current_block_time;
        }

        // Create treasury associated token account if doesn't exist
        let treasury_token_address = spl_associated_token_account::get_associated_token_address(
            treasury_account_info.key,
            beneficiary_mint_account_info.key
        );

        if treasury_token_address != *treasury_token_account_info.key {
            msg!("Error: Treasury associated token address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        if (*treasury_token_account_info.owner).ne(token_program_account_info.key)
        {
            let create_treasury_associated_token_ix = spl_associated_token_account::create_associated_token_account(
                contributor_account_info.key,
                treasury_account_info.key,
                beneficiary_mint_account_info.key
            );

            invoke(&create_treasury_associated_token_ix, &[
                associated_token_program_account_info.clone(),
                contributor_account_info.clone(),
                treasury_token_account_info.clone(),
                treasury_account_info.clone(),
                beneficiary_mint_account_info.clone(),
                system_account_info.clone(),
                token_program_account_info.clone(),
                rent_account_info.clone()
            ]);

            msg!(
                "Treasury associated token account created at: {:?} address", 
                (*treasury_token_account_info.key).to_string()
            );
        }

        let fee = 0.3f64 * contribution_amount / 100f64;
        let amount = contribution_amount - fee;
        let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;

        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes()
            ], 
            msp_account_info.key
        );

        if treasury_pool_address != *treasury_account_info.key 
        {
            msg!("Error: Treasury mint address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        if (*contributor_treasury_token_account_info.key).ne(&Pubkey::default()) &&
           (*treasury_mint_account_info.key).ne(&Pubkey::default())
        {
            if (*contributor_treasury_token_account_info.owner).ne(token_program_account_info.key)
            {
                // Create contributor treasury associated token account
                let contributor_treasury_token_address = spl_associated_token_account::get_associated_token_address(
                    contributor_account_info.key,
                    treasury_mint_account_info.key
                );

                if contributor_treasury_token_address != *contributor_treasury_token_account_info.key {
                    msg!("Error: Contributor associated token address does not match seed derivation");
                    return Err(StreamError::InvalidTreasuryData.into());
                }

                // Create the contributor treasury token account if there is a treasury pool and the account does not exists
                let create_contributor_treasury_atoken_ix = spl_associated_token_account::create_associated_token_account(
                    contributor_account_info.key,
                    contributor_account_info.key,
                    treasury_mint_account_info.key
                );

                invoke(&create_contributor_treasury_atoken_ix, &[
                    associated_token_program_account_info.clone(),
                    contributor_account_info.clone(),
                    contributor_treasury_token_account_info.clone(),
                    treasury_mint_account_info.clone(),
                    system_account_info.clone(),
                    token_program_account_info.clone(),
                    rent_account_info.clone()
                ]);

                msg!(
                    "Contributor associated token account created at: {:?} address", 
                    (*contributor_treasury_token_account_info.key).to_string()
                );
            }
            
            // Mint just if there is a treasury pool
            let treasury_mint = spl_token::state::Mint::unpack_from_slice(&treasury_mint_account_info.data.borrow())?;
            let treasury_mint_signer_seed: &[&[_]] = &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes(),
                &[treasury_pool_bump_seed]
            ];
 
            let treasury_pow = num_traits::pow(10f64, treasury_mint.decimals.into());    
            let mint_to_ix = spl_token::instruction::mint_to(
                token_program_account_info.key,
                treasury_mint_account_info.key,
                contributor_treasury_token_account_info.key,
                treasury_account_info.key,
                &[],
                (amount * treasury_pow) as u64
            )?;

            invoke_signed(&mint_to_ix,
                &[
                    token_program_account_info.clone(),
                    treasury_mint_account_info.clone(),
                    contributor_treasury_token_account_info.clone(),
                    treasury_account_info.clone()
                ],
                &[treasury_mint_signer_seed]
            )?;

            msg!("Minting {:?} treasury pool tokens to: {:?}", 
                amount, 
                (*contributor_treasury_token_account_info.key).to_string()
            );
        }

        // Transfer tokens from contributor to treasury pool
        let beneficiary_mint = spl_token::state::Mint::unpack_from_slice(&beneficiary_mint_account_info.data.borrow())?;
        let beneficiary_pow = num_traits::pow(10f64, beneficiary_mint.decimals.into());
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            contributor_token_account_info.key,
            treasury_token_account_info.key,
            contributor_account_info.key,
            &[],
            (amount * beneficiary_pow) as u64
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

        stream.total_deposits += amount;

        if stream.funded_on_utc == 0 // First time the stream is being funded
        {
            stream.funded_on_utc = funded_on_utc
        }
        // Resume if it was paused by lack of funds OR it was manually paused 
        // and it is going to be manually resumed again        
        if no_funds == 1 || resume == true
        {
            stream.stream_resumed_block_height = current_block_height;
            stream.stream_resumed_block_time = current_block_time;
        }

        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Create the Money Streaming Program operations token account if not exists
        let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
            msp_ops_account_info.key,
            beneficiary_mint_account_info.key
        );

        if msp_ops_token_address != *msp_ops_token_account_info.key {
            msg!("Error: Treasury associated token address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        if *msp_ops_token_account_info.owner != *token_program_account_info.key
        {
            let create_msp_associated_token_ix = spl_associated_token_account::create_associated_token_account(
                contributor_account_info.key,
                msp_ops_account_info.key,
                beneficiary_mint_account_info.key
            );

            invoke(&create_msp_associated_token_ix, &[
                associated_token_program_account_info.clone(),
                contributor_account_info.clone(),
                msp_ops_token_account_info.clone(),
                msp_ops_account_info.clone(),
                beneficiary_mint_account_info.clone(),
                system_account_info.clone(),
                token_program_account_info.clone(),
                rent_account_info.clone()
            ]);

            msg!(
                "Money Streaming Program associated token account created at: {:?} address", 
                (*msp_ops_token_account_info.key).to_string()
            );
        }

        // Pay fees
        let fees_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            contributor_token_account_info.key,
            msp_ops_token_account_info.key,
            contributor_account_info.key,
            &[],
            (fee * beneficiary_pow) as u64
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
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let _system_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if !contributor_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }      

        if treasury_account_info.owner != program_id || stream_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Get contributor treasury associated token account
        let contributor_treasury_token_address = spl_associated_token_account::get_associated_token_address(
            contributor_account_info.key,
            treasury_mint_account_info.key
        );

        if contributor_treasury_token_address.ne(contributor_treasury_token_account_info.key) 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let fee = 0.3f64 * recover_amount / 100f64;
        let treasury_mint = spl_token::state::Mint::unpack_from_slice(&treasury_mint_account_info.data.borrow())?;
        let treasury_mint_pow = num_traits::pow(10f64, treasury_mint.decimals.into());
        let burn_amount = recover_amount * treasury_mint_pow;
        let recover_amount_percent = recover_amount / (treasury_mint.supply as f64) * 100f64; // The percent that represents the `recover_amount` in the pool

        // Burn treasury tokens from the contributor treasury token account       
        let burn_ix = spl_token::instruction::burn(
            token_program_account_info.key,
            contributor_treasury_token_account_info.key,
            treasury_mint_account_info.key,
            contributor_account_info.key,
            &[],
            burn_amount as u64
        )?;

        invoke(&burn_ix, &[
            token_program_account_info.clone(),
            contributor_treasury_token_account_info.clone(),
            treasury_mint_account_info.clone(),
            contributor_account_info.clone()
        ]);

        msg!("Burning {:?} treasury tokens from: {:?}", 
            recover_amount, 
            (*contributor_treasury_token_account_info.key).to_string()
        );
        
        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?; 
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
        let amount = recover_amount_percent * escrow_unvested_amount / 100f64; // The amount calculated by the percent of the pool that the contributor owns
        let transfer_amount = amount - fee;

        if transfer_amount > escrow_unvested_amount
        {
            return Err(StreamError::NotAllowedRecoverableAmount.into());
        }

        // Transfer tokens to contributor        
        let contributor_mint = spl_token::state::Mint::unpack_from_slice(&contributor_mint_account_info.data.borrow())?;
        let contributor_mint_pow = num_traits::pow(10f64, contributor_mint.decimals.into());
        let mut treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes()
            ], 
            msp_account_info.key
        );

        if treasury_pool_address.ne(treasury_account_info.key)
        {
            msg!("Error: Treasury pool address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_signer_seed: &[&[_]] = &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes(),
            &[treasury_pool_bump_seed]
        ];

        let contributor_transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            contributor_token_account_info.key,
            treasury_account_info.key,
            &[],
            (transfer_amount * contributor_mint_pow) as u64
        )?;

        invoke_signed(&contributor_transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                contributor_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_signer_seed]
        );

        msg!("Transfer {:?} tokens to: {:?}",
            transfer_amount, 
            (*contributor_token_account_info.key).to_string()
        );

        // Update the stream
        stream.total_deposits -= transfer_amount;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Check the total supply of the treasury
        if treasury_mint.supply == 0
        {
            // Cleaning treasury data
            treasury.treasury_block_height = 0;
            treasury.treasury_mint_address = Pubkey::default();
            treasury.treasury_base_address = Pubkey::default();
            treasury.initialized = false;

            Treasury::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

            // Close the treasury
            let msp_ops_lamports = msp_ops_account_info.lamports();
            let treasury_lamports = treasury_account_info.lamports();

            **treasury_account_info.lamports.borrow_mut() = 0;
            **msp_ops_account_info.lamports.borrow_mut() = msp_ops_lamports
                .checked_add(treasury_lamports)
                .ok_or(StreamError::Overflow)?;

            msg!("Closing the treasury");
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
        let beneficiary_mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if !beneficiary_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if stream_account_info.owner != program_id || treasury_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        let _current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount >= (stream.total_deposits - stream.total_withdrawals)
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        if withdrawal_amount > escrow_vested_amount
        {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        let _escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        let fee = 0.3f64 * withdrawal_amount / 100f64;
        let transfer_amount = withdrawal_amount - fee;
        let beneficiary_mint = spl_token::state::Mint::unpack_from_slice(&beneficiary_mint_account_info.data.borrow())?;
        let beneficiary_mint_pow = num_traits::pow(10f64, beneficiary_mint.decimals.into());

        // Withdraw
        let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes()
            ], 
            msp_account_info.key
        );

        if treasury_pool_address.ne(treasury_account_info.key)
        {
            msg!("Error: Treasury pool address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_signer_seed: &[&[_]] = &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes(),
            &[treasury_pool_bump_seed]
        ];

        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            beneficiary_token_account_info.key,
            treasury_account_info.key,
            &[],
            (transfer_amount * beneficiary_mint_pow) as u64
        )?;

        invoke_signed(&transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_signer_seed]
        );

        msg!("Transfer {:?} tokens to: {:?}",
            transfer_amount, 
            (*beneficiary_token_account_info.key).to_string()
        );

        // Update stream account data
        stream.total_withdrawals += withdrawal_amount;
        stream.escrow_vested_amount_snap = escrow_vested_amount - withdrawal_amount;
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64; 
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Pay fees
        let fees_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            beneficiary_token_account_info.key,
            msp_ops_token_account_info.key,
            beneficiary_account_info.key,
            &[],
            (fee * beneficiary_mint_pow) as u64
        )?;

        invoke(&fees_ix, &[
            beneficiary_account_info.clone(),
            beneficiary_token_account_info.clone(),
            msp_ops_token_account_info.clone(),
            token_program_account_info.clone()
        ]);

        msg!("Transfer {:?} tokens of fee to: {:?}",
            fee, 
            (*msp_ops_token_account_info.key).to_string()
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
        let clock = Clock::get()?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id ||
        (
            stream.treasurer_address.ne(initializer_account_info.key) && 
            stream.beneficiary_address.ne(initializer_account_info.key)
        )
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

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

        // Pay fees
        let fee = 0.025f64;
        let fee_lamports = fee * (LAMPORTS_PER_SOL as f64);
        let fee_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fee_lamports as u64
        );

        invoke(&fee_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fee_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

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
        let clock = Clock::get()?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_account_info.owner != program_id || 
        (
            stream.treasurer_address.ne(initializer_account_info.key) && 
            stream.beneficiary_address.ne(initializer_account_info.key)
        )
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Resuming the stream and updating data
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());
        msg!("Resuming the stream");

        // Pay fees
        let fee = 0.025f64;
        let fee_lamports = fee * (LAMPORTS_PER_SOL as f64);
        let fee_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fee_lamports as u64
        );

        invoke(&fee_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fee_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

        Ok(())
    }

    fn process_propose_update(
        accounts: &[AccountInfo], 
        program_id:  &Pubkey,
        _proposed_by: Pubkey,
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

        let _treasurer_account_info: &AccountInfo;
        let _beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let stream_terms_account_info = next_account_info(account_info_iter)?;
        let _counterparty_account_info = next_account_info(account_info_iter)?;
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

        if stream.treasurer_address.ne(initializer_account_info.key) &&
           stream.beneficiary_address.ne(initializer_account_info.key)
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
        let flat_fee = 0.025f64;
        let fee_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let fee_transfer_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fee_lamports as u64
        );

        invoke(&fee_transfer_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fee_lamports, 
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

        if stream_terms.proposed_by.eq(initializer_account_info.key) && approve == true
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // Only the counterparty of a previous of the stream terms can approve it
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream_terms.stream_id.ne(stream_account_info.key) 
        {
            return Err(StreamError::InvalidStreamData.into());
        }
        
        if stream.treasurer_address.eq(initializer_account_info.key)
        {
            treasurer_account_info = initializer_account_info;
        } 
        else if stream.treasurer_address.eq(counterparty_account_info.key) 
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
            let stream_terms_lamports = stream_terms_account_info.lamports();

            **stream_terms_account_info.lamports.borrow_mut() = 0;
            **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
                .checked_add(stream_terms_lamports)
                .ok_or(StreamError::Overflow)?;
            
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
                stream_terms.associated_token_address.ne(&stream.beneficiary_associated_token) 
            {       
                stream.beneficiary_associated_token = stream_terms.associated_token_address;
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
        let flat_fee = 0.025f64;
        let fee_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let fee_transfer_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fee_lamports as u64
        );

        invoke(&fee_transfer_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fee_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

        Ok(())
    }

    fn process_close_stream(
        accounts: &[AccountInfo],
        _program_id: &Pubkey

    ) -> ProgramResult {

        let treasurer_account_info: &AccountInfo;
        let beneficiary_account_info: &AccountInfo;
        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let counterparty_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let beneficiary_mint_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;  
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let msp_ops_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if !initializer_account_info.is_signer 
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
        let current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
        let rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
        let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
        let elapsed_time = (current_block_time - marker_block_time) as f64;
        let mut escrow_vested_amount = stream.escrow_vested_amount_snap + rate * elapsed_time;
        
        if escrow_vested_amount > stream.total_deposits - stream.total_withdrawals 
        {
            escrow_vested_amount = stream.total_deposits - stream.total_withdrawals;
        }

        let _escrow_unvested_amount = stream.total_deposits - stream.total_withdrawals - escrow_vested_amount;
        // Pausing the stream
        stream.escrow_vested_amount_snap = escrow_vested_amount;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
        msg!("Pausing the stream");

        if stream.treasurer_address.ne(initializer_account_info.key) &&
           stream.beneficiary_address.ne(initializer_account_info.key) 
        {
            return Err(StreamError::InstructionNotAuthorized.into()); // Just the treasurer or the beneficiary can close a stream
        }
        
        if stream.treasurer_address.eq(initializer_account_info.key)
        {
            treasurer_account_info = initializer_account_info;
            beneficiary_account_info = counterparty_account_info;
        } 
        else 
        {
            treasurer_account_info = counterparty_account_info;
            beneficiary_account_info = initializer_account_info;
        }
        
        if escrow_vested_amount > 0.0 
        {
            // Crediting escrow vested amount to the beneficiary
            let beneficiary_mint = spl_token::state::Mint::unpack_from_slice(&beneficiary_mint_account_info.data.borrow())?;
            let beneficiary_mint_pow = num_traits::pow(10f64, beneficiary_mint.decimals.into());
            let beneficiary_fee = 0.3f64 * escrow_vested_amount / 100f64;
            let transfer_amount = escrow_vested_amount - beneficiary_fee;            
            let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
            let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
                &[
                    treasury.treasury_base_address.as_ref(),
                    &treasury.treasury_block_height.to_le_bytes()
                ], 
                msp_account_info.key
            );

            if treasury_pool_address.ne(treasury_account_info.key)
            {
                msg!("Error: Treasury pool address does not match seed derivation");
                return Err(StreamError::InvalidTreasuryData.into());
            }

            let treasury_signer_seed: &[&[_]] = &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes(),
                &[treasury_pool_bump_seed]
            ];

            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasury_token_account_info.key,
                beneficiary_token_account_info.key,
                treasury_account_info.key,
                &[],
                (transfer_amount * beneficiary_mint_pow) as u64
            )?;

            invoke_signed(&transfer_ix, 
                &[
                    treasury_account_info.clone(),
                    treasury_token_account_info.clone(),
                    beneficiary_token_account_info.clone(),
                    token_program_account_info.clone(),
                    msp_account_info.clone()
                ],
                &[treasury_signer_seed]
            );

            msg!("Transfer {:?} tokens to: {:?}",
                transfer_amount, 
                (*beneficiary_token_account_info.key).to_string()
            );

            // Pay fee by the beneficiary
            let beneficiary_fee_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasury_token_account_info.key,
                msp_ops_token_account_info.key,
                treasury_account_info.key,
                &[],
                (beneficiary_fee * beneficiary_mint_pow) as u64
            )?;

            invoke_signed(&beneficiary_fee_ix, 
                &[
                    treasury_account_info.clone(),
                    treasury_token_account_info.clone(),
                    msp_ops_token_account_info.clone(),
                    token_program_account_info.clone(),
                    msp_account_info.clone()
                ],
                &[treasury_signer_seed]
            );

            msg!("Transfer {:?} tokens of fee to: {:?}",
                beneficiary_fee, 
                (*msp_ops_token_account_info.key).to_string()
            );
        }
            
        // Cleaning data
        stream.treasurer_address = Pubkey::default();
        stream.rate_amount = 0.0;
        stream.rate_interval_in_seconds = 0;
        stream.start_utc = 0;
        stream.rate_cliff_in_seconds = 0;
        stream.cliff_vest_amount = 0.0;
        stream.cliff_vest_percent = 0.0;
        stream.beneficiary_address = Pubkey::default();
        stream.beneficiary_associated_token = Pubkey::default();
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

        // Debit fees from the initializer of the instruction
        let flat_fee = 0.025f64;
        let fee_lamports = flat_fee * (LAMPORTS_PER_SOL as f64);
        let fee_transfer_ix = system_instruction::transfer(
            initializer_account_info.key,
            msp_ops_account_info.key,
            fee_lamports as u64
        );

        invoke(&fee_transfer_ix, &[
            initializer_account_info.clone(),
            msp_ops_account_info.clone(),
            system_account_info.clone()
        ]);

        msg!("Transfer {:?} lamports of fee to: {:?}", 
            fee_lamports, 
            (*msp_ops_account_info.key).to_string()
        );

        // Close stream account
        let initializer_lamports = initializer_account_info.lamports();
        let stream_lamports = stream_account_info.lamports();

        **stream_account_info.lamports.borrow_mut() = 0;
        **initializer_account_info.lamports.borrow_mut() = initializer_lamports
            .checked_add(stream_lamports)
            .ok_or(StreamError::Overflow)?;

        msg!("Closing the stream");

        Ok(())
    }

    fn process_create_treasury(
        accounts: &[AccountInfo], 
        _program_id: &Pubkey,
        treasury_block_height: u64,
        treasury_base_address: Pubkey

    ) -> ProgramResult {
        
        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let treasury_token_mint_account_info = next_account_info(account_info_iter)?;
        let treasury_mint_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let msp_ops_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;

        if !treasurer_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        // Create treasury account
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury_base_address.as_ref(),
                &treasury_block_height.to_le_bytes(),
            ], 
            msp_account_info.key
        );

        if treasury_pool_address.ne(treasury_account_info.key) 
        {
            msg!("Error: Treasury pool address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_pool_signer_seed: &[&[_]] = &[
            treasury_base_address.as_ref(),
            &treasury_block_height.to_le_bytes(),
            &[treasury_pool_bump_seed]
        ];

        let treasury_pool_balance = rent.minimum_balance(Treasury::LEN);
        let create_treasury_pool_ix = system_instruction::create_account(
            treasurer_account_info.key,
            treasury_account_info.key,
            treasury_pool_balance,
            u64::from_le_bytes(Treasury::LEN.to_le_bytes()),
            msp_account_info.key
        );

        invoke_signed(&create_treasury_pool_ix, 
            &[
                treasurer_account_info.clone(),
                treasury_account_info.clone(),
                msp_account_info.clone(),
                system_account_info.clone()
            ], 
            &[treasury_pool_signer_seed]
        );

        msg!(
            "Treasury account created at: {:?} address", 
            treasury_pool_address.to_string()
        );

        // Create treasury associated token account
        let treasury_token_address = spl_associated_token_account::get_associated_token_address(
            treasury_account_info.key,
            treasury_token_mint_account_info.key
        );

        if treasury_token_address.ne(treasury_token_account_info.key) 
        {
            msg!("Error: Treasury associated token address does not match seed derivation");
            return Err(StreamError::InvalidTreasuryData.into());
        }

        if (*treasury_token_account_info.owner).ne(token_program_account_info.key)
        {
            let create_treasury_associated_token_ix = spl_associated_token_account::create_associated_token_account(
                treasurer_account_info.key,
                treasury_account_info.key,
                treasury_token_mint_account_info.key
            );

            invoke(&create_treasury_associated_token_ix, &[
                associated_token_program_account_info.clone(),
                treasurer_account_info.clone(),
                treasury_token_account_info.clone(),
                treasury_account_info.clone(),
                treasury_token_mint_account_info.clone(),
                system_account_info.clone(),
                token_program_account_info.clone(),
                rent_account_info.clone()
            ]);

            msg!(
                "Treasury associated token account created at: {:?} address", 
                treasury_token_address.to_string()
            );
        }

        if (*treasury_mint_account_info.key).ne(&Pubkey::default())
        {
            // Create treasury mint
            let (treasury_mint_address, treasury_mint_bump_seed) = Pubkey::find_program_address(
                &[
                    treasury_base_address.as_ref(),
                    treasury_pool_address.as_ref(),
                    &treasury_block_height.to_le_bytes()
                ], 
                msp_account_info.key
            );

            if treasury_mint_address.ne(treasury_mint_account_info.key)
            {
                msg!("Error: Treasury mint address does not match seed derivation");
                return Err(StreamError::InvalidTreasuryData.into());
            }

            let treasury_mint_signer_seed: &[&[_]] = &[
                treasury_base_address.as_ref(),
                treasury_pool_address.as_ref(),
                &treasury_block_height.to_le_bytes(),
                &[treasury_mint_bump_seed]
            ];

            let treasury_mint_balance = rent.minimum_balance(spl_token::state::Mint::LEN);
            let create_treasury_mint_ix = system_instruction::create_account(
                treasurer_account_info.key,
                treasury_mint_account_info.key,
                treasury_mint_balance,
                u64::from_le_bytes(spl_token::state::Mint::LEN.to_le_bytes()),
                token_program_account_info.key
            );

            invoke_signed(&create_treasury_mint_ix, 
                &[
                    treasurer_account_info.clone(),
                    treasury_mint_account_info.clone(),
                    token_program_account_info.clone(),
                    system_account_info.clone()
                ], 
                &[treasury_mint_signer_seed]
            );

            msg!(
                "Treasury mint account created at: {:?} address", 
                treasury_mint_address.to_string()
            );

            // Initialize treasury mint
            let init_mint_ix = spl_token::instruction::initialize_mint(
                token_program_account_info.key,
                treasury_mint_account_info.key,
                treasury_account_info.key, // msp_account_info.key,
                None,
                TREASURY_MINT_DECIMALS
            )?;

            invoke(&init_mint_ix, &[
                token_program_account_info.clone(),
                treasury_mint_account_info.clone(),
                treasury_account_info.clone(), // msp_account_info.clone(),
                rent_account_info.clone()
            ]);

            msg!("Treasury mint account initialized");
        }

        // Update treasury data
        let mut treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;

        treasury.treasury_block_height = treasury_block_height;
        treasury.treasury_mint_address = *treasury_mint_account_info.key;
        treasury.treasury_base_address = treasury_base_address;
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
        _program_id: &Pubkey,
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

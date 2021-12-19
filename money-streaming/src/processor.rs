// Program
use num_traits;
use crate::error::StreamError;
use crate::utils::*;
use crate::instruction::{ StreamInstruction };
use crate::state::*;
use crate::constants::*;
use crate::account_validations::*;
use crate::extensions::*;
use crate::backwards_comp::*;
use solana_program::{
    msg,
    program::{ invoke },
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    account_info::{ next_account_info, AccountInfo },
    program_pack::{ Pack },
    sysvar::{ clock::Clock, Sysvar } 
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
                rate_amount,
                rate_interval_in_seconds,
                allocation_reserved,
                allocation_assigned,
                funded_on_utc,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent,
                auto_pause_in_seconds

            } => {

                msg!("Instruction: CreateStream");

                Self::process_create_stream(
                    accounts, program_id, stream_name,
                    rate_amount, rate_interval_in_seconds,
                    allocation_reserved, allocation_assigned,
                    funded_on_utc, start_utc, rate_cliff_in_seconds,
                    cliff_vest_amount, cliff_vest_percent, auto_pause_in_seconds
                )
            },

            StreamInstruction::AddFunds { 
                amount,
                allocation_type,
                allocation_stream_address

            } => {
                msg!("Instruction: AddFunds");

                Self::process_add_funds(
                    accounts, program_id, amount,
                    allocation_type, allocation_stream_address
                )
            },

            StreamInstruction::Withdraw { amount } => {
                msg!("Instruction: Withdraw");
                Self::process_withdraw(accounts, program_id, amount)
            },

            StreamInstruction::PauseStream => {
                msg!("Instruction: PauseStream");
                Self::process_pause_stream(accounts, program_id)
            },

            StreamInstruction::ResumeStream => {
                msg!("Instruction: ResumeStream");
                Self::process_resume_stream(accounts, program_id)
            },

            StreamInstruction::CloseStream { auto_close_treasury } => {
                msg!("Instruction: CloseStream");
                Self::process_close_stream(accounts, program_id, auto_close_treasury)
            },

            StreamInstruction::CreateTreasury { 
                slot,
                label,
                treasury_type,
                auto_close

            } => {

                msg!("Instruction: CreateTreasury");
                Self::process_create_treasury(
                    accounts, program_id, slot, label, treasury_type, auto_close
                )
            },

            StreamInstruction::CloseTreasury => {
                msg!("Instruction: CloseTreasury");
                Self::process_close_treasury(accounts, program_id)
            },
        }
    }

    fn process_create_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        stream_name: String,
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        allocation_reserved: f64,
        allocation_assigned: f64,
        funded_on_utc: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64,
        cliff_vest_percent: f64,
        auto_pause_in_seconds: u64
        
    ) -> ProgramResult {

        // Get accounts
        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let beneficiary_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;

        // Verify the correct MSP Operations Account 
        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let _ = check_can_create_stream(
            program_id, &treasurer_account_info, &treasury_account_info, 
            &associated_token_mint_info, &stream_account_info, &msp_account_info,
            &system_account_info, &rent_account_info, allocation_assigned, allocation_reserved
        )?;        
        // Create stream account
        let _ = create_stream_account(
            &treasurer_account_info, &stream_account_info, &msp_account_info,
            &rent_account_info, &system_account_info
        )?;

        let clock = Clock::get()?;
        let mut stream = StreamV2::unpack_from_slice(&stream_account_info.data.borrow())?;
        // Updating stream data
        stream.stream_name = stream_name;
        stream.treasurer_address = *treasurer_account_info.key;
        stream.rate_amount = rate_amount;
        stream.rate_interval_in_seconds = rate_interval_in_seconds;
        stream.allocation_reserved = allocation_reserved;
        stream.allocation_assigned = allocation_assigned;
        stream.allocation_left = allocation_assigned;
        stream.funded_on_utc = funded_on_utc;
        stream.start_utc = start_utc;
        stream.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream.cliff_vest_amount = cliff_vest_amount;
        stream.cliff_vest_percent = cliff_vest_percent;
        stream.beneficiary_address = *beneficiary_account_info.key;
        stream.beneficiary_associated_token = *associated_token_mint_info.key;
        stream.treasury_address = *treasury_account_info.key;
        stream.treasury_estimated_depletion_utc = 0;
        stream.escrow_vested_amount_snap_slot = clock.slot as u64;
        stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;
        stream.stream_resumed_slot = clock.slot;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
        stream.auto_pause_in_seconds = auto_pause_in_seconds;

        let status = get_stream_status(&stream, &clock)?;

        if status == StreamStatus::Scheduled {
            stream.stream_resumed_block_time = start_utc / 1000u64;
        }

        // if there is a cliff amount then assign it the escrow vested snap so
        // that way is included in the calculation of the vested amount
        let mut cliff_amount = cliff_vest_amount;
        
        if cliff_vest_percent > 0.0 {
            cliff_amount = cliff_amount * allocation_assigned / 100f64;
        }

        stream.escrow_vested_amount_snap = cliff_amount;

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let _ = create_stream_update_treasury(&treasury_account_info, &stream, associated_token_mint.decimals.into())?;        
        // Save stream
        stream.initialized = true;
        StreamV1::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        // Pay fee
        transfer_sol_fee(
            &system_account_info,
            &treasurer_account_info,
            &fee_treasury_account_info, 
            CREATE_STREAM_FLAT_FEE
        )
    }

    fn process_add_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount: f64,
        allocation_type: u8,
        allocation_stream_address: Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;
        let contributor_token_account_info = next_account_info(account_info_iter)?;
        let contributor_treasury_pool_token_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;    
        let treasury_pool_mint_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let associated_token_program_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;

        // Verify the correct MSP Operations Account 
        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN {
            return add_funds_v0(
                program_id, msp_account_info, fee_treasury_account_info,
                associated_token_program_account_info, token_program_account_info,
                system_account_info, rent_account_info, contributor_account_info,
                contributor_token_account_info, contributor_treasury_pool_token_account_info,
                treasury_account_info, treasury_token_account_info, associated_token_mint_info,   
                treasury_pool_mint_info, stream_account_info, amount
            );
        }

        let _ = check_can_add_funds(
            program_id, &msp_account_info, &contributor_account_info,
            &contributor_token_account_info, &contributor_treasury_pool_token_account_info,
            &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
            &treasury_pool_mint_info, &stream_account_info, &associated_token_program_account_info,
            &token_program_account_info, &rent_account_info, &system_account_info
        )?;
        // Create contributor deposit receipt
        let _ = create_deposit_receipt(
            &treasury_account_info, &treasury_pool_mint_info,
            &contributor_treasury_pool_token_account_info, &msp_account_info,
            &token_program_account_info, amount
        )?;
        // Transfer tokens from contributor to treasury associated token account
        let _ = transfer_tokens(
            &contributor_account_info, &contributor_token_account_info,
            &treasury_token_account_info, &associated_token_mint_info,
            &token_program_account_info, amount
        )?;
        // Update and save treasury
        let _ = add_funds_update_treasury(
            &treasury_account_info, &associated_token_mint_info, allocation_type, amount
        )?;

        if stream_account_info.data_len() == StreamV1::LEN {
            let clock = Clock::get()?;
            let _ = add_funds_update_stream(
                &stream_account_info, &associated_token_mint_info,
                &clock, &allocation_stream_address, allocation_type, amount
            )?;
        }
        // Pay fees
        transfer_sol_fee(
            &system_account_info,
            &contributor_account_info,
            &fee_treasury_account_info, 
            ADD_FUNDS_FLAT_FEE
        )
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let beneficiary_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let associated_token_program_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()){
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN && stream_account_info.data_len() == Stream::LEN {
            return withdraw_v0(
                program_id, &beneficiary_account_info, &beneficiary_token_account_info,
                &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
                &stream_account_info, &fee_treasury_account_info, &fee_treasury_token_account_info,
                &msp_account_info, &associated_token_program_account_info, &token_program_account_info,
                &rent_account_info, &system_account_info, &clock, amount
            );
        }

        let _ = check_can_withdraw_funds(
            program_id, &beneficiary_account_info, &beneficiary_token_account_info,
            &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
            &stream_account_info, &fee_treasury_token_account_info, &msp_account_info,
            &associated_token_program_account_info, &token_program_account_info,
            &rent_account_info, &system_account_info
        )?;

        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;        
        let withdrawable_amount = get_beneficiary_withdrawable_amount(&stream, &clock, associated_token_mint.decimals.into())?;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let withdraw_request_amount = (amount * pow) as u64;

        if withdraw_request_amount > withdrawable_amount {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        if beneficiary_token_account_info.data_len() == 0 { // Create treasury associated token account if doesn't exist
            let _ = create_ata_account(
                &system_account_info, &rent_account_info, &associated_token_program_account_info,
                &token_program_account_info, &beneficiary_account_info, &beneficiary_account_info,
                &beneficiary_token_account_info, &associated_token_mint_info
            )?;
        }
        // Withdraw
        let _ = transfer_from_treasury(
            &msp_account_info, &token_program_account_info, &treasury_account_info,
            &treasury_token_account_info, &beneficiary_token_account_info, withdraw_request_amount
        )?;
        // Update stream data
        let _ = post_withdrawal_update_stream(
            &mut stream, &stream_account_info, &associated_token_mint_info,
            &clock, withdrawable_amount, withdraw_request_amount
        )?;
        // Update treasury account data
        let _ = withdraw_funds_update_treasury(
            &treasury_account_info, &associated_token_mint_info, withdraw_request_amount
        )?;

        if fee_treasury_token_account_info.data_len() == 0 { // Create fee treasury associated token account if doesn't exist
            let _ = create_ata_account(
                &system_account_info, &rent_account_info, &associated_token_program_account_info,
                &token_program_account_info, &beneficiary_account_info, &fee_treasury_account_info,
                &fee_treasury_token_account_info, &associated_token_mint_info
            )?;
        }
        
        let fee = WITHDRAW_PERCENT_FEE * withdraw_request_amount as f64 / 100f64;
        // Pay fees
        transfer_token_fee(
            &token_program_account_info,
            &beneficiary_token_account_info,
            &fee_treasury_token_account_info,
            &beneficiary_account_info,
            fee as u64
        )
    }

    fn process_pause_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let _ = check_can_pause_or_resume_stream(
            program_id, &initializer_account_info, &treasury_account_info,
            &associated_token_mint_info, &stream_account_info, &msp_account_info
        )?;

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let mut escrow_vested_amount = get_beneficiary_withdrawable_amount(
            &stream, &clock, associated_token_mint.decimals.into()
        )?;
        let current_slot = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let stream_allocation_left = (stream.allocation_left * pow) as u64;
        
        if escrow_vested_amount > stream_allocation_left {
            escrow_vested_amount = stream_allocation_left;
        }

        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let stream_rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;

        if treasury.depletion_rate >= stream_rate {
            let treasury_depletion_rate = ((treasury.depletion_rate * pow) as u64)
                .checked_sub((stream_rate * pow) as u64)
                .ok_or(StreamError::Overflow)? as f64 / pow;
                
            treasury.depletion_rate = treasury_depletion_rate;
        }

        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_slot = current_slot;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
        StreamV1::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_resume_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let _ = check_can_pause_or_resume_stream(
            program_id, &initializer_account_info, &treasury_account_info,
            &associated_token_mint_info, &stream_account_info, &msp_account_info
        )?;

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let stream_rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;
        
        if treasury.depletion_rate >= stream_rate {
            let treasury_depletion_rate = ((treasury.depletion_rate * pow) as u64)
                .checked_sub((stream_rate * pow) as u64)
                .ok_or(StreamError::Overflow)? as f64 / pow;
                
            treasury.depletion_rate = treasury_depletion_rate;
        }
    
        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());
        // Resuming the stream and updating data
        stream.stream_resumed_slot = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
        // Save
        StreamV1::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_close_stream(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        auto_close_treasury: bool

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let initializer_account_info = next_account_info(account_info_iter)?;
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasurer_token_account_info = next_account_info(account_info_iter)?;
        let treasurer_treasury_pool_token_account_info = next_account_info(account_info_iter)?;
        let beneficiary_account_info = next_account_info(account_info_iter)?;
        let beneficiary_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;  
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let treasury_pool_mint_info = next_account_info(account_info_iter)?;
        let stream_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let associated_token_program_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if stream_account_info.data_len() == Stream::LEN {
            return close_stream_v0(
                program_id, &initializer_account_info, &treasurer_account_info,
                &treasurer_token_account_info, &treasurer_treasury_pool_token_account_info,
                &beneficiary_account_info, &beneficiary_token_account_info, &associated_token_mint_info,
                &treasury_account_info, &treasury_token_account_info, &treasury_pool_mint_info,
                &stream_account_info, &fee_treasury_account_info, &fee_treasury_token_account_info,
                &msp_account_info, &associated_token_program_account_info, &token_program_account_info,
                &rent_account_info, &system_account_info, auto_close_treasury,
            );
        }

        let _ = check_can_close_stream(
            program_id, &initializer_account_info, &treasurer_account_info,
            &treasurer_token_account_info, &beneficiary_account_info, &beneficiary_token_account_info,
            &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
            &treasury_pool_mint_info, &stream_account_info, &fee_treasury_token_account_info,
            &msp_account_info, &associated_token_program_account_info, &token_program_account_info,
            &rent_account_info, &system_account_info
        )?;

        let clock = Clock::get()?;
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;        
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;  
        let mut escrow_vested_amount = get_beneficiary_withdrawable_amount(
            &stream, &clock, associated_token_mint.decimals.into()
        )?;
        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

        if stream.allocation_left < 0f64 { // TODO: Remove (this is temp)
            stream.allocation_left = 0f64;
        }

        let stream_allocation_left = (stream.allocation_left * pow) as u64;

        if escrow_vested_amount > stream_allocation_left {
            escrow_vested_amount = stream_allocation_left;
        }
        
        if escrow_vested_amount > treasury_token.amount { // TODO: Remove (this is temp)
            escrow_vested_amount = treasury_token.amount;
        }
        // Pausing the stream
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_slot = clock.slot as u64;
        stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;

        if escrow_vested_amount > 0u64 { // Transfer vested amount to beneficiary and deduct fee
            let _ = close_stream_transfer_vested_amount(
                &initializer_account_info, &treasury_account_info, &treasury_token_account_info,
                &beneficiary_account_info, &beneficiary_token_account_info, &associated_token_mint_info,
                &fee_treasury_account_info, &fee_treasury_token_account_info, &msp_account_info,
                &associated_token_program_account_info, &token_program_account_info, &rent_account_info,
                &system_account_info, escrow_vested_amount
            )?;
        }

        let mut escrow_unvested_amount = 0;
        let stream_allocation_left = (stream.allocation_left * pow) as u64;

        if escrow_vested_amount > 0u64 && escrow_vested_amount <= stream_allocation_left {
            escrow_unvested_amount = stream_allocation_left
                .checked_sub(escrow_vested_amount)
                .ok_or(StreamError::Overflow)?;
        }
        
        let _ = close_stream_update_treasury(
            &mut treasury, &stream, &associated_token_mint_info,
            escrow_vested_amount, escrow_unvested_amount
        )?;
        // Save
        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

        if treasury.auto_close == true && auto_close_treasury == true && stream.treasurer_address.eq(initializer_account_info.key) {
            let _ = close_stream_close_treasury(
                program_id, &treasurer_account_info, &treasurer_token_account_info,
                &treasurer_treasury_pool_token_account_info, &associated_token_mint_info,
                &treasury_account_info, &treasury_token_account_info, &treasury_pool_mint_info,
                &fee_treasury_account_info, &fee_treasury_token_account_info, 
                &msp_account_info, &token_program_account_info
            )?;
        }
        // Debit fees from the initializer of the instruction
        let _ = transfer_sol_fee(
            &system_account_info, &initializer_account_info,
            &fee_treasury_account_info, CLOSE_STREAM_FLAT_FEE
        );
        // Close stream account
        let treasurer_lamports = treasurer_account_info.lamports();
        let stream_lamports = stream_account_info.lamports();

        **stream_account_info.lamports.borrow_mut() = 0;
        **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
            .checked_add(stream_lamports)
            .ok_or(StreamError::Overflow)?;

        Ok(())
    }

    fn process_create_treasury(
        accounts: &[AccountInfo], 
        program_id: &Pubkey,
        slot: u64,
        label: String,
        treasury_type: u8,
        auto_close: bool,

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_pool_token_mint_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let clock = Clock::get()?;

        if msp_account_info.key.ne(program_id) {
            return Err(StreamError::IncorrectProgramId.into());
        }

        if !treasurer_account_info.is_signer {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        // Create Treasury PDA
        let (treasury_address, bump_seed) = Pubkey::find_program_address(
            &[
                treasurer_account_info.key.as_ref(), 
                &slot.to_le_bytes()
            ], 
            msp_account_info.key
        );
    
        if treasury_address.ne(treasury_account_info.key) {
            return Err(StreamError::InvalidTreasuryPoolMint.into());
        }

        let treasury_pool_signer_seed: &[&[_]] = &[
            treasurer_account_info.key.as_ref(),
            &slot.to_le_bytes(),
            &bump_seed.to_le_bytes()
        ];

        let _ = create_pda_account(
            &system_account_info, &rent_account_info, &msp_account_info,
            &treasury_account_info, &treasurer_account_info,
            TreasuryV1::LEN, &[treasury_pool_signer_seed]
        );
        // Create Treasury Pool Mint PDA
        let (treasury_pool_mint_address, bump_seed) = Pubkey::find_program_address(
            &[
                treasurer_account_info.key.as_ref(), 
                treasury_account_info.key.as_ref(), 
                &slot.to_le_bytes()
            ], 
            msp_account_info.key
        );
    
        if treasury_pool_mint_address.ne(treasury_pool_token_mint_info.key) {
            return Err(StreamError::InvalidTreasuryPoolMint.into());
        }

        let treasury_pool_mint_signer_seed: &[&[_]] = &[
            treasurer_account_info.key.as_ref(),
            treasury_account_info.key.as_ref(),
            &slot.to_le_bytes(),
            &bump_seed.to_le_bytes()
        ];

        let _ = create_pda_account(
            &system_account_info, &rent_account_info, &token_program_account_info,
            &treasury_pool_token_mint_info, &treasurer_account_info,
            spl_token::state::Mint::LEN, &[treasury_pool_mint_signer_seed]
        );
        // Initialize pool treasury mint
        let init_treasury_pool_mint_ix = spl_token::instruction::initialize_mint(
            token_program_account_info.key, treasury_pool_token_mint_info.key,
            treasury_account_info.key, None, TREASURY_POOL_MINT_DECIMALS
        )?;

        let _ = invoke(&init_treasury_pool_mint_ix, &[
            token_program_account_info.clone(),
            treasury_pool_token_mint_info.clone(),
            treasury_account_info.clone(),
            rent_account_info.clone()
        ]);

        // Update Treasury data
        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

        treasury.slot = slot;
        treasury.treasurer_address = *treasurer_account_info.key;
        treasury.mint_address = *treasury_pool_token_mint_info.key;
        treasury.label = label;
        treasury.balance = 0.0;
        treasury.allocation_reserved = 0.0;
        treasury.allocation_left = 0.0;
        treasury.allocation_assigned = 0.0;
        treasury.streams_amount = 0;
        treasury.created_on_utc = clock.unix_timestamp as u64 * 1000u64;
        treasury.depletion_rate = 0.0;
        treasury.treasury_type = treasury_type;
        treasury.auto_close = auto_close;
        treasury.initialized = true;
        // Save
        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

        // Debit fees from treasurer
        transfer_sol_fee(
            &system_account_info,
            &treasurer_account_info,
            &fee_treasury_account_info,
            CREATE_TREASURY_FLAT_FEE
        )
    }

    fn process_close_treasury(
        accounts: &[AccountInfo],
        program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasurer_token_account_info = next_account_info(account_info_iter)?;
        let treasurer_treasury_pool_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;  
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let treasury_pool_mint_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN { // close treasury
            return close_treasury_v0(
                program_id, &treasurer_account_info, &treasurer_token_account_info,
                &treasurer_treasury_pool_token_account_info, &associated_token_mint_info,
                &treasury_account_info, &treasury_token_account_info, &treasury_pool_mint_info,
                &fee_treasury_token_account_info, &msp_account_info, &token_program_account_info
            );
        }

        let _ = check_can_close_treasury(
            &program_id, &treasurer_account_info,
            &treasury_account_info, &msp_account_info,
            &token_program_account_info
        )?;

        let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        // Close treaurer treasury pool token account
        let _ = close_treasury_pool_token_account(
            &treasury, &treasurer_account_info, &treasurer_treasury_pool_token_account_info,
            &treasury_account_info, &treasury_pool_mint_info, &msp_account_info, &token_program_account_info
        )?;

        if treasury.associated_token_address.eq(associated_token_mint_info.key) {
            let _ = close_treasury_token_account(
                &treasury, &treasurer_account_info, &treasurer_token_account_info,
                &treasury_account_info, &treasury_token_account_info,
                &msp_account_info, &token_program_account_info,
            )?;
        }
        // Close treasury account
        let treasurer_lamports = treasurer_account_info.lamports();
        let treasury_lamports = treasury_account_info.lamports();

        **treasury_account_info.lamports.borrow_mut() = 0;
        **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
            .checked_add(treasury_lamports)
            .ok_or(StreamError::Overflow)?;

        Ok(())
    }
}

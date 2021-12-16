// Program

use num_traits;
use solana_program::{
    msg,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    entrypoint::ProgramResult,
    account_info::{ next_account_info, AccountInfo },
    program_pack::{ IsInitialized, Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar } 
};

use crate::error::StreamError;
use crate::utils::*;
use crate::instruction::{ StreamInstruction };
use crate::state::*;
use crate::constants::*;
use crate::account_validations::*;

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
                allocation,
                funded_on_utc,
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
                    stream_name,
                    rate_amount,
                    rate_interval_in_seconds,
                    allocation_reserved,
                    allocation,
                    funded_on_utc,
                    start_utc,
                    rate_cliff_in_seconds,
                    cliff_vest_amount,
                    cliff_vest_percent,
                    auto_pause_in_seconds
                )
            },

            StreamInstruction::AddFunds { 
                amount,
                allocation_type,
                allocation_stream_address

            } => {
                msg!("Instruction: AddFunds");

                Self::process_add_funds(
                    accounts, 
                    program_id,
                    amount,
                    allocation_type,
                    allocation_stream_address
                )
            },

            StreamInstruction::RecoverFunds { amount } => {
                msg!("Instruction: RecoverFunds");

                Self::process_recover_funds(
                    accounts, 
                    program_id,
                    amount
                )
            },

            StreamInstruction::Withdraw { amount } => {

                msg!("Instruction: Withdraw");

                Self::process_withdraw(
                    accounts, 
                    program_id, 
                    amount
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

            StreamInstruction::CloseStream { auto_close_treasury } => {
                msg!("Instruction: CloseStream");

                Self::process_close_stream(
                    accounts, 
                    program_id,
                    auto_close_treasury
                )
            },

            StreamInstruction::CreateTreasury { 
                slot,
                label,
                treasury_type

            } => {

                msg!("Instruction: CreateTreasury");

                Self::process_create_treasury(
                    accounts, 
                    program_id,
                    slot,
                    label,
                    treasury_type
                )
            },

            StreamInstruction::CloseTreasury => {
                msg!("Instruction: CloseTreasury");

                Self::process_close_treasury(
                    accounts, 
                    program_id
                )
            },

            StreamInstruction::UpgradeTreasury => {
                msg!("Instruction: UpgradeTreasury");

                Self::process_upgrade_treasury(
                    accounts, 
                    program_id
                )
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
        allocation: f64,
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
        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Deserialize treasury
        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let _ = check_can_create_stream(
            program_id,
            &treasurer_account_info,
            &treasury_account_info,
            &associated_token_mint_info,
            &stream_account_info,
            &msp_account_info,
            &system_account_info,
            &rent_account_info,
            allocation,
            allocation_reserved
        )?;
        
        // Create stream account
        let _ = create_stream_account(
            &treasurer_account_info,
            &stream_account_info,
            &msp_account_info,
            &rent_account_info,
            &system_account_info
        )?;

        let clock = Clock::get()?;
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

        // Updating stream data
        stream.stream_name = stream_name;
        stream.treasurer_address = *treasurer_account_info.key;
        stream.rate_amount = rate_amount;
        stream.rate_interval_in_seconds = rate_interval_in_seconds;
        stream.allocation_reserved = allocation_reserved;
        stream.allocation = allocation;
        stream.funded_on_utc = funded_on_utc;
        stream.start_utc = start_utc;
        stream.rate_cliff_in_seconds = rate_cliff_in_seconds;
        stream.cliff_vest_amount = cliff_vest_amount;
        stream.cliff_vest_percent = cliff_vest_percent;
        stream.beneficiary_address = *beneficiary_account_info.key;
        stream.beneficiary_associated_token = *associated_token_mint_info.key;
        stream.treasury_address = *treasury_account_info.key;
        stream.treasury_estimated_depletion_utc = 0;
        stream.escrow_vested_amount_snap = 0.0;
        stream.escrow_vested_amount_snap_slot = clock.slot as u64;
        stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;
        stream.stream_resumed_slot = clock.slot;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
        stream.auto_pause_in_seconds = auto_pause_in_seconds;

        let status = get_stream_status(&stream, &clock)?;

        if status == StreamStatus::Scheduled
        {
            stream.stream_resumed_block_time = start_utc / 1000u64;
        }

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let _ = create_stream_update_treasury(&mut treasury, &stream, associated_token_mint.decimals.into())?;
        
        // Save treasury
        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());
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
        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN
        {
            return add_funds_v0(
                program_id,
                msp_account_info,
                fee_treasury_account_info,
                associated_token_program_account_info,
                token_program_account_info,
                system_account_info,
                rent_account_info,
                contributor_account_info,
                contributor_token_account_info,
                contributor_treasury_pool_token_account_info,
                treasury_account_info,
                treasury_token_account_info,
                associated_token_mint_info,   
                treasury_pool_mint_info,
                stream_account_info,               
                amount
            );
        }

        let _ = check_can_add_funds(
            program_id,
            &msp_account_info,
            &contributor_account_info,
            &contributor_token_account_info,
            &contributor_treasury_pool_token_account_info,
            &associated_token_mint_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &treasury_pool_mint_info,
            &stream_account_info,
            &associated_token_program_account_info,
            &token_program_account_info,
            &rent_account_info,
            &system_account_info
        )?;

        // Create contributor deposit receipt
        let _ = create_deposit_receipt(
            &treasury_account_info,
            &treasury_pool_mint_info,
            &contributor_treasury_pool_token_account_info,
            &msp_account_info,
            &token_program_account_info,
            amount
        )?;

        // Transfer tokens from contributor to treasury associated token account
        let _ = transfer_tokens(
            &contributor_account_info,
            &contributor_token_account_info,
            &treasury_token_account_info,
            &associated_token_mint_info,
            &token_program_account_info,
            amount
        )?;

        // Update and save treasury
        let _ = add_funds_update_treasury(
            &treasury_account_info,
            &associated_token_mint_info,
            allocation_type,
            amount
        )?;

        if stream_account_info.data_len() == StreamV1::LEN
        {
            let clock = Clock::get()?;
            let _ = add_funds_update_stream(
                &stream_account_info,
                &associated_token_mint_info,
                &clock,
                &allocation_stream_address,
                allocation_type,
                amount
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

    fn process_recover_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        amount: f64

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let contributor_account_info = next_account_info(account_info_iter)?;
        let contributor_token_account_info = next_account_info(account_info_iter)?;
        let contributor_treasury_pool_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let treasury_pool_mint_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let fee_treasury_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;

        if msp_account_info.key.ne(program_id)
        {
            return Err(StreamError::IncorrectProgramId.into());
        }

        if !contributor_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }
        
        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap()) ||
           treasury_account_info.owner != program_id
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        // Get contributor treasury pool token account
        let contributor_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
            contributor_account_info.key,
            treasury_pool_mint_info.key
        );

        if contributor_treasury_pool_token_address.ne(contributor_treasury_pool_token_account_info.key) 
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let treasury_pool_mint = spl_token::state::Mint::unpack_from_slice(&treasury_pool_mint_info.data.borrow())?;
        let treasury_pool_mint_pow = num_traits::pow(10u64, treasury_pool_mint.decimals.into());

        let burn_amount = (amount as u64)
            .checked_mul(treasury_pool_mint_pow)
            .ok_or(StreamError::Overflow)?;

        // Burn treasury tokens from the contributor treasury token account       
        let burn_ix = spl_token::instruction::burn(
            token_program_account_info.key,
            contributor_treasury_pool_token_account_info.key,
            treasury_pool_mint_info.key,
            contributor_account_info.key,
            &[],
            burn_amount
        )?;

        let _ = invoke(&burn_ix, &[
            token_program_account_info.clone(),
            contributor_treasury_pool_token_account_info.clone(),
            treasury_pool_mint_info.clone(),
            contributor_account_info.clone()
        ]);

        // Transfer tokens to contributor
        // The percent that represents the `amount` in the pool  
        let recover_amount_percent = (amount as u64)
            .checked_mul(treasury_pool_mint.supply)
            .unwrap()
            .checked_div(100u64)
            .ok_or(StreamError::Overflow)? as f64;   
        
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let associated_token_mint_pow = num_traits::pow(10u64, associated_token_mint.decimals.into());
        let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

        let recover_amount = (recover_amount_percent as u64)
            .checked_mul(
                (treasury.balance as u64)
                    .checked_sub(treasury.allocation_reserved as u64)
                    .ok_or(StreamError::Overflow)?
            )
            .unwrap()
            .checked_div(100u64)
            .unwrap()
            .checked_mul(associated_token_mint_pow)
            .ok_or(StreamError::Overflow)?;
        
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasurer_address.as_ref(),
                &treasury.slot.to_le_bytes()
            ], 
            msp_account_info.key
        );

        if treasury_pool_address.ne(treasury_account_info.key)
        {
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_signer_seed: &[&[_]] = &[
            treasury.treasurer_address.as_ref(),
            &treasury.slot.to_le_bytes(),
            &[treasury_pool_bump_seed]
        ];

        let contributor_transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            contributor_token_account_info.key,
            treasury_account_info.key,
            &[],
            recover_amount
        )?;

        let _ = invoke_signed(&contributor_transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                contributor_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_signer_seed]
        );

        let fee = (WITHDRAW_PERCENT_FEE as u64)
            .checked_mul(amount as u64)
            .unwrap()
            .checked_div(100u64)
            .unwrap()
            .checked_mul(associated_token_mint_pow)
            .ok_or(StreamError::Overflow)?;

        // Pay fees
        transfer_token_fee(
            &token_program_account_info,
            &contributor_token_account_info,
            &fee_treasury_token_account_info,
            &contributor_account_info,
            fee
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

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN && stream_account_info.data_len() == Stream::LEN
        {
            return withdraw_v0(
                program_id,
                beneficiary_account_info,
                beneficiary_token_account_info,
                associated_token_mint_info,
                treasury_account_info,
                treasury_token_account_info,
                stream_account_info,
                fee_treasury_account_info,
                fee_treasury_token_account_info,
                msp_account_info,
                associated_token_program_account_info,
                token_program_account_info,
                rent_account_info,
                system_account_info,
                &clock,
                amount
            );
        }

        let _ = check_can_withdraw_funds(
            program_id,
            &beneficiary_account_info,
            &beneficiary_token_account_info,
            &associated_token_mint_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &stream_account_info,
            &fee_treasury_token_account_info,
            &msp_account_info,
            &associated_token_program_account_info,
            &token_program_account_info,
            &rent_account_info,
            &system_account_info
        )?;

        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;        
        let mut escrow_vested_amount = get_stream_vested_amount(
            &stream, 
            &clock, 
            associated_token_mint.decimals.into()
        )?;

        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;
        let stream_allocation = (stream.allocation * pow) as u64;

        if stream_allocation > 0 && escrow_vested_amount > stream_allocation
        {
            escrow_vested_amount = stream_allocation;
        }
        else if escrow_vested_amount > treasury_token.amount
        {
            escrow_vested_amount = treasury_token.amount;
        }

        let transfer_amount = (amount * pow) as u64;

        if transfer_amount > escrow_vested_amount
        {
            return Err(StreamError::NotAllowedWithdrawalAmount.into());
        }

        if beneficiary_token_account_info.data_len() == 0
        {
            // Create treasury associated token account if doesn't exist
            let _ = create_ata_account(
                &system_account_info,
                &rent_account_info,
                &associated_token_program_account_info,
                &token_program_account_info,
                &beneficiary_account_info,
                &beneficiary_account_info,
                &beneficiary_token_account_info,
                &associated_token_mint_info
            )?;
        }

        // Withdraw
        let _ = claim_treasury_funds(
            &msp_account_info,
            &token_program_account_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &beneficiary_token_account_info,
            transfer_amount
        )?;
 
        // Update stream data
        let _ = withdraw_funds_update_stream(
            &mut stream,
            &stream_account_info,
            &associated_token_mint_info,
            &clock,
            escrow_vested_amount,
            transfer_amount
        )?;

        // Update treasury account data
        let _ = withdraw_funds_update_treasury(
            &treasury_account_info,
            &associated_token_mint_info,
            transfer_amount
        )?;

        if fee_treasury_token_account_info.data_len() == 0
        {
            // Create treasury associated token account if doesn't exist
            let _ = create_ata_account(
                &system_account_info,
                &rent_account_info,
                &associated_token_program_account_info,
                &token_program_account_info,
                &beneficiary_account_info,
                &fee_treasury_account_info,
                &fee_treasury_token_account_info,
                &associated_token_mint_info
            )?;
        }
        
        let fee = WITHDRAW_PERCENT_FEE * transfer_amount as f64 / 100f64;
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

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let _ = check_can_pause_stream(
            program_id,
            &initializer_account_info,
            &stream_account_info,
            &msp_account_info
        )?;

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let mut escrow_vested_amount = get_stream_vested_amount(
            &stream, 
            &clock, 
            associated_token_mint.decimals.into()
        )?;

        let current_slot = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let stream_allocation = (stream.allocation * pow) as u64;
        
        if escrow_vested_amount > stream_allocation
        {
            escrow_vested_amount = stream_allocation;
        }

        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let stream_rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;

        if treasury.depletion_rate >= stream_rate
        {
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

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let _ = check_can_resume_stream(
            program_id,
            &initializer_account_info,
            &stream_account_info,
            &msp_account_info
        )?;

        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let stream_rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;
        
        if treasury.depletion_rate >= stream_rate
        {
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
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
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
            let funding_amount = (stream.total_deposits as u64)
                .checked_sub(stream.total_withdrawals as u64)
                .ok_or(StreamError::Overflow)?;

            stream_terms.auto_pause_in_seconds = funding_amount
                .checked_mul(rate_interval_in_seconds)
                .unwrap()
                .checked_div(rate_amount as u64)
                .ok_or(StreamError::Overflow)?;
        }

        stream_terms.initialized = true;
        // Save
        StreamTerms::pack_into_slice(&stream_terms, &mut stream_terms_account_info.data.borrow_mut());

        // Debit fees from the initializer of the instruction
        transfer_sol_fee(
            &system_account_info,
            &initializer_account_info,
            &fee_treasury_account_info,
            PROPOSE_UPDATE_FLAT_FEE
        )
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
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
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
        transfer_sol_fee(
            &system_account_info,
            &initializer_account_info,
            &fee_treasury_account_info,
            PROPOSE_UPDATE_FLAT_FEE
        )
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

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if stream_account_info.data_len() == Stream::LEN 
        {
            return close_stream_v0(
                &msp_account_info,
                &fee_treasury_account_info,
                &fee_treasury_token_account_info,
                &token_program_account_info,
                &system_account_info,
                &initializer_account_info,
                &treasurer_account_info,
                &treasurer_token_account_info,
                &treasurer_treasury_pool_token_account_info,
                &beneficiary_token_account_info,
                &associated_token_mint_info,
                &treasury_account_info,
                &treasury_token_account_info,
                &treasury_pool_mint_info,
                &stream_account_info,
                auto_close_treasury,
            );
        }

        let _ = check_can_close_stream(
            program_id,
            &initializer_account_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &beneficiary_token_account_info,
            &associated_token_mint_info,
            &stream_account_info,
            &fee_treasury_token_account_info,
            &msp_account_info
        )?;

        let clock = Clock::get()?;
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        
        let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;  
        let mut escrow_vested_amount = get_stream_vested_amount(
            &stream,
            &clock,
            associated_token_mint.decimals.into()
        )?;

        let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

        if stream.allocation < 0f64 // TODO: Remove (this is temp)
        {
            stream.allocation = 0f64;
        }

        let stream_allocation = (stream.allocation * pow) as u64;

        if escrow_vested_amount > stream_allocation
        {
            escrow_vested_amount = stream_allocation;
        }
        
        if escrow_vested_amount > treasury_token.amount
        {
            escrow_vested_amount = treasury_token.amount;
        }

        // Pausing the stream
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_slot = clock.slot as u64;
        stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;

        if escrow_vested_amount > 0u64
        {
            // Transfer vested amount to beneficiary and deduct fee
            let _ = close_stream_transfer_vested_amount(
                &initializer_account_info,
                &treasury_account_info,
                &treasury_token_account_info,
                &beneficiary_account_info,
                &beneficiary_token_account_info,
                &associated_token_mint_info,
                &fee_treasury_account_info,
                &fee_treasury_token_account_info,
                &msp_account_info,
                &associated_token_program_account_info,
                &token_program_account_info,
                &rent_account_info,
                &system_account_info,
                escrow_vested_amount
            )?;
        }

        let escrow_unvested_amount = ((stream.allocation * pow) as u64)
            .checked_sub(escrow_vested_amount)
            .ok_or(StreamError::Overflow)?;

        let _ = close_stream_update_treasury(
            &mut treasury,
            &stream,
            &associated_token_mint_info,
            escrow_vested_amount,
            escrow_unvested_amount
        )?;

        // Save
        TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

        if auto_close_treasury == true && stream.treasurer_address.eq(initializer_account_info.key)
        {
            let _ = close_stream_close_treasury(
                program_id,
                &treasurer_account_info,
                &treasurer_token_account_info,
                &treasurer_treasury_pool_token_account_info,
                &associated_token_mint_info,
                &treasury_account_info,
                &treasury_token_account_info,
                &treasury_pool_mint_info,
                &fee_treasury_account_info,
                &fee_treasury_token_account_info,
                &msp_account_info,
                &token_program_account_info
            )?;
        }

        // Debit fees from the initializer of the instruction
        let _ = transfer_sol_fee(
            &system_account_info,
            &initializer_account_info,
            &fee_treasury_account_info,
            CLOSE_STREAM_FLAT_FEE
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
        treasury_type: u8

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

        if msp_account_info.key.ne(program_id)
        {
            return Err(StreamError::IncorrectProgramId.into());
        }

        if !treasurer_account_info.is_signer
        {
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
    
        if treasury_address.ne(treasury_account_info.key)
        {
            return Err(StreamError::InvalidTreasuryPoolMint.into());
        }

        let treasury_pool_signer_seed: &[&[_]] = &[
            treasurer_account_info.key.as_ref(),
            &slot.to_le_bytes(),
            &bump_seed.to_le_bytes()
        ];

        let _ = create_pda_account(
            &system_account_info,
            &rent_account_info,
            &msp_account_info,
            &treasury_account_info,
            &treasurer_account_info,
            TreasuryV1::LEN,
            &[treasury_pool_signer_seed]
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
    
        if treasury_pool_mint_address.ne(treasury_pool_token_mint_info.key)
        {
            return Err(StreamError::InvalidTreasuryPoolMint.into());
        }

        let treasury_pool_mint_signer_seed: &[&[_]] = &[
            treasurer_account_info.key.as_ref(),
            treasury_account_info.key.as_ref(),
            &slot.to_le_bytes(),
            &bump_seed.to_le_bytes()
        ];

        let _ = create_pda_account(
            &system_account_info,
            &rent_account_info,
            &token_program_account_info,
            &treasury_pool_token_mint_info,
            &treasurer_account_info,
            spl_token::state::Mint::LEN,
            &[treasury_pool_mint_signer_seed]
        );

        // Initialize pool treasury mint
        let init_treasury_pool_mint_ix = spl_token::instruction::initialize_mint(
            token_program_account_info.key,
            treasury_pool_token_mint_info.key,
            treasury_account_info.key,
            None,
            TREASURY_POOL_MINT_DECIMALS
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
        treasury.allocation = 0.0;
        treasury.streams_amount = 0;
        treasury.created_on_utc = clock.unix_timestamp as u64 * 1000u64;
        treasury.depletion_rate = 0.0;
        treasury.treasury_type = treasury_type;
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
        let _fee_treasury_token_account_info = next_account_info(account_info_iter)?;
        let msp_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        if treasury_account_info.data_len() == Treasury::LEN
        {
            // close treasury
            return close_treasury_v0(
                msp_account_info,
                token_program_account_info,
                treasurer_account_info,
                treasurer_token_account_info,
                treasurer_treasury_pool_token_account_info,
                treasury_account_info,
                treasury_token_account_info,
                treasury_pool_mint_info
            );
        }

        let _ = check_can_close_treasury(
            &program_id,
            &treasurer_account_info,
            &treasury_account_info,
            &msp_account_info
        )?;

        let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
        // Close treaurer treasury pool token account
        let _ = close_treasury_pool_token_account(
            &treasury,
            &treasurer_account_info,
            &treasurer_treasury_pool_token_account_info,
            &treasury_account_info,
            &treasury_pool_mint_info,
            &msp_account_info,
            &token_program_account_info
        )?;

        if treasury.associated_token_address.eq(associated_token_mint_info.key)
        {
            let _ = close_treasury_token_account(
                &treasury,
                &treasurer_account_info,
                &treasurer_token_account_info,
                &treasury_account_info,
                &treasury_token_account_info,
                &msp_account_info,
                &token_program_account_info,
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

    fn process_upgrade_treasury(
        accounts: &[AccountInfo],
        _program_id: &Pubkey

    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();
        let treasurer_account_info = next_account_info(account_info_iter)?;
        let treasury_account_info = next_account_info(account_info_iter)?;
        let treasury_token_account_info = next_account_info(account_info_iter)?;
        let associated_token_mint_info = next_account_info(account_info_iter)?;
        let fee_treasury_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_account_info)?;
        // let clock = Clock::get()?;

        if !treasurer_account_info.is_signer
        {
            return Err(StreamError::MissingInstructionSignature.into());
        }

        if fee_treasury_account_info.key.ne(&FEE_TREASURY_ACCOUNT.parse().unwrap())
        {
            return Err(StreamError::InstructionNotAuthorized.into());
        }

        let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
        let mut streams_amount = 0;
        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;
        let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
        let associated_token_mint_pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
        
        if treasury_token.amount > 0
        {
            streams_amount = 1;
        }

        let new_treasury_data: &[u8; TreasuryV1::LEN] = &[0; TreasuryV1::LEN];
        let mut new_treasury = TreasuryV1::unpack_from_slice(new_treasury_data)?;
            
        // Cleaning data
        new_treasury.balance = treasury_token.amount as f64 / associated_token_mint_pow;
        new_treasury.allocation = treasury_token.amount as f64 / associated_token_mint_pow;
        new_treasury.allocation_reserved = 0.0;
        new_treasury.streams_amount = streams_amount;
        new_treasury.slot = treasury.treasury_block_height;
        new_treasury.treasurer_address = treasury.treasury_base_address;
        new_treasury.associated_token_address = *associated_token_mint_info.key;
        new_treasury.mint_address = treasury.treasury_mint_address;
        new_treasury.created_on_utc = 0;
        new_treasury.label = String::default();
        new_treasury.initialized = treasury.initialized;

        // Save
        TreasuryV1::pack_into_slice(&new_treasury, &mut treasury_account_info.data.borrow_mut());
        
        let new_treasury_balance = rent.minimum_balance(TreasuryV1::LEN);
        // Update treasury rent excempt balance 
        let treasurer_lamports = treasurer_account_info.lamports();
        let treasury_lamports = treasury_account_info.lamports();

        **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
            .checked_add(treasury_lamports)
            .ok_or(StreamError::Overflow)?;

        **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
            .checked_sub(new_treasury_balance)            
            .ok_or(StreamError::Overflow)?;
        
        **treasury_account_info.lamports.borrow_mut() = treasury_lamports
            .checked_add(new_treasury_balance)
            .ok_or(StreamError::Overflow)?;

        **treasury_account_info.lamports.borrow_mut() = treasury_lamports
            .checked_sub(treasury_lamports)            
            .ok_or(StreamError::Overflow)?;

        Ok(())
    }
}

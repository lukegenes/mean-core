use std::cmp;
use num_traits;
use std::{ convert::TryInto };
use crate::error::StreamError;
use crate::state::*;
use crate::constants::*;
use crate::utils::*;
use crate::extensions::*;
use crate::account_validations::*;
use solana_program::{
    // msg,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_pack::{ Pack },
    sysvar::{ clock::Clock, Sysvar } 
};

pub fn claim_treasury_funds_v0<'info>(
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    destination_account_info: &AccountInfo<'info>,
    amount: u64

) -> ProgramResult {

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address.ne(treasury_account_info.key) {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    let treasury_pool_signer_seed: &[&[_]] = &[
        treasury.treasury_base_address.as_ref(),
        &treasury.treasury_block_height.to_le_bytes(),
        &treasury_pool_bump_seed.to_le_bytes()
    ];

    let transfer_ix = spl_token::instruction::transfer(
        token_program_account_info.key,
        treasury_token_account_info.key,
        destination_account_info.key,
        treasury_account_info.key,
        &[],
        amount
    )?;

    let _ = invoke_signed(&transfer_ix, 
        &[
            treasury_account_info.clone(),
            treasury_token_account_info.clone(),
            destination_account_info.clone(),
            token_program_account_info.clone(),
            msp_account_info.clone()
        ],
        &[treasury_pool_signer_seed]
    );

    Ok(())
}

pub fn get_stream_status_v0<'info>(
    stream: &Stream,
    clock: &Clock

) -> Result<StreamStatus, StreamError> {

    let now = clock.unix_timestamp as u64 * 1000u64;

    if stream.start_utc > now {
        return Ok(StreamStatus::Scheduled);
    }

    if stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time {
        return Ok(StreamStatus::Running);
    }

    return Ok(StreamStatus::Paused);
}

pub fn get_beneficiary_withdrawable_amount_v0<'info>(
    stream: &Stream,
    clock: &Clock,
    decimals: u64

) -> Result<u64, StreamError> {

    let status = get_stream_status_v0(stream, clock)?;

    //Check if SCHEDULED
    if status == StreamStatus::Scheduled{
        return Ok(0);
    }

    //Check if PAUSED
    let pow = num_traits::pow(10f64, decimals.try_into().unwrap());
    let allocation_left = ((stream.total_deposits * pow) as u64)
        .checked_sub((stream.total_withdrawals * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    if status == StreamStatus::Paused {
        let is_manual_pause = stream.escrow_vested_amount_snap_block_time > stream.stream_resumed_block_time;
        let paused_withdrawable = match is_manual_pause {
            true => (stream.escrow_vested_amount_snap * pow) as u64,
            _ => allocation_left
        };
        return Ok(paused_withdrawable);
    }

    //Check if RUNNING
    if stream.rate_interval_in_seconds <= 0 || stream.rate_amount <= 0.0 {
        return Err(StreamError::InvalidArgument.into());
    }
    let rate_amount_per_second = stream.rate_amount / (stream.rate_interval_in_seconds as f64);
    let block_time_at_last_snap_or_resume = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time_since_last_snap_or_resume = (clock.unix_timestamp as u64)
                                                .checked_sub(block_time_at_last_snap_or_resume)
                                                .ok_or(StreamError::Overflow)?;
    let vested_amount_since_last_snap_or_resume = rate_amount_per_second * elapsed_time_since_last_snap_or_resume as f64; 
    let allocation_left_vested_amount = ((stream.escrow_vested_amount_snap * pow) as u64)
                                        .checked_add((vested_amount_since_last_snap_or_resume * pow) as u64)
                                        .ok_or(StreamError::Overflow)?;
    let stream_allocation_left = ((stream.total_deposits * pow) as u64)
        .checked_sub((stream.total_withdrawals * pow) as u64)
        .ok_or(StreamError::Overflow)?;
    let withdrawable = cmp::min(stream_allocation_left, allocation_left_vested_amount);
    return Ok(withdrawable);
}

pub fn add_funds_v0<'info>(
    program_id: &Pubkey,
    msp_account_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    contributor_account_info: &AccountInfo<'info>,
    contributor_token_account_info: &AccountInfo<'info>,
    contributor_treasury_pool_token_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,   
    treasury_pool_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,    
    amount: f64

) -> ProgramResult {

    // Validate add funds accounts
    let _ = check_can_add_funds_v0(
        program_id, &msp_account_info, &contributor_account_info,
        &contributor_token_account_info, &contributor_treasury_pool_token_account_info,
        &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
        &treasury_pool_mint_info, &stream_account_info, &associated_token_program_account_info,
        &token_program_account_info, &rent_account_info, &system_account_info
    )?;
    // Create contributor deposit receipt
    let _ = create_deposit_receipt_v0(
        &treasury_account_info, &treasury_pool_mint_info,
        &contributor_treasury_pool_token_account_info,  &msp_account_info,
        &token_program_account_info, amount
    )?;
    // Transfer tokens from contributor to treasury associated token account
    let _ = transfer_tokens(
        &contributor_account_info, &contributor_token_account_info,
        &treasury_token_account_info, &associated_token_mint_info,
        &token_program_account_info, amount
    )?;

    if stream_account_info.data_len() == Stream::LEN {
        let clock = Clock::get()?;
        let _ = add_funds_update_stream_v0(
            &stream_account_info, &associated_token_mint_info, &clock, amount
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

pub fn withdraw_v0<'info>(
    program_id: &Pubkey,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    clock: &Clock,
    amount: f64

) -> ProgramResult {

    let _ = check_can_withdraw_funds_v0(
        program_id, &beneficiary_account_info, &beneficiary_token_account_info,
        &associated_token_mint_info, &treasury_account_info,
        &treasury_token_account_info, &stream_account_info,
        &fee_treasury_token_account_info, &msp_account_info,
        &associated_token_program_account_info, &token_program_account_info,
        &rent_account_info, &system_account_info
    )?;

    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;        
    let mut escrow_vested_amount = get_beneficiary_withdrawable_amount_v0(
        &stream, &clock, associated_token_mint.decimals.into()
    )?;

    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;
    let stream_allocation = ((stream.total_deposits * pow) as u64)
        .checked_sub((stream.total_withdrawals * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    if stream_allocation > 0 && escrow_vested_amount > stream_allocation {
        escrow_vested_amount = stream_allocation;
    } else if escrow_vested_amount > treasury_token.amount {
        escrow_vested_amount = treasury_token.amount;
    }

    let transfer_amount = (amount * pow) as u64;

    if transfer_amount > escrow_vested_amount {
        return Err(StreamError::NotAllowedWithdrawalAmount.into());
    }

    if beneficiary_token_account_info.data_len() == 0 { // Create beneficiary associated token account if doesn't exist
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &beneficiary_account_info, &beneficiary_account_info,
            &beneficiary_token_account_info, &associated_token_mint_info
        )?;
    }
    // Withdraw
    let _ = claim_treasury_funds_v0(
        &msp_account_info, &token_program_account_info, &treasury_account_info, 
        &treasury_token_account_info, &beneficiary_token_account_info, transfer_amount
    )?;
    // Update stream data
    let _ = withdraw_funds_update_stream_v0(
        &mut stream, &stream_account_info, &associated_token_mint_info, 
        &clock, escrow_vested_amount, transfer_amount
    )?;

    if fee_treasury_token_account_info.data_len() == 0 { // Create treasury associated token account if doesn't exist
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &beneficiary_account_info, &fee_treasury_account_info,
            &fee_treasury_token_account_info, &associated_token_mint_info
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

pub fn close_treasury_v0<'info>(
    program_id: &Pubkey,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,

) -> ProgramResult {

    let _ = check_can_close_treasury_v0(
        program_id, &treasurer_account_info, &treasurer_token_account_info,
        &treasurer_treasury_pool_token_account_info, &associated_token_mint_info,
        &treasury_account_info, &treasury_token_account_info, &treasury_pool_mint_info,
        &fee_treasury_token_account_info, &msp_account_info, &token_program_account_info
    )?;

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasurer_treasury_pool_token_account_info.data_len() == spl_token::state::Account::LEN {
        let treasurer_treasury_pool_token = spl_token::state::Account::unpack_from_slice(
            &treasurer_treasury_pool_token_account_info.data.borrow()
        )?;    
        // Burn treasury tokens from the contributor treasury token account       
        let burn_ix = spl_token::instruction::burn(
            token_program_account_info.key, treasurer_treasury_pool_token_account_info.key,
            treasury_pool_mint_info.key, treasurer_account_info.key, &[], treasurer_treasury_pool_token.amount
        )?;
    
        let _ = invoke(&burn_ix, &[
            token_program_account_info.clone(), treasurer_treasury_pool_token_account_info.clone(),
            treasury_pool_mint_info.clone(), treasurer_account_info.clone()
        ]);
        // Close treasurer treasury pool token account
        let treasurer_treasury_pool_token_close_ix = spl_token::instruction::close_account(
            token_program_account_info.key, treasurer_treasury_pool_token_account_info.key, 
            treasurer_account_info.key, treasurer_account_info.key, &[]
        )?;
    
        let _ = invoke(&treasurer_treasury_pool_token_close_ix, &[
            treasurer_treasury_pool_token_account_info.clone(),
            treasurer_account_info.clone(), token_program_account_info.clone(),
        ]);
    }

    if treasury_token_account_info.data_len() == spl_token::state::Account::LEN {
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes()
            ], 
            msp_account_info.key
        );
    
        if treasury_pool_address.ne(treasury_account_info.key) {
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_pool_signer_seed: &[&[_]] = &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes(),
            &treasury_pool_bump_seed.to_le_bytes()
        ];

        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

        if treasury_token.amount > 0 { // Credit all treasury token amount to treasurer
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key, treasury_token_account_info.key,
                treasurer_token_account_info.key, treasury_account_info.key, &[], treasury_token.amount
            )?;
        
            let _ = invoke_signed(&transfer_ix, &[
                treasury_account_info.clone(), treasury_token_account_info.clone(),
                treasurer_token_account_info.clone(), token_program_account_info.clone(),
                msp_account_info.clone()], &[treasury_pool_signer_seed]
            );
        }
        // Close treasury token account
        let close_token_ix = spl_token::instruction::close_account(
            token_program_account_info.key, treasury_token_account_info.key, 
            treasurer_account_info.key, treasury_account_info.key, &[]
        )?;

        let _ = invoke_signed(&close_token_ix, &[
            treasury_account_info.clone(), treasury_token_account_info.clone(),
            treasurer_account_info.clone(), token_program_account_info.clone()], 
            &[treasury_pool_signer_seed]
        );
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

pub fn close_stream_v0<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>, 
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    close_treasury: bool

) -> ProgramResult {

    let _ = check_can_close_stream_v0(
        program_id, &initializer_account_info, &treasurer_account_info,
        &treasurer_token_account_info, &beneficiary_account_info, &beneficiary_token_account_info,
        &associated_token_mint_info, &treasury_account_info, &treasury_token_account_info,
        &treasury_pool_mint_info, &stream_account_info, &fee_treasury_token_account_info,
        &msp_account_info, &associated_token_program_account_info, &token_program_account_info,
        &rent_account_info, &system_account_info
    )?;

    let clock = Clock::get()?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;    
    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;  
    let mut escrow_vested_amount = get_beneficiary_withdrawable_amount_v0(
        &stream, &clock, associated_token_mint.decimals.into()
    )?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

    if stream.total_deposits < 0.0 {
        stream.total_deposits = 0.0;
    }

    if stream.total_withdrawals < 0.0 {
        stream.total_withdrawals = 0.0;
    }

    let mut stream_allocation = 0;

    if stream.total_deposits >= stream.total_withdrawals {
        stream_allocation = ((stream.total_deposits * pow) as u64)
            .checked_sub((stream.total_withdrawals * pow) as u64)
            .ok_or(StreamError::Overflow)?;
    }

    if escrow_vested_amount > stream_allocation {
        escrow_vested_amount = stream_allocation;
    }

    if escrow_vested_amount > treasury_token.amount {
        escrow_vested_amount = treasury_token.amount;
    }

    // Pausing the stream
    stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
    stream.escrow_vested_amount_snap_block_height = clock.slot as u64;
    stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;

    if escrow_vested_amount > 0u64 { // Transfer vested amount to beneficiary and deduct fee
        let _ = close_stream_transfer_vested_amount_v0(
            &initializer_account_info, &treasury_account_info, &treasury_token_account_info,
            &beneficiary_account_info, &beneficiary_token_account_info, &associated_token_mint_info,
            &fee_treasury_account_info, &fee_treasury_token_account_info, &msp_account_info,
            &associated_token_program_account_info, &token_program_account_info,
            &rent_account_info, &system_account_info, escrow_vested_amount
        )?;
    }
    // Debit fees from the initializer of the instruction
    let _ = transfer_sol_fee(
        &system_account_info, &initializer_account_info,
        &fee_treasury_account_info, CLOSE_STREAM_FLAT_FEE
    );

    if close_treasury == true && stream.treasurer_address.eq(initializer_account_info.key) {
        let _ = close_treasury_v0(
            program_id, &treasurer_account_info, &treasurer_token_account_info,
            &treasurer_treasury_pool_token_account_info, &associated_token_mint_info,
            &treasury_account_info, &treasury_token_account_info, &treasury_pool_mint_info,
            &fee_treasury_token_account_info, &msp_account_info, &token_program_account_info,
        )?;
    }    
    // Close stream account
    let treasurer_lamports = treasurer_account_info.lamports();
    let stream_lamports = stream_account_info.lamports();

    **stream_account_info.lamports.borrow_mut() = 0;
    **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
        .checked_add(stream_lamports)
        .ok_or(StreamError::Overflow)?;

    Ok(())
}

pub fn create_deposit_receipt_v0<'info>(
    treasury_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    dest_pool_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (_, treasury_pool_bump_seed) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );
    
    // Mint just if there is a treasury pool
    let treasury_pool_mint = spl_token::state::Mint::unpack_from_slice(&treasury_pool_mint_info.data.borrow())?;
    let treasury_pool_mint_signer_seed: &[&[_]] = &[
        treasury.treasury_base_address.as_ref(),
        &treasury.treasury_block_height.to_le_bytes(),
        &[treasury_pool_bump_seed]
    ];

    let pow = num_traits::pow(10f64, treasury_pool_mint.decimals.into());
    let mint_amount = (amount * pow) as u64;

    let mint_to_ix = spl_token::instruction::mint_to(
        token_program_account_info.key,
        treasury_pool_mint_info.key,
        dest_pool_token_account_info.key,
        treasury_account_info.key,
        &[],
        mint_amount
    )?;

    invoke_signed(&mint_to_ix,
        &[
            token_program_account_info.clone(),
            treasury_pool_mint_info.clone(),
            dest_pool_token_account_info.clone(),
            treasury_account_info.clone()
        ],
        &[treasury_pool_mint_signer_seed]
    )
}

pub fn add_funds_update_stream_v0<'info>(
    stream_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    clock: &Clock,
    amount: f64

) -> ProgramResult {

    let current_block_height = clock.slot as u64;
    let current_block_time = clock.unix_timestamp as u64;
    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;

    let escrow_vested_amount = get_beneficiary_withdrawable_amount_v0(
        &stream, &clock, associated_token_mint.decimals.try_into().unwrap()
    )?;

    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let max_vested_amount = ((stream.total_deposits * pow) as u64)
        .checked_sub((stream.total_withdrawals * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    // Pause because the allocation amount was reached
    if escrow_vested_amount == max_vested_amount
    {
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
    }

    stream.total_deposits = ((stream.total_deposits * pow) as u64)
        .checked_add((amount * pow) as u64)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    // if it was paused before because of lack of money then resume it again 
    if escrow_vested_amount == max_vested_amount
    {
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
    }

    Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

    Ok(())
}

pub fn withdraw_funds_update_stream_v0<'info>(
    stream: &mut Stream,
    stream_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    clock: &Clock,
    vested_amount: u64,
    transfer_amount: u64

) -> ProgramResult {

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());

    // Update stream account data
    let escrow_vested_amount_snap = vested_amount
        .checked_sub(transfer_amount)
        .ok_or(StreamError::Overflow)?;

    stream.escrow_vested_amount_snap = escrow_vested_amount_snap as f64 / pow;
    let status = get_stream_status_v0(stream, clock)?;

    if status == StreamStatus::Running {
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
    }

    stream.total_withdrawals = ((stream.total_withdrawals * pow) as u64)
        .checked_add(transfer_amount)
        .ok_or(StreamError::Overflow)? as f64 / pow;    

    // Save
    Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

    Ok(())
}

pub fn close_stream_transfer_vested_amount_v0<'info>(
    initializer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    vested_amount: u64

) -> ProgramResult {

    if beneficiary_token_account_info.data_len() == 0 {
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &initializer_account_info, &beneficiary_account_info,
            &beneficiary_token_account_info,&associated_token_mint_info
        );
    }

    let fee = (CLOSE_STREAM_PERCENT_FEE * vested_amount as f64 / 100f64) as u64;
    let transfer_amount = vested_amount
        .checked_sub(fee)
        .ok_or(StreamError::Overflow)?;

    // Credit vested amount minus fee to the beneficiary
    let _ = claim_treasury_funds_v0(
        &msp_account_info, &token_program_account_info, &treasury_account_info,
        &treasury_token_account_info, &beneficiary_token_account_info, transfer_amount
    )?;

    if fee_treasury_token_account_info.data_len() == 0 { // Create treasury associated token account if doesn't exist
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &initializer_account_info, &fee_treasury_account_info,
            &fee_treasury_token_account_info, &associated_token_mint_info
        )?;
    }

    // Pay fee by the beneficiary from the vested amount
    claim_treasury_funds_v0(
        &msp_account_info,
        &token_program_account_info,
        &treasury_account_info,
        &treasury_token_account_info,
        &fee_treasury_token_account_info,
        fee
    )
}
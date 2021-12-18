use std::cmp;
use num_traits;
use std::{ string::String, convert::TryInto };
use crate::error::StreamError;
use crate::state::*;
use crate::constants::*;
use solana_program::{
    // msg,
    system_program,
    system_instruction,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_pack::{ Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar } 
};

pub fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), StreamError> {
    if input.len() >= 32 {
        let (key, rest) = input.split_at(32);
        let pk = Pubkey::new(key);
        Ok((pk, rest))
    } else {
        Err(StreamError::InvalidArgument.into())
    }
}

pub fn unpack_string(input: &[u8]) -> Result<(String, &[u8]), StreamError> {
    if input.len() >= 32 {
        let (bytes, rest) = input.split_at(32);
        Ok((String::from_utf8_lossy(bytes).to_string(), rest))
    } else {
        Err(StreamError::InvalidArgument.into())
    }
}

pub fn unpack_u64(input: &[u8]) -> Result<u64, StreamError> {
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(u64::from_le_bytes)
        .ok_or(StreamError::InvalidStreamInstruction)?;

    Ok(amount)
}

pub fn unpack_f64(input: &[u8]) -> Result<f64, StreamError> {
    let amount = input
        .get(..8)
        .and_then(|slice| slice.try_into().ok())
        .map(f64::from_le_bytes)
        .ok_or(StreamError::InvalidStreamInstruction)?;

    Ok(amount)
}

pub fn unpack_u8(input: &[u8]) -> Result<u8, StreamError> {
    let amount = input
        .get(..1)
        .and_then(|slice| slice.try_into().ok())
        .map(u8::from_le_bytes)
        .ok_or(StreamError::InvalidStreamInstruction)?;

    Ok(amount)
}

pub fn create_pda_account<'info>(
    system_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    program_account_info: &AccountInfo<'info>,
    pda_account_info: &AccountInfo<'info>,
    base_account_info: &AccountInfo<'info>,
    pda_size: usize,
    pda_signer_seed: &[&[&[u8]]]

) -> ProgramResult {

    let rent = &Rent::from_account_info(rent_account_info)?;
    let pda_balance = rent.minimum_balance(pda_size);
    let create_pda_ix = system_instruction::create_account(
        base_account_info.key,
        pda_account_info.key,
        pda_balance,
        u64::from_le_bytes(pda_size.to_le_bytes()),
        program_account_info.key
    );

    invoke_signed(&create_pda_ix, 
        &[
            base_account_info.clone(),
            pda_account_info.clone(),
            program_account_info.clone(),
            system_account_info.clone()
        ], 
        pda_signer_seed
    )
}

pub fn create_ata_account<'info>(
    system_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    payer_account_info: &AccountInfo<'info>,
    owner_account_info: &AccountInfo<'info>,
    owner_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>

) -> ProgramResult {

    let create_account_ix = spl_associated_token_account::create_associated_token_account(
        payer_account_info.key,
        owner_account_info.key,
        associated_token_mint_info.key
    );

    let _ = invoke(&create_account_ix, &[
        associated_token_program_account_info.clone(),
        payer_account_info.clone(),
        owner_token_account_info.clone(),
        owner_account_info.clone(),
        associated_token_mint_info.clone(),
        system_account_info.clone(),
        token_program_account_info.clone(),
        rent_account_info.clone()
    ]);

    Ok(())
}

pub fn claim_treasury_funds<'info>(
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    destination_account_info: &AccountInfo<'info>,
    amount: u64

) -> ProgramResult {

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address.ne(treasury_account_info.key) {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    let treasury_pool_signer_seed: &[&[_]] = &[
        treasury.treasurer_address.as_ref(),
        &treasury.slot.to_le_bytes(),
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

pub fn transfer_sol_fee<'info>(
    system_account_info: &AccountInfo<'info>,
    payer_account_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let lamports = amount * LAMPORTS_PER_SOL as f64;
    let pay_fee_ix = system_instruction::transfer(
        payer_account_info.key,
        fee_treasury_account_info.key,
        lamports as u64
    );

    invoke(&pay_fee_ix, &[
        payer_account_info.clone(),
        fee_treasury_account_info.clone(),
        system_account_info.clone()
    ])
}

pub fn transfer_token_fee<'info>(
    token_program_account_info: &AccountInfo<'info>,
    payer_token_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    payer_authority_account_info: &AccountInfo<'info>,
    amount: u64

) -> ProgramResult {

    let fees_ix = spl_token::instruction::transfer(
        token_program_account_info.key,
        payer_token_account_info.key,
        fee_treasury_token_account_info.key,
        payer_authority_account_info.key,
        &[],
        amount
    )?;

    invoke(&fees_ix, &[
        payer_authority_account_info.clone(),
        payer_token_account_info.clone(),
        fee_treasury_token_account_info.clone(),
        token_program_account_info.clone()
    ])
}

pub fn get_stream_status<'info>(
    stream: &StreamV1,
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

pub fn get_beneficiary_withdrawable_amount<'info>( //rename to get_beneficiary_withdrawable_amount 
    stream: &StreamV1,
    clock: &Clock,
    decimals: u64

) -> Result<u64, StreamError> {

    let status = get_stream_status(stream, clock)?;

    if status == StreamStatus::Scheduled{
        return Ok(0);
    }

    let pow = num_traits::pow(10f64, decimals.try_into().unwrap());

    if status == StreamStatus::Paused{
        let is_manual_pause = stream.escrow_vested_amount_snap_block_time > stream.stream_resumed_block_time;

        let paused_withdrawable = match is_manual_pause {
            true => (stream.escrow_vested_amount_snap * pow) as u64,
            _ => (stream.allocation_left * pow) as u64
        };
        
        return Ok(paused_withdrawable);
    }

    //Else this stream is running, do the hard calculation (capped by the Allocation Left)
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

    let stream_allocation_left = (stream.allocation_left * pow) as u64;

    let withdrawable = match stream_allocation_left < allocation_left_vested_amount{
            true => stream_allocation_left,
            _ => allocation_left_vested_amount
    };

    return Ok(withdrawable);
}

pub fn get_stream_vested_amount<'info>(
    stream: &StreamV1,
    clock: &Clock,
    decimals: u64

) -> Result<u64, StreamError> {

    let status = get_stream_status(stream, clock)?;

    if status == StreamStatus::Scheduled{
        return Ok(0);
    }

    let is_running = match status {
        k if k == StreamStatus::Running => 1,
        _ => 0
    };

    let rate = match stream.rate_interval_in_seconds {
        k if k > 0 => stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64),
        _ => stream.allocation_left
    };

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = (clock.unix_timestamp as u64)
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;    
    let pow = num_traits::pow(10f64, decimals.try_into().unwrap());
    let stream_allocation_left = (stream.allocation_left * pow) as u64;
    let mut escrow_vested_amount = ((stream.escrow_vested_amount_snap * pow) as u64)
        .checked_add((rate_time * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    if escrow_vested_amount > stream_allocation_left {
        escrow_vested_amount = stream_allocation_left;
    }

    return Ok(escrow_vested_amount);
}

pub fn check_system_accounts<'info>(
    associated_token_program_account: Option<&AccountInfo<'info>>,
    token_program_account: Option<&AccountInfo<'info>>,
    rent_account: Option<&AccountInfo<'info>>,
    system_account: Option<&AccountInfo<'info>>

) -> ProgramResult {

    // Check associated token program account info
    if let Some(associated_token_program_account_info) = associated_token_program_account {
        if associated_token_program_account_info.key.ne(&spl_associated_token_account::id()) {
            return Err(StreamError::IncorrectProgramId.into());
        }
    }
    // Check token program account info
    if let Some(token_program_account_info) = token_program_account {
        if token_program_account_info.key.ne(&spl_token::id()) {
            return Err(StreamError::IncorrectProgramId.into());
        }
    }
    // Check rent program account info
    if let Some(rent_account_info) = rent_account {
        if rent_account_info.key.ne(&solana_program::sysvar::rent::id()) {
            return Err(StreamError::IncorrectProgramId.into());
        }
    }
    // Check system program account info
    if let Some(system_account_info) = system_account {
        if system_account_info.key.ne(&system_program::id()){
            return Err(StreamError::IncorrectProgramId.into());
        }
    }

    Ok(())
}
use num_traits;
use std::{ convert::TryInto };
use crate::instruction::*;
use crate::error::StreamError;
use crate::state::*;
use crate::constants::*;
use crate::utils::*;
use solana_program::{
    // msg,
    system_instruction,
    program::{ invoke, invoke_signed },
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_pack::{ Pack },
    sysvar::{ clock::Clock, rent::Rent, Sysvar } 
};

pub fn create_stream_account<'info>(
    treasurer_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    let rent = &Rent::from_account_info(rent_account_info)?;
    // Create stream account
    let stream_balance = rent.minimum_balance(StreamV1::LEN);
    let create_stream_ix = system_instruction::create_account(
        treasurer_account_info.key,
        stream_account_info.key,
        stream_balance,
        u64::from_le_bytes(StreamV1::LEN.to_le_bytes()),
        msp_account_info.key
    );

    invoke(&create_stream_ix, &[
        treasurer_account_info.clone(),
        stream_account_info.clone(),
        msp_account_info.clone(),
        system_account_info.clone()
    ])
} 

pub fn create_stream_update_treasury<'info>(
    treasury_account_info: &AccountInfo<'info>,
    stream: &StreamV1,
    decimals: usize

) -> ProgramResult {

    let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let pow = num_traits::pow(10f64, decimals.into());
    let rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;
    let depletion_rate = ((treasury.depletion_rate * pow) as u64)
        .checked_add((rate * pow) as u64)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    treasury.depletion_rate = depletion_rate;        
    treasury.streams_amount = treasury.streams_amount.checked_add(1).ok_or(StreamError::Overflow)?;

    if stream.allocation_assigned > 0.0 {
        treasury.allocation_assigned = ((treasury.allocation_assigned * pow) as u64)
            .checked_add((stream.allocation_assigned * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        treasury.allocation_left = ((treasury.allocation_left * pow) as u64)
            .checked_add((stream.allocation_left * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    if stream.allocation_reserved > 0.0 {
        treasury.allocation_reserved = ((treasury.allocation_reserved * pow) as u64)
            .checked_add((stream.allocation_reserved * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    // Save treasury
    TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

    Ok(())
}

pub fn create_deposit_receipt<'info>(
    treasury_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    dest_pool_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (_, treasury_pool_bump_seed) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );    
    // Mint just if there is a treasury pool
    let treasury_pool_mint = spl_token::state::Mint::unpack_from_slice(&treasury_pool_mint_info.data.borrow())?;
    let treasury_pool_mint_signer_seed: &[&[_]] = &[
        treasury.treasurer_address.as_ref(),
        &treasury.slot.to_le_bytes(),
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

pub fn add_funds_update_treasury<'info>(
    treasury_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    allocation_type: u8,
    amount: f64

) -> ProgramResult {

    let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let balance = ((treasury.balance * pow) as u64)
        .checked_add((amount * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    treasury.balance = balance as f64 / pow;

    if allocation_type == 0 {
        treasury.allocation_assigned = ((treasury.allocation_assigned * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
        
        treasury.allocation_left = ((treasury.allocation_left * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

    } else if allocation_type == 1 {   
        treasury.allocation_assigned = ((treasury.allocation_assigned * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        treasury.allocation_left = ((treasury.allocation_left * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        treasury.allocation_reserved = ((treasury.allocation_reserved * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    treasury.associated_token_address = *associated_token_mint_info.key;
    // Save
    TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

    Ok(())
}

pub fn add_funds_update_stream<'info>(
    stream_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    clock: &Clock,
    allocation_stream_address: &Pubkey,
    allocation_type: u8,
    amount: f64

) -> ProgramResult {

    let current_slot = clock.slot as u64;
    let current_block_time = clock.unix_timestamp as u64;
    let mut stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let escrow_vested_amount = get_stream_vested_amount(
        &stream, &clock, associated_token_mint.decimals.try_into().unwrap()
    )?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let allocation_assigned = (stream.allocation_assigned * pow) as u64;
    // Pause because the allocation amount was reached
    if escrow_vested_amount > allocation_assigned {
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_slot = current_slot;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
    }

    if allocation_type == 1 && allocation_stream_address.ne(&Pubkey::default()) && 
       stream_account_info.key.eq(&allocation_stream_address)
    {
        stream.allocation_assigned = allocation_assigned
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        stream.allocation_left = ((stream.allocation_left * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        stream.allocation_reserved = ((stream.allocation_reserved * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }
    // if it was paused before because of lack of money then resume it again 
    if escrow_vested_amount > allocation_assigned {
        stream.stream_resumed_slot = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
    }

    StreamV1::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

    Ok(())
}

pub fn transfer_tokens<'info>(
    source_owner_account_info: &AccountInfo<'info>,
    source_token_account_info: &AccountInfo<'info>,
    dest_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let transfer_amount = (amount * pow) as u64;

    let transfer_ix = spl_token::instruction::transfer(
        token_program_account_info.key,
        source_token_account_info.key,
        dest_token_account_info.key,
        source_owner_account_info.key,
        &[],
        transfer_amount
    )?;

    invoke(&transfer_ix, &[
        source_owner_account_info.clone(),
        dest_token_account_info.clone(),
        source_token_account_info.clone(),
        token_program_account_info.clone()
    ])
}

pub fn withdraw_funds_update_stream<'info>(
    stream: &mut StreamV1,
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
    stream.escrow_vested_amount_snap_slot = clock.slot as u64;
    stream.escrow_vested_amount_snap_block_time = clock.unix_timestamp as u64;

    let stream_allocation_left = (stream.allocation_left * pow) as u64;

    if escrow_vested_amount_snap < stream_allocation_left {
        stream.stream_resumed_slot = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
    }

    stream.allocation_left = stream_allocation_left
        .checked_sub(transfer_amount)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    let stream_allocation_reserved = (stream.allocation_reserved * pow) as u64;

    if stream_allocation_reserved >= transfer_amount {
        stream.allocation_reserved = stream_allocation_reserved
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }
    // Save
    StreamV1::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

    Ok(())
}

pub fn withdraw_funds_update_treasury<'info>(
    treasury_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    transfer_amount: u64

) -> ProgramResult {

    let mut treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let treasury_allocation_left = (treasury.allocation_left * pow) as u64;

    if treasury_allocation_left >= transfer_amount {
        treasury.allocation_left = treasury_allocation_left
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    let treasury_allocation_reserved = (treasury.allocation_reserved * pow) as u64;

    if treasury_allocation_reserved >= transfer_amount {
        treasury.allocation_reserved = treasury_allocation_reserved
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    let treasury_balance = (treasury.balance * pow) as u64;

    if treasury_balance >= transfer_amount {
        treasury.balance = treasury_balance
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }
    // Save
    TreasuryV1::pack_into_slice(&treasury, &mut treasury_account_info.data.borrow_mut());

    Ok(())
}

pub fn close_stream_transfer_vested_amount<'info>(
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
            &beneficiary_token_account_info, &associated_token_mint_info
        );
    }

    let fee = (CLOSE_STREAM_PERCENT_FEE * vested_amount as f64 / 100f64) as u64;
    let transfer_amount = vested_amount.checked_sub(fee).ok_or(StreamError::Overflow)?;
    // Credit vested amount minus fee to the beneficiary
    let _ = claim_treasury_funds(
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
    claim_treasury_funds(
        &msp_account_info, &token_program_account_info, &treasury_account_info,
        &treasury_token_account_info, &fee_treasury_token_account_info, fee
    )
}

pub fn close_stream_update_treasury<'info>(
    treasury: &mut TreasuryV1,
    stream: &StreamV1,
    associated_token_mint_info: &AccountInfo<'info>,
    vested_amount: u64,
    unvested_amount: u64

) -> ProgramResult {

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());    
    let treasury_balance = (treasury.balance * pow) as u64;

    treasury.balance = treasury_balance
        .checked_sub(vested_amount).unwrap().checked_sub(unvested_amount)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    let treasury_allocation_left = (treasury.allocation_left * pow) as u64;

    treasury.allocation_left = treasury_allocation_left
        .checked_sub(vested_amount).unwrap().checked_sub(unvested_amount)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    let treasury_allocation_reserved = (treasury.allocation_reserved * pow) as u64;

    if treasury_allocation_reserved >= vested_amount {
        treasury.allocation_reserved = treasury_allocation_reserved
            .checked_sub(vested_amount).unwrap().checked_sub(unvested_amount)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    let stream_rate = match stream.rate_interval_in_seconds {
        k if k > 0 => stream.rate_amount / (stream.rate_interval_in_seconds as f64),
        _ => 0.0
    };

    if treasury.depletion_rate >= stream_rate {
        treasury.depletion_rate = ((treasury.depletion_rate * pow) as u64)
            .checked_sub((stream_rate * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    treasury.streams_amount = treasury.streams_amount.checked_sub(1).ok_or(StreamError::Overflow)?;

    Ok(())
}

pub fn close_stream_close_treasury<'info>(
    program_id: &Pubkey,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    fee_treasury_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>

) -> ProgramResult {

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    
    if treasury.streams_amount > 0 {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

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

    let close_treasury_ix = close_treasury(
        *treasurer_account_info.key,
        *treasurer_token_account_info.key,
        *treasurer_treasury_pool_token_account_info.key,
        *associated_token_mint_info.key,
        *treasury_account_info.key,
        *treasury_token_account_info.key,
        *treasury_pool_mint_info.key,
        *fee_treasury_account_info.key,
        *fee_treasury_token_account_info.key,
        *token_program_account_info.key,
        program_id
    )?;

    invoke_signed(&close_treasury_ix, 
        &[
            treasurer_account_info.clone(),
            treasurer_token_account_info.clone(),
            treasurer_treasury_pool_token_account_info.clone(),
            associated_token_mint_info.clone(),
            treasury_account_info.clone(),
            treasury_token_account_info.clone(),
            treasury_pool_mint_info.clone(),
            fee_treasury_account_info.clone(),
            fee_treasury_token_account_info.clone(),
            token_program_account_info.clone(),
            msp_account_info.clone()
        ],
        &[treasury_pool_signer_seed]
    )
}

pub fn close_treasury_pool_token_account<'info>(
    treasury: &TreasuryV1,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>

) -> ProgramResult {

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

    let treasury_pool_signer_seed: &[&[_]] = &[
        treasury.treasurer_address.as_ref(),
        &treasury.slot.to_le_bytes(),
        &treasury_pool_bump_seed.to_le_bytes()
    ];

    if treasurer_treasury_pool_token_account_info.data_len() == spl_token::state::Account::LEN
    {
        let treasurer_treasury_pool_token = spl_token::state::Account::unpack_from_slice(
            &treasurer_treasury_pool_token_account_info.data.borrow()
        )?;
    
        // Burn treasury tokens from the contributor treasury token account       
        let burn_ix = spl_token::instruction::burn(
            token_program_account_info.key,
            treasurer_treasury_pool_token_account_info.key,
            treasury_pool_mint_info.key,
            treasurer_account_info.key,
            &[],
            treasurer_treasury_pool_token.amount
        )?;
    
        let _ = invoke(&burn_ix, &[
            token_program_account_info.clone(),
            treasurer_treasury_pool_token_account_info.clone(),
            treasury_pool_mint_info.clone(),
            treasurer_account_info.clone()
        ]);
    
        // Close treasurer treasury pool token account
        let treasurer_treasury_pool_token_close_ix = spl_token::instruction::close_account(
            token_program_account_info.key, 
            treasurer_treasury_pool_token_account_info.key, 
            treasurer_account_info.key, 
            treasurer_account_info.key,
            &[]
        )?;
    
        let _ = invoke_signed(&treasurer_treasury_pool_token_close_ix, 
            &[
                treasurer_treasury_pool_token_account_info.clone(),
                treasurer_account_info.clone(),
                token_program_account_info.clone(),
            ],
            &[treasury_pool_signer_seed]
        );
    }

    Ok(())
}

pub fn close_treasury_token_account<'info>(
    treasury: &TreasuryV1,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,

) -> ProgramResult {

    let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

    if treasury_token.amount > 0
    {
        // Credit all treasury token amount to treasurer
        let _ = claim_treasury_funds(
            &msp_account_info,
            &token_program_account_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &treasurer_token_account_info,
            treasury_token.amount
        );      
    }

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

    let treasury_pool_signer_seed: &[&[_]] = &[
        treasury.treasurer_address.as_ref(),
        &treasury.slot.to_le_bytes(),
        &treasury_pool_bump_seed.to_le_bytes()
    ];

    // Close treasury token account
    let close_token_ix = spl_token::instruction::close_account(
        token_program_account_info.key, 
        treasury_token_account_info.key, 
        treasurer_account_info.key, 
        treasury_account_info.key, 
        &[]
    )?;

    invoke_signed(&close_token_ix, 
        &[
            treasury_account_info.clone(),
            treasury_token_account_info.clone(),
            treasurer_account_info.clone(),
            token_program_account_info.clone(),
        ],
        &[treasury_pool_signer_seed]
    )
}
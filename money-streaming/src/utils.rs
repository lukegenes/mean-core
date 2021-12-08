
use std::cmp;
use num_traits;
use std::{ string::String, convert::TryInto };
use crate::error::StreamError;
use crate::state::{ Treasury, TreasuryV1, Stream };
use crate::constants::{
    ADD_FUNDS_FLAT_FEE,
    CLOSE_STREAM_FLAT_FEE,
    CLOSE_STREAM_PERCENT_FEE,
    WITHDRAW_PERCENT_FEE,
    LAMPORTS_PER_SOL
};
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

    if treasury_pool_address.ne(treasury_account_info.key)
    {
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

pub fn add_funds_v0<'info>(
    msp_account_info: &AccountInfo<'info>,
    msp_ops_account_info: &AccountInfo<'info>,
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
    amount: f64,
    resume: bool

) -> ProgramResult {

    let clock = Clock::get()?;
    // Check is the stream needs to be paused because of lacks of funds
    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    let current_block_height = clock.slot as u64;
    let current_block_time = clock.unix_timestamp as u64;
    let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
    let mut rate = 0.0;
    
    if stream.rate_interval_in_seconds > 0
    {
        rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
    }

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = current_block_time
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let mut escrow_vested_amount = (stream.escrow_vested_amount_snap as u64)
        .checked_add(
            (rate as u64)
              .checked_mul(elapsed_time)
              .ok_or(StreamError::Overflow)?

        ).ok_or(StreamError::Overflow)? as f64;

    let no_funds = (escrow_vested_amount >= (stream.total_deposits as u64)
        .checked_sub(stream.total_withdrawals as u64)
        .ok_or(StreamError::Overflow)? as f64) as u64;

    // Pause if no funds
    if no_funds == 1
    {
        escrow_vested_amount = (stream.total_deposits as u64)
            .checked_sub(stream.total_withdrawals as u64)
            .ok_or(StreamError::Overflow)? as f64;

        stream.escrow_vested_amount_snap = escrow_vested_amount;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
    }

    // Create treasury associated token account if doesn't exist
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        treasury_account_info.key,
        associated_token_mint_info.key
    );

    if treasury_token_address != *treasury_token_account_info.key 
    {
        return Err(StreamError::InvalidTreasuryAccount.into());
    }

    if (*treasury_token_account_info.owner).ne(token_program_account_info.key)
    {
        // Create treasury associated token account if doesn't exist
        let _ = create_ata_account(
            &system_account_info,
            &rent_account_info,
            &associated_token_program_account_info,
            &token_program_account_info,
            &contributor_account_info,
            &treasury_account_info,
            &treasury_token_account_info,
            &associated_token_mint_info
        );
    }

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
        return Err(StreamError::InvalidTreasuryPool.into());
    }

    if (*contributor_treasury_pool_token_account_info.key).ne(&Pubkey::default()) &&
        (*treasury_pool_mint_info.key).ne(&Pubkey::default())
    {
        if (*contributor_treasury_pool_token_account_info.owner).ne(token_program_account_info.key)
        {
            // Create contributor treasury associated token account
            let contributor_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
                contributor_account_info.key,
                treasury_pool_mint_info.key
            );

            if contributor_treasury_pool_token_address != *contributor_treasury_pool_token_account_info.key 
            {
                return Err(StreamError::InvalidTreasuryPoolAddress.into());
            }

            // Create the contributor treasury token account if there is a treasury pool and the account does not exists
            let _ = create_ata_account(
                &system_account_info,
                &rent_account_info,
                &associated_token_program_account_info,
                &token_program_account_info,
                &contributor_account_info,
                &contributor_account_info,
                &contributor_treasury_pool_token_account_info,
                &treasury_pool_mint_info
            );
        }
        
        // Mint just if there is a treasury pool
        let treasury_pool_mint = spl_token::state::Mint::unpack_from_slice(&treasury_pool_mint_info.data.borrow())?;
        let treasury_pool_mint_signer_seed: &[&[_]] = &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes(),
            &[treasury_pool_bump_seed]
        ];

        let treasury_pool_mint_pow = num_traits::pow(10f64, treasury_pool_mint.decimals.into());    
        let mint_to_ix = spl_token::instruction::mint_to(
            token_program_account_info.key,
            treasury_pool_mint_info.key,
            contributor_treasury_pool_token_account_info.key,
            treasury_account_info.key,
            &[],
            (amount * treasury_pool_mint_pow) as u64
        )?;

        let _ = invoke_signed(&mint_to_ix,
            &[
                token_program_account_info.clone(),
                treasury_pool_mint_info.clone(),
                contributor_treasury_pool_token_account_info.clone(),
                treasury_account_info.clone()
            ],
            &[treasury_pool_mint_signer_seed]
        )?;
    }

    // Transfer tokens from contributor to treasury pool
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let associated_token_mint_pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let transfer_ix = spl_token::instruction::transfer(
        token_program_account_info.key,
        contributor_token_account_info.key,
        treasury_token_account_info.key,
        contributor_account_info.key,
        &[],
        (amount * associated_token_mint_pow) as u64
    )?;

    let _ = invoke(&transfer_ix, &[
        contributor_account_info.clone(),
        treasury_token_account_info.clone(),
        contributor_token_account_info.clone(),
        token_program_account_info.clone()
    ]);

    stream.total_deposits = (stream.total_deposits as u64)
        .checked_add(amount as u64)
        .ok_or(StreamError::Overflow)? as f64;

    if stream.funded_on_utc == 0 // First time the stream is being funded
    {
        stream.funded_on_utc = 1000u64
            .checked_mul(clock.unix_timestamp as u64)
            .ok_or(StreamError::Overflow)?;
    }

    // Resume if it was paused by lack of funds OR it was manually paused 
    // and it is going to be manually resumed again 
    if resume == true || no_funds == 1
    {
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64;
    }

    // Save
    Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

    // Pay fees
    transfer_sol_fee(
        system_account_info,
        contributor_account_info,
        msp_ops_account_info,
        ADD_FUNDS_FLAT_FEE
    )
}

pub fn withdraw_v0<'info>(
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    msp_ops_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let clock = Clock::get()?;
    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    let current_block_time = clock.unix_timestamp as u64;
    let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;    
    let mut rate = 0.0;
    
    if stream.rate_interval_in_seconds > 0
    {
        rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
    }

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let associated_token_mint_pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = current_block_time
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;
    let mut escrow_vested_amount = ((stream.escrow_vested_amount_snap * associated_token_mint_pow) as u64)
        .checked_add((rate_time * associated_token_mint_pow) as u64)
        .ok_or(StreamError::Overflow)?;

    let max_vested_amount = ((stream.total_deposits * associated_token_mint_pow) as u64)
        .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
        .ok_or(StreamError::Overflow)?;
    
    if escrow_vested_amount > max_vested_amount
    {
        escrow_vested_amount = max_vested_amount;
    }

    let mut transfer_amount = (amount * associated_token_mint_pow) as u64;

    if transfer_amount > escrow_vested_amount
    {
        transfer_amount = escrow_vested_amount;
    }

    if transfer_amount > 0
    {
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
            transfer_amount
        )?;

        let _ = invoke_signed(&transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_signer_seed]
        );

        // Update stream account data
        let stream_total_wwithdrawals = ((stream.total_withdrawals * associated_token_mint_pow) as u64)
            .checked_add((amount * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

        stream.total_withdrawals = stream_total_wwithdrawals;

        let stream_escrow_vested_amount_snap = escrow_vested_amount
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

        stream.escrow_vested_amount_snap = stream_escrow_vested_amount_snap;
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64; 

        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        let fee = WITHDRAW_PERCENT_FEE * transfer_amount as f64 / associated_token_mint_pow / 100f64;
        // Pay fees
        let _ = transfer_token_fee(
            token_program_account_info,
            beneficiary_token_account_info,
            msp_ops_token_account_info,
            beneficiary_account_info,
            (fee * associated_token_mint_pow) as u64
        );
    }
    
    Ok(())
}

pub fn close_treasury_v0<'info>(
    msp_account_info: &AccountInfo<'info>,
    _msp_ops_token_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    _associated_token_mint_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,

) -> ProgramResult {

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasury.treasury_base_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

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
    
        let _ = invoke(&treasurer_treasury_pool_token_close_ix, &[
            treasurer_treasury_pool_token_account_info.clone(),
            treasurer_account_info.clone(),
            token_program_account_info.clone(),
        ]);
    }

    if treasury_token_account_info.data_len() == spl_token::state::Account::LEN
    {
        let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
            &[
                treasury.treasury_base_address.as_ref(),
                &treasury.treasury_block_height.to_le_bytes()
            ], 
            msp_account_info.key
        );
    
        if treasury_pool_address.ne(treasury_account_info.key)
        {
            return Err(StreamError::InvalidTreasuryData.into());
        }

        let treasury_pool_signer_seed: &[&[_]] = &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes(),
            &treasury_pool_bump_seed.to_le_bytes()
        ];

        let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;

        if treasury_token.amount > 0
        {
            // Credit all treasury token amount to treasurer
            let transfer_ix = spl_token::instruction::transfer(
                token_program_account_info.key,
                treasury_token_account_info.key,
                treasurer_token_account_info.key,
                treasury_account_info.key,
                &[],
                treasury_token.amount
            )?;
        
            let _ = invoke_signed(&transfer_ix, 
                &[
                    treasury_account_info.clone(),
                    treasury_token_account_info.clone(),
                    treasurer_token_account_info.clone(),
                    token_program_account_info.clone(),
                    msp_account_info.clone()
                ],
                &[treasury_pool_signer_seed]
            );
        }

        // Close treasury token account
        let close_token_ix = spl_token::instruction::close_account(
            token_program_account_info.key, 
            treasury_token_account_info.key, 
            treasurer_account_info.key, 
            treasury_account_info.key, 
            &[]
        )?;

        let _ = invoke_signed(&close_token_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                treasurer_account_info.clone(),
                token_program_account_info.clone(),
            ],
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
    msp_account_info: &AccountInfo<'info>,
    msp_ops_account_info: &AccountInfo<'info>,
    msp_ops_token_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
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
    close_treasury: bool

) -> ProgramResult {

    let clock = Clock::get()?;
    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let associated_token_mint_pow = num_traits::pow(10f64, associated_token_mint.decimals.into());

    if stream.treasurer_address.ne(initializer_account_info.key) &&
        stream.beneficiary_address.ne(initializer_account_info.key) 
    {
        return Err(StreamError::InstructionNotAuthorized.into()); // Just the treasurer or the beneficiary can close a stream
    }
    
    let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
    let mut rate = 0.0;

    if stream.rate_interval_in_seconds > 0
    {
        rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
    }

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = (clock.unix_timestamp as u64)
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;
    let mut escrow_vested_amount = ((stream.escrow_vested_amount_snap * associated_token_mint_pow) as u64)
        .checked_add((rate_time * associated_token_mint_pow) as u64)
        .ok_or(StreamError::Overflow)?;

    let vested_amount = ((stream.total_deposits * associated_token_mint_pow) as u64)
        .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
        .ok_or(StreamError::Overflow)?;

    if escrow_vested_amount > vested_amount
    {
        escrow_vested_amount = vested_amount;
    }    

    let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;
    let mut token_amount = treasury_token.amount;    
    
    if escrow_vested_amount > token_amount
    {
        return Err(StreamError::AvailableTreasuryAmountExceeded.into());
    }

    let (treasury_pool_address, treasury_pool_bump_seed) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address.ne(treasury_account_info.key)
    {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    let treasury_pool_signer_seed: &[&[_]] = &[
        treasury.treasury_base_address.as_ref(),
        &treasury.treasury_block_height.to_le_bytes(),
        &treasury_pool_bump_seed.to_le_bytes()
    ];

    if escrow_vested_amount > 0
    {
        // Pausing the stream
        let current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / associated_token_mint_pow;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;

        let beneficiary_fee = CLOSE_STREAM_PERCENT_FEE * escrow_vested_amount as f64 / associated_token_mint_pow / 100f64;
        let transfer_amount = escrow_vested_amount
            .checked_sub((beneficiary_fee * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)?;

        // Credit vested amount minus fee to the beneficiary    
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            beneficiary_token_account_info.key,
            treasury_account_info.key,
            &[],
            transfer_amount
        )?;
    
        let _ = invoke_signed(&transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                beneficiary_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_pool_signer_seed]
        );

        // Pay fee by the beneficiary from the vested amount
        let _ = transfer_token_fee(
            &token_program_account_info,
            &treasury_token_account_info,
            &msp_ops_token_account_info,
            &treasury_account_info,
            (beneficiary_fee * associated_token_mint_pow) as u64
        )?;

        token_amount = token_amount
            .checked_sub(escrow_vested_amount)
            .ok_or(StreamError::Overflow)?;
    }

    let mut escrow_unvested_amount = ((stream.total_deposits * associated_token_mint_pow) as u64)
        .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
        .unwrap()
        .checked_sub(escrow_vested_amount)
        .ok_or(StreamError::Overflow)?;

    if escrow_unvested_amount > 0
    {
        if escrow_unvested_amount > token_amount
        {
            escrow_unvested_amount = token_amount;
        }

        // Crediting escrow unvested amount to the contributor
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            treasurer_token_account_info.key,
            treasury_account_info.key,
            &[],
            escrow_unvested_amount
        )?;

        let _ = invoke_signed(&transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                treasurer_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_pool_signer_seed]
        );
    }

    if close_treasury == true &&
       (
           (*initializer_account_info).key.eq(&treasurer_account_info.key) ||
           (*initializer_account_info).key.eq(&beneficiary_account_info.key)
       )
    {
        // Close treasury account
        let _ = close_treasury_v0(
            msp_account_info,
            msp_ops_token_account_info,
            token_program_account_info,
            treasurer_account_info,
            treasurer_token_account_info,
            treasurer_treasury_pool_token_account_info,
            treasury_account_info,
            treasury_token_account_info,
            associated_token_mint_info,
            treasury_pool_mint_info
        );
    }

    // Close stream account
    let initializer_lamports = initializer_account_info.lamports();
    let stream_lamports = stream_account_info.lamports();

    **stream_account_info.lamports.borrow_mut() = 0;
    **initializer_account_info.lamports.borrow_mut() = initializer_lamports
        .checked_add(stream_lamports)
        .ok_or(StreamError::Overflow)?;

    // Debit fees from the initializer of the instruction
    let _ = transfer_sol_fee(
        &system_account_info,
        &initializer_account_info,
        &msp_ops_account_info,
        CLOSE_STREAM_FLAT_FEE
    )?;

    Ok(())
}

pub fn transfer_sol_fee<'info>(
    system_account_info: &AccountInfo<'info>,
    payer_account_info: &AccountInfo<'info>,
    msp_ops_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let lamports = amount * LAMPORTS_PER_SOL as f64;
    let pay_fee_ix = system_instruction::transfer(
        payer_account_info.key,
        msp_ops_account_info.key,
        lamports as u64
    );

    invoke(&pay_fee_ix, &[
        payer_account_info.clone(),
        msp_ops_account_info.clone(),
        system_account_info.clone()
    ])
}

pub fn transfer_token_fee<'info>(
    token_program_account_info: &AccountInfo<'info>,
    payer_token_account_info: &AccountInfo<'info>,
    msp_ops_token_account_info: &AccountInfo<'info>,
    payer_authority_account_info: &AccountInfo<'info>,
    amount: u64

) -> ProgramResult {

    let fees_ix = spl_token::instruction::transfer(
        token_program_account_info.key,
        payer_token_account_info.key,
        msp_ops_token_account_info.key,
        payer_authority_account_info.key,
        &[],
        amount
    )?;

    invoke(&fees_ix, &[
        payer_authority_account_info.clone(),
        payer_token_account_info.clone(),
        msp_ops_token_account_info.clone(),
        token_program_account_info.clone()
    ])
}



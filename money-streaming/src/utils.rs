
use std::cmp;
use num_traits;
use std::{ string::String, convert::TryInto };
use crate::error::StreamError;
use crate::state::{ Treasury, TreasuryV1, Stream, StreamV1, StreamStatus };
use crate::constants::{
    ADD_FUNDS_FLAT_FEE,
    CLOSE_STREAM_FLAT_FEE,
    CLOSE_STREAM_PERCENT_FEE,
    WITHDRAW_PERCENT_FEE,
    LAMPORTS_PER_SOL,
    MSP_OPS_ACCOUNT_ADDRESS
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
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    msp_ops_account_info: &AccountInfo<'info>,
    msp_ops_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    amount: f64

) -> ProgramResult {

    let mut stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
        &MSP_OPS_ACCOUNT_ADDRESS.parse().unwrap(),
        associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       msp_ops_token_address.ne(msp_ops_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let clock = Clock::get()?;
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

    let transfer_amount = (amount * associated_token_mint_pow) as u64;

    if transfer_amount > escrow_vested_amount
    {
        return Err(StreamError::NotAllowedWithdrawalAmount.into());
    }

    if transfer_amount > 0
    {
        // Withdraw
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
        let stream_total_withdrawals = ((stream.total_withdrawals * associated_token_mint_pow) as u64)
            .checked_add(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

        stream.total_withdrawals = stream_total_withdrawals;

        let stream_escrow_vested_amount_snap = escrow_vested_amount
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

        stream.escrow_vested_amount_snap = stream_escrow_vested_amount_snap;
        stream.stream_resumed_block_height = clock.slot as u64;
        stream.stream_resumed_block_time = clock.unix_timestamp as u64; 

        // Save
        Stream::pack_into_slice(&stream, &mut stream_account_info.data.borrow_mut());

        let fee = WITHDRAW_PERCENT_FEE * transfer_amount as f64 / associated_token_mint_pow / 100f64;
        let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
            msp_ops_account_info.key,
            associated_token_mint_info.key
        );
    
        if msp_ops_token_address != *msp_ops_token_account_info.key 
        {
            return Err(StreamError::InvalidMspOpsToken.into());
        }
    
        if msp_ops_token_account_info.data_len() != spl_token::state::Account::LEN
        {
            // Create treasury associated token account if doesn't exist
            let _ = create_ata_account(
                &system_account_info,
                &rent_account_info,
                &associated_token_program_account_info,
                &token_program_account_info,
                &beneficiary_account_info,
                &msp_ops_account_info,
                &msp_ops_token_account_info,
                &associated_token_mint_info
            )?;
        }

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
    token_program_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    treasurer_treasury_pool_token_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
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

    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
        &MSP_OPS_ACCOUNT_ADDRESS.parse().unwrap(),
        associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       msp_ops_token_address.ne(msp_ops_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    
    let is_running = (stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time) as u64;
    let mut rate = 0.0;

    if stream.rate_interval_in_seconds > 0
    {
        rate = stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64);
    }
    else if stream.total_deposits > stream.total_withdrawals
    {
        rate = ((stream.total_deposits * associated_token_mint_pow) as u64)
            .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;
    }

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = (clock.unix_timestamp as u64)
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;
    let mut escrow_vested_amount = ((stream.escrow_vested_amount_snap * associated_token_mint_pow) as u64)
        .checked_add((rate_time * associated_token_mint_pow) as u64)
        .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

    if stream.total_deposits > stream.total_withdrawals
    {
        let vested_amount = ((stream.total_deposits * associated_token_mint_pow) as u64)
            .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;

        if escrow_vested_amount > vested_amount
        {
            escrow_vested_amount = vested_amount;
        }
    }   

    let treasury_token = spl_token::state::Account::unpack_from_slice(&treasury_token_account_info.data.borrow())?;
    let mut token_amount = treasury_token.amount as f64 / associated_token_mint_pow;    
    
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

    if escrow_vested_amount > 0.0
    {
        // Pausing the stream
        let current_block_height = clock.slot as u64;
        let current_block_time = clock.unix_timestamp as u64;
        stream.escrow_vested_amount_snap = escrow_vested_amount;
        stream.escrow_vested_amount_snap_block_height = current_block_height;
        stream.escrow_vested_amount_snap_block_time = current_block_time;

        let beneficiary_fee = CLOSE_STREAM_PERCENT_FEE * escrow_vested_amount as f64 / 100f64;
        let transfer_amount = ((escrow_vested_amount * associated_token_mint_pow) as u64)
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
        let fee_transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            msp_ops_token_account_info.key,
            treasury_account_info.key,
            &[],
            (beneficiary_fee * associated_token_mint_pow) as u64
        )?;
    
        let _ = invoke_signed(&fee_transfer_ix, 
            &[
                treasury_account_info.clone(),
                treasury_token_account_info.clone(),
                msp_ops_token_account_info.clone(),
                token_program_account_info.clone(),
                msp_account_info.clone()
            ],
            &[treasury_pool_signer_seed]
        );

        token_amount = ((token_amount * associated_token_mint_pow) as u64)
            .checked_sub(transfer_amount)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;
    }

    let escrow_unvested_amount;

    if stream.total_deposits > stream.total_withdrawals
    {
        escrow_unvested_amount = ((stream.total_deposits * associated_token_mint_pow) as u64)
            .checked_sub((stream.total_withdrawals * associated_token_mint_pow) as u64)
            .unwrap()
            .checked_sub((escrow_vested_amount * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;
    }
    else
    {
        escrow_unvested_amount = ((token_amount * associated_token_mint_pow) as u64)
            .checked_sub((escrow_vested_amount * associated_token_mint_pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / associated_token_mint_pow;
    }

    if escrow_unvested_amount > 0.0
    {
        let transfer_unvested_amount = (escrow_unvested_amount as f64 * associated_token_mint_pow) as u64;

        // Crediting escrow unvested amount to the treasurer
        let transfer_ix = spl_token::instruction::transfer(
            token_program_account_info.key,
            treasury_token_account_info.key,
            treasurer_token_account_info.key,
            treasury_account_info.key,
            &[],
            transfer_unvested_amount
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

    // Debit fees from the initializer of the instruction
    let _ = transfer_sol_fee(
        &system_account_info,
        &initializer_account_info,
        &msp_ops_account_info,
        CLOSE_STREAM_FLAT_FEE
    )?;

    if close_treasury == true && stream.treasurer_address.eq(initializer_account_info.key)
    {
        // Close treasury account
        let _ = close_treasury_v0(
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

    // Close stream account
    let treasurer_lamports = treasurer_account_info.lamports();
    let stream_lamports = stream_account_info.lamports();

    **stream_account_info.lamports.borrow_mut() = 0;
    **treasurer_account_info.lamports.borrow_mut() = treasurer_lamports
        .checked_add(stream_lamports)
        .ok_or(StreamError::Overflow)?;

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

pub fn get_stream_status_v0<'info>(
    stream: &Stream,
    clock: &Clock

) -> Result<StreamStatus, StreamError> {

    let now = clock.unix_timestamp as u64 * 1000u64;

    if stream.start_utc > now
    {
        return Ok(StreamStatus::Scheduled);
    }

    if stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time
    {
        return Ok(StreamStatus::Running);
    }

    return Ok(StreamStatus::Paused);
}

pub fn get_stream_status<'info>(
    stream: &StreamV1,
    clock: &Clock

) -> Result<StreamStatus, StreamError> {

    let now = clock.unix_timestamp as u64 * 1000u64;

    if stream.start_utc > now
    {
        return Ok(StreamStatus::Scheduled);
    }

    if stream.stream_resumed_block_time >= stream.escrow_vested_amount_snap_block_time
    {
        return Ok(StreamStatus::Running);
    }

    return Ok(StreamStatus::Paused);
}

pub fn get_stream_vested_amount_v0<'info>(
    stream: &Stream,
    clock: &Clock,
    decimals: u64

) -> Result<u64, StreamError> {

    let status = get_stream_status_v0(stream, clock)?;

    if status == StreamStatus::Scheduled
    {
        return Ok(0);
    }

    let is_running = match status
    {
        k if k == StreamStatus::Running => 1,
        _ => 0
    };

    let rate = match stream.rate_interval_in_seconds
    {
        k if k > 0 => stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64),
        _ => stream.total_deposits - stream.total_withdrawals
    };

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = (clock.unix_timestamp as u64)
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;    
    let pow = num_traits::pow(10u64, decimals.try_into().unwrap());
    let total_deposits = stream.total_deposits as u64 * pow;
    let total_withdrawals = stream.total_withdrawals as u64 * pow;
    let max_vested_amount = total_deposits
        .checked_sub(total_withdrawals)
        .ok_or(StreamError::Overflow)?;

    let mut cliff_vest_amount = stream.cliff_vest_amount as u64 * pow;

    if stream.cliff_vest_percent > 0.0
    {
        cliff_vest_amount = stream.cliff_vest_percent as u64 * max_vested_amount / 100u64;
    }

    let mut escrow_vested_amount = (stream.escrow_vested_amount_snap as u64 * pow)
        .checked_add(cliff_vest_amount)
        .unwrap()
        .checked_add(rate_time as u64 * pow)
        .ok_or(StreamError::Overflow)?;

    if escrow_vested_amount > max_vested_amount
    {
        escrow_vested_amount = max_vested_amount;
    }

    return Ok(escrow_vested_amount);
}

pub fn get_stream_vested_amount<'info>(
    stream: &StreamV1,
    clock: &Clock,
    decimals: u64

) -> Result<u64, StreamError> {

    let status = get_stream_status(stream, clock)?;

    if status == StreamStatus::Scheduled
    {
        return Ok(0);
    }

    let is_running = match status
    {
        k if k == StreamStatus::Running => 1,
        _ => 0
    };

    let rate = match stream.rate_interval_in_seconds
    {
        k if k > 0 => stream.rate_amount / (stream.rate_interval_in_seconds as f64) * (is_running as f64),
        _ => stream.allocation
    };

    let marker_block_time = cmp::max(stream.stream_resumed_block_time, stream.escrow_vested_amount_snap_block_time);
    let elapsed_time = (clock.unix_timestamp as u64)
        .checked_sub(marker_block_time)
        .ok_or(StreamError::Overflow)?;

    let rate_time = rate * elapsed_time as f64;    
    let pow = num_traits::pow(10u64, decimals.try_into().unwrap());
    let stream_allocation = stream.allocation as u64 * pow;
    let mut cliff_vest_amount = stream.cliff_vest_amount as u64 * pow;

    if stream.cliff_vest_percent > 0.0
    {
        cliff_vest_amount = stream.cliff_vest_percent as u64 * stream_allocation / 100u64;
    }

    let mut escrow_vested_amount = (stream.escrow_vested_amount_snap as u64 * pow)
        .checked_add(cliff_vest_amount)
        .unwrap()
        .checked_add(rate_time as u64 * pow)
        .ok_or(StreamError::Overflow)?;

    if escrow_vested_amount > stream_allocation
    {
        escrow_vested_amount = stream_allocation;
    }

    return Ok(escrow_vested_amount);
}

pub fn check_can_create_stream<'info>(
    program_id: &Pubkey,
    msp_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    allocation: f64

) -> ProgramResult {

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    if stream.initialized == true
    {
        return Err(StreamError::StreamAlreadyInitialized.into());
    }

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !treasurer_account_info.is_signer
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    if treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasury.associated_token_address.ne(associated_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    
    if allocation > treasury.balance
    {
        return Err(StreamError::AvailableTreasuryAmountExceeded.into());
    }
    else if treasury.streams_amount > 0 && treasury.allocation <= treasury.balance
    {
        let available_balance = ((treasury.balance * pow) as u64)
            .checked_sub((treasury.allocation * pow) as u64) 
            .ok_or(StreamError::Overflow)? as f64 / pow;

        if allocation > available_balance
        {
            return Err(StreamError::AvailableTreasuryAmountExceeded.into());
        }
    }

    Ok(())
}

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

pub fn create_stream_update_treasury(
    treasury: &mut TreasuryV1,
    stream: &StreamV1,
    decimals: usize

) -> ProgramResult {

    let pow = num_traits::pow(10f64, decimals.into());
    let rate = stream.rate_amount / stream.rate_interval_in_seconds as f64;
    let depletion_rate = ((treasury.depletion_rate * pow) as u64)
        .checked_add((rate * pow) as u64)
        .ok_or(StreamError::Overflow)? as f64 / pow;

    treasury.depletion_rate = depletion_rate;        
    treasury.streams_amount = treasury.streams_amount
        .checked_add(1)
        .ok_or(StreamError::Overflow)?;

    if stream.allocation > 0.0
    {
        let treasury_allocation = ((treasury.allocation * pow) as u64)
            .checked_add((stream.allocation * pow) as u64)
            .ok_or(StreamError::Overflow)?;

        treasury.allocation = treasury_allocation as f64 / pow;
    }

    if stream.allocation_reserved > 0.0
    {
        let treasury_allocation = ((treasury.allocation * pow) as u64)
            .checked_add((stream.allocation_reserved * pow) as u64)
            .ok_or(StreamError::Overflow)?;

        treasury.allocation = treasury_allocation as f64 / pow;
    }

    Ok(())
}

pub fn check_can_add_funds<'info>(
    program_id: &Pubkey,
    msp_account_info: &AccountInfo<'info>,
    contributor_account_info: &AccountInfo<'info>,
    contributor_treasury_pool_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !contributor_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    if treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        treasury_account_info.key,
        associated_token_mint_info.key
    );

    if treasury_token_address != *treasury_token_account_info.key 
    {
        return Err(StreamError::InvalidTreasuryAccount.into());
    }

    if treasury_token_account_info.data_len() == 0
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
        )?;
    }

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address != *treasury_account_info.key 
    {
        return Err(StreamError::InvalidTreasuryPool.into());
    }

    if contributor_treasury_pool_token_account_info.data_len() == 0
    {
        // Create contributor treasury associated token account
        let contributor_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
            contributor_account_info.key,
            treasury_pool_mint_info.key
        );

        if contributor_treasury_pool_token_address.ne(contributor_treasury_pool_token_account_info.key)
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

    Ok(())
}

pub fn mint_treasury_pool_tokens<'info>(
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
    treasury: &mut TreasuryV1,
    associated_token_mint_info: &AccountInfo<'info>,
    allocation_type: u8,
    amount: f64

) -> ProgramResult {

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());

    let balance = ((treasury.balance * pow) as u64)
        .checked_add((amount * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    treasury.balance = balance as f64 / pow;

    if allocation_type == 0
    {
        let treasury_allocation = ((treasury.allocation * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)?;

        treasury.allocation = treasury_allocation as f64 / pow;

    } 
    else if allocation_type == 1
    {
        let treasury_allocation = ((treasury.allocation * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)?;

        treasury.allocation = treasury_allocation as f64 / pow;
        
        let treasury_allocation_reserved = ((treasury.allocation_reserved * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)?;

        treasury.allocation_reserved = treasury_allocation_reserved as f64 / pow;
    }

    treasury.associated_token_address = *associated_token_mint_info.key;

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
        &stream, 
        &clock, 
        associated_token_mint.decimals.try_into().unwrap()
    )?;

    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let allocation = (stream.allocation * pow) as u64;

    // Pause because the allocation amount was reached
    if escrow_vested_amount > allocation
    {
        stream.escrow_vested_amount_snap = escrow_vested_amount as f64 / pow;
        stream.escrow_vested_amount_snap_slot = current_slot;
        stream.escrow_vested_amount_snap_block_time = current_block_time;
    }

    if allocation_type == 1 &&
       allocation_stream_address.ne(&Pubkey::default()) && 
       stream_account_info.key.eq(&allocation_stream_address)
    {
        stream.allocation = allocation
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;

        stream.allocation_reserved = ((stream.allocation_reserved * pow) as u64)
            .checked_add((amount * pow) as u64)
            .ok_or(StreamError::Overflow)? as f64 / pow;
    }

    // if it was paused before because of lack of money then resume it again 
    if escrow_vested_amount > allocation
    {
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

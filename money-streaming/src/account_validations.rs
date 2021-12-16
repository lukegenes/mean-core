use num_traits;
use crate::error::StreamError;
use crate::state::*;
use crate::constants::*;
use crate::utils::*;
use solana_program::{
    // msg,
    pubkey::Pubkey,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_pack::{ Pack },
};

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

pub fn check_can_add_funds_v0<'info>(
    program_id: &Pubkey,
    msp_account_info: &AccountInfo<'info>,
    contributor_account_info: &AccountInfo<'info>,
    contributor_treasury_pool_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
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

    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, _) = Pubkey::find_program_address(
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

    if stream_account_info.data_len() == Stream::LEN
    {
        let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasury_address.ne(treasury_account_info.key)
        {
            return Err(StreamError::InvalidStreamAccount.into());
        }
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
    stream_account_info: &AccountInfo<'info>,
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

    // Check treasury address
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

    // Check treasury pool mint address
    let (treasury_pool_mint_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            treasury_pool_address.as_ref(),
            &slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_mint_address.ne(treasury_pool_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
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

    if stream_account_info.data_len() == StreamV1::LEN
    {
        let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasury_address.ne(treasury_account_info.key)
        {
            return Err(StreamError::InvalidStreamAccount.into());
        }
    }

    Ok(())
}

pub fn check_can_withdraw_funds<'info>(
    program_id: &Pubkey,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !beneficiary_account_info.is_signer
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    if treasury_account_info.owner != program_id ||
       stream_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    if treasury_token_address.ne(treasury_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT_ADDRESS.parse().unwrap(),
        associated_token_mint_info.key
    );

    if msp_ops_token_address.ne(fee_treasury_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    Ok(())
}

pub fn check_can_pause_stream<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    if stream_account_info.owner != program_id ||
    (
        stream.treasurer_address.ne(initializer_account_info.key) && 
        stream.beneficiary_address.ne(initializer_account_info.key)
    )
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    Ok(())
}

pub fn check_can_resume_stream<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    if stream_account_info.owner != program_id ||
    (
        stream.treasurer_address.ne(initializer_account_info.key) && 
        stream.beneficiary_address.ne(initializer_account_info.key)
    )
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    Ok(())
}

pub fn check_can_close_stream<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    if stream_account_info.owner != program_id || treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    if stream.treasurer_address.ne(initializer_account_info.key) &&
       stream.beneficiary_address.ne(initializer_account_info.key) 
    {
        return Err(StreamError::InstructionNotAuthorized.into()); // Just the treasurer or the beneficiary can close a stream
    }

    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    let msp_ops_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT_ADDRESS.parse().unwrap(),
        associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       msp_ops_token_address.ne(fee_treasury_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    if treasury.associated_token_address.ne(associated_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }

    Ok(())
}

pub fn check_can_close_treasury<'info>(
    program_id: &Pubkey,
    treasurer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !treasurer_account_info.is_signer
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    
    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasury.treasurer_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    if treasury.streams_amount > 0
    {
        return Err(StreamError::CloseTreasuryWithStreams.into());
    }

    Ok(())
}
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
    treasurer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    allocation_assigned: f64,
    allocation_reserved: f64

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::None, Option::None, Option::Some(rent_account_info), Option::Some(system_account_info)
    );
    // Check the tresurer is the signer
    if !treasurer_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    // Check the MSP is the owner of the treasury 
    if treasury_account_info.owner != program_id {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check the treasury associated token account info
    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasury.associated_token_address.ne(associated_token_mint_info.key) {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }
    // Check if the stream is already initialized
    if stream_account_info.data_len() > 0 {
        return Err(StreamError::StreamAlreadyInitialized.into())
    }
    // Check Money Streaming Program account info
    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }
    // Check if the requested allocations are valid
    if allocation_reserved > allocation_assigned {
        return Err(StreamError::StreamAllocationExceeded.into());
    }

    let associated_token_mint = spl_token::state::Mint::unpack_from_slice(&associated_token_mint_info.data.borrow())?;
    let pow = num_traits::pow(10f64, associated_token_mint.decimals.into());
    let requested_allocation = (allocation_assigned * pow) as u64;
    let unallocated_balance = ((treasury.balance * pow) as u64)
        .checked_sub((treasury.allocation_assigned * pow) as u64)
        .ok_or(StreamError::Overflow)?;

    if requested_allocation <= 0 || requested_allocation > unallocated_balance {
        return Err(StreamError::InvalidAssignedAllocation.into());
    }

    Ok(())
}

pub fn check_can_add_funds_v0<'info>(
    program_id: &Pubkey,
    msp_account_info: &AccountInfo<'info>,
    contributor_account_info: &AccountInfo<'info>,
    contributor_token_account_info: &AccountInfo<'info>,
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

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );

    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    // Check the contributor is the signer
    if !contributor_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    // Check the contributor token account
    let contributor_token_address = spl_associated_token_account::get_associated_token_address(
        contributor_account_info.key,
        associated_token_mint_info.key
    );

    if contributor_token_address.ne(contributor_token_account_info.key)
    {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }

    // Check the contributor treasury pool token account
    let contributor_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
        contributor_account_info.key,
        treasury_pool_mint_info.key
    );

    if contributor_treasury_pool_token_address.ne(contributor_treasury_pool_token_account_info.key)
    {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }

    if contributor_treasury_pool_token_account_info.data_len() == 0
    {
        // Create the contributor treasury token account if the account does not exists
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

    // Check the treasury account is owned by the Money Streaming Program
    if treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check treasury address the valid PDA
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

    // Check the treasury token account is valid for the associated token Mint
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

    // Check treasury pool mint address
    let (treasury_pool_mint_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            treasury_pool_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_mint_address.ne(treasury_pool_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    if stream_account_info.data_len() == Stream::LEN
    {
        let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

        if stream.treasury_address.ne(&treasury_pool_address)
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
    contributor_token_account_info: &AccountInfo<'info>, 
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

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );

    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }
    // Check the contributor is the signer
    if !contributor_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    // Check the contributor token account
    let contributor_token_address = spl_associated_token_account::get_associated_token_address(
        contributor_account_info.key, associated_token_mint_info.key
    );

    if contributor_token_address.ne(contributor_token_account_info.key) {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }
    // Check the contributor treasury pool token account
    let contributor_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
        contributor_account_info.key, treasury_pool_mint_info.key
    );

    if contributor_treasury_pool_token_address.ne(contributor_treasury_pool_token_account_info.key) {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }

    if contributor_treasury_pool_token_account_info.data_len() == 0 { // Create the contributor treasury token account if the account does not exists
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &contributor_account_info, &contributor_account_info,
            &contributor_treasury_pool_token_account_info, &treasury_pool_mint_info
        );
    }
    // Check the treasury account is owned by the Money Streaming Program
    if treasury_account_info.owner != program_id {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check treasury address the valid PDA
    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address != *treasury_account_info.key {
        return Err(StreamError::InvalidTreasuryPool.into());
    }
    // Check the treasury token account is valid for the associated token Mint
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        treasury_account_info.key,
        associated_token_mint_info.key
    );

    if treasury_token_address != *treasury_token_account_info.key {
        return Err(StreamError::InvalidTreasuryAccount.into());
    }

    if treasury_token_account_info.data_len() == 0 { // Create treasury associated token account if doesn't exist
        let _ = create_ata_account(
            &system_account_info, &rent_account_info, &associated_token_program_account_info,
            &token_program_account_info, &contributor_account_info, &treasury_account_info,
            &treasury_token_account_info, &associated_token_mint_info
        )?;
    }
    // Check treasury pool mint address
    let (treasury_pool_mint_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            treasury_pool_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_mint_address.ne(treasury_pool_mint_info.key) {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    if stream_account_info.data_len() == StreamV1::LEN {
        let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
        if stream.treasury_address.ne(&treasury_pool_address) {
            return Err(StreamError::InvalidStreamAccount.into());
        }
    }

    Ok(())
}

pub fn check_can_withdraw_funds_v0<'info>(
    program_id: &Pubkey,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );

    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }
    // Check the treasury and the stream are owned by the MSP
    if treasury_account_info.owner != program_id || stream_account_info.owner != program_id {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check the beneficiary is the signer
    if !beneficiary_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    // Check if the stream data has a valid size
    if stream_account_info.data_len() != Stream::LEN {
        return Err(StreamError::InvalidStreamData.into());
    }

    let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;
    // Check the beneficiary account info
    if stream.beneficiary_address.ne(beneficiary_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check the beneficiary token account info
    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address, associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key){
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }
    // Check the associated token mint account
    if stream.beneficiary_associated_token.ne(associated_token_mint_info.key) {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }
    // Check treasury account info
    if stream.treasury_address.ne(treasury_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check treasury token account info
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address, associated_token_mint_info.key
    );

    if treasury_token_address.ne(treasury_token_account_info.key) {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }
    // Check the fee treasury token account info
    let fee_treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT.parse().unwrap(), associated_token_mint_info.key
    );

    if fee_treasury_token_address.ne(fee_treasury_token_account_info.key) {
        return Err(StreamError::InvalidMspOpsToken.into());
    }

    Ok(())
}

pub fn check_can_withdraw_funds<'info>(
    program_id: &Pubkey,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    treasury_token_account_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );
    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }
    // Check the treasury and the stream are owned by the MSP
    if treasury_account_info.owner != program_id || stream_account_info.owner != program_id {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check the beneficiary is the signer
    if !beneficiary_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    // Check if the stream data has a valid size
    if stream_account_info.data_len() != StreamV1::LEN {
        return Err(StreamError::InvalidStreamData.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;
    // Check the beneficiary account info
    if stream.beneficiary_address.ne(beneficiary_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check the beneficiary token account info
    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address, associated_token_mint_info.key
    );

    if beneficiary_token_address.ne(beneficiary_token_account_info.key) {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }
    // Check the associated token mint account
    if stream.beneficiary_associated_token.ne(associated_token_mint_info.key) {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }
    // Check treasury account info
    if stream.treasury_address.ne(treasury_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check treasury token account info
    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address, associated_token_mint_info.key
    );

    if treasury_token_address.ne(treasury_token_account_info.key) {
        return Err(StreamError::InvalidAssociatedTokenAccount.into());
    }

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;
    // Check the treasury token mint account
    if treasury.associated_token_address.ne(associated_token_mint_info.key) {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }
    // Check the fee treasury token account info
    let fee_treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT.parse().unwrap(), associated_token_mint_info.key
    );

    if fee_treasury_token_address.ne(fee_treasury_token_account_info.key) {
        return Err(StreamError::InvalidMspOpsToken.into());
    }

    Ok(())
}

pub fn check_can_pause_or_resume_stream<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check the initializer is the signer
    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    // Check that only the treasurer or the beneficiary can pause the stream
    if stream_account_info.owner != program_id ||
    (
        stream.treasurer_address.ne(initializer_account_info.key) && 
        stream.beneficiary_address.ne(initializer_account_info.key)
    )
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasury account info
    if stream.treasury_address.ne(treasury_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the size of the Treasury in the correct
    if treasury_account_info.data_len() != TreasuryV1::LEN
    {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

    // Check the associated token mint info
    if stream.beneficiary_associated_token.ne(associated_token_mint_info.key) || 
       treasury.associated_token_address.ne(associated_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }

    // Check Money Streaming Program account info
    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    Ok(())
}

pub fn check_can_close_stream_v0<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>, 
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );

    // Check the initializer is the signer
    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    // Check that the stream and treasury accounts owner is the MSP 
    if stream_account_info.owner != program_id || treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the stream account has a valid size
    if stream_account_info.data_len() != Stream::LEN
    {
        return Err(StreamError::InvalidStreamData.into());
    }

    let stream = Stream::unpack_from_slice(&stream_account_info.data.borrow())?;

    // Validate that only the treasurer or the beneficiary can close the stream
    if stream.treasurer_address.ne(initializer_account_info.key) &&
       stream.beneficiary_address.ne(initializer_account_info.key) 
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasurer account info in the stream
    if stream.treasurer_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the associated token mint account info
    if stream.beneficiary_address.ne(beneficiary_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasury account has a valid size
    if treasury_account_info.data_len() != Treasury::LEN
    {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    // Check that the treasury address is the valid PDA
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
    
    // Check the treasurer account info in the treasury
    if treasury.treasury_base_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the associated token mint account info
    if stream.beneficiary_associated_token.ne(associated_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }

    // Check all associated token accounts info
    let treasurer_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasurer_address,
        associated_token_mint_info.key
    );

    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    let fee_treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT.parse().unwrap(),
        associated_token_mint_info.key
    );

    if treasurer_token_address.ne(treasurer_token_account_info.key) || 
       beneficiary_token_address.ne(beneficiary_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       fee_treasury_token_address.ne(fee_treasury_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasury pool mint account info
    if treasury.treasury_mint_address.ne(treasury_pool_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    // Check that the treasury pool mint address is the valid PDA
    let (treasury_pool_mint_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            treasury_pool_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_mint_address.ne(treasury_pool_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    Ok(())
}

pub fn check_can_close_stream<'info>(
    program_id: &Pubkey,
    initializer_account_info: &AccountInfo<'info>,
    treasurer_account_info: &AccountInfo<'info>,
    treasurer_token_account_info: &AccountInfo<'info>,
    beneficiary_account_info: &AccountInfo<'info>,
    beneficiary_token_account_info: &AccountInfo<'info>,
    associated_token_mint_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>, 
    treasury_token_account_info: &AccountInfo<'info>,
    treasury_pool_mint_info: &AccountInfo<'info>,
    stream_account_info: &AccountInfo<'info>,
    fee_treasury_token_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    associated_token_program_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>,
    rent_account_info: &AccountInfo<'info>,
    system_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::Some(associated_token_program_account_info),
        Option::Some(token_program_account_info),
        Option::Some(rent_account_info),
        Option::Some(system_account_info)
    );

    // Check the initializer is the signer
    if !initializer_account_info.is_signer 
    {
        return Err(StreamError::MissingInstructionSignature.into());
    }

    // Check that the stream and treasury accounts owner is the MSP 
    if stream_account_info.owner != program_id || treasury_account_info.owner != program_id
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the stream account has a valid size
    if stream_account_info.data_len() != StreamV1::LEN
    {
        return Err(StreamError::InvalidStreamData.into());
    }

    let stream = StreamV1::unpack_from_slice(&stream_account_info.data.borrow())?;

    // Validate that only the treasurer or the beneficiary can close the stream
    if stream.treasurer_address.ne(initializer_account_info.key) &&
       stream.beneficiary_address.ne(initializer_account_info.key) 
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasurer account info in the stream
    if stream.treasurer_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the beneficiary account info in the stream
    if stream.beneficiary_address.ne(beneficiary_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasury account has a valid size
    if treasury_account_info.data_len() != TreasuryV1::LEN
    {
        return Err(StreamError::InvalidTreasuryData.into());
    }

    // Check that the treasury address is the valid PDA
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
    
    // Check the treasurer account info in the treasury
    if treasury.treasurer_address.ne(treasurer_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the associated token mint account info
    if stream.beneficiary_associated_token.ne(associated_token_mint_info.key) ||
       treasury.associated_token_address.ne(associated_token_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryAssociatedToken.into());
    }

    // Check all associated token accounts info
    let treasurer_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasurer_address,
        associated_token_mint_info.key
    );

    let beneficiary_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.beneficiary_address,
        associated_token_mint_info.key
    );

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &stream.treasury_address,
        associated_token_mint_info.key
    );

    let fee_treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT.parse().unwrap(),
        associated_token_mint_info.key
    );

    if treasurer_token_address.ne(treasurer_token_account_info.key) || 
       beneficiary_token_address.ne(beneficiary_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       fee_treasury_token_address.ne(fee_treasury_token_account_info.key)
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check the treasury pool mint account info
    if treasury.mint_address.ne(treasury_pool_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    // Check that the treasury pool mint address is the valid PDA
    let (treasury_pool_mint_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasurer_address.as_ref(),
            treasury_pool_address.as_ref(),
            &treasury.slot.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_mint_address.ne(treasury_pool_mint_info.key)
    {
        return Err(StreamError::InvalidTreasuryPoolMint.into());
    }

    // Check the Money Streaming Program account info
    if msp_account_info.key.ne(program_id)
    {
        return Err(StreamError::IncorrectProgramId.into());
    }

    Ok(())
}

pub fn check_can_close_treasury_v0<'info>(
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
    token_program_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::None, Option::Some(token_program_account_info),  Option::None, Option::None
    );
    // Check the tresurer is the signer
    if !treasurer_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    // Check the treasury account size is valid
    if treasury_account_info.data_len() != Treasury::LEN {
        return Err(StreamError::InvalidTreasuryData.into());
    }
    // Check that the treasury address is the valid PDA
    let treasury = Treasury::unpack_from_slice(&treasury_account_info.data.borrow())?;
    let (treasury_pool_address, _) = Pubkey::find_program_address(
        &[
            treasury.treasury_base_address.as_ref(),
            &treasury.treasury_block_height.to_le_bytes()
        ], 
        msp_account_info.key
    );

    if treasury_pool_address != *treasury_account_info.key {
        return Err(StreamError::InvalidTreasuryPool.into());
    }
    // Check the treasurer account info
    if treasury.treasury_base_address.ne(treasurer_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }
    // Check all associated token accounts info
    let treasurer_token_address = spl_associated_token_account::get_associated_token_address(
        &treasury.treasury_base_address,
        associated_token_mint_info.key
    );

    let treasurer_treasury_pool_token_address = spl_associated_token_account::get_associated_token_address(
        &treasury.treasury_mint_address,
        treasury_pool_mint_info.key
    );

    let treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &treasury_pool_address,
        associated_token_mint_info.key
    );

    let fee_treasury_token_address = spl_associated_token_account::get_associated_token_address(
        &FEE_TREASURY_ACCOUNT.parse().unwrap(),
        associated_token_mint_info.key
    );

    if treasurer_token_address.ne(treasurer_token_account_info.key) || 
       treasurer_treasury_pool_token_address.ne(treasurer_treasury_pool_token_account_info.key) ||
       treasury_token_address.ne(treasury_token_account_info.key) ||
       fee_treasury_token_address.ne(fee_treasury_token_account_info.key) 
    {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    // Check Money Streaming Program account info
    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    Ok(())
}

pub fn check_can_close_treasury<'info>(
    program_id: &Pubkey,
    treasurer_account_info: &AccountInfo<'info>,
    treasury_account_info: &AccountInfo<'info>,
    msp_account_info: &AccountInfo<'info>,
    token_program_account_info: &AccountInfo<'info>

) -> ProgramResult {

    // Check system accounts
    let _ = check_system_accounts(
        Option::None, Option::Some(token_program_account_info), Option::None, Option::None
    );

    if msp_account_info.key.ne(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    if !treasurer_account_info.is_signer {
        return Err(StreamError::MissingInstructionSignature.into());
    }
    
    let treasury = TreasuryV1::unpack_from_slice(&treasury_account_info.data.borrow())?;

    if treasury.treasurer_address.ne(treasurer_account_info.key) {
        return Err(StreamError::InstructionNotAuthorized.into());
    }

    if treasury.streams_amount > 0 {
        return Err(StreamError::CloseTreasuryWithStreams.into());
    }

    Ok(())
}
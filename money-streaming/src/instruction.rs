// Program API, (de)serializing instruction data

use std::{ mem::size_of };

use solana_program::{
    pubkey::Pubkey,
    instruction::{ AccountMeta, Instruction }
};

use crate::{
    check_program_account,
    utils::*,
    error::StreamError
};

pub enum StreamInstruction {

    /// Initialize a new stream contract
    ///
    /// 0. `[signer]` The treasurer account (The creator of the money stream).
    /// 1. `[]` The treasury account (The stream contract treasury account).
    /// 2. `[]` The beneficiary associated token mint account.
    /// 3. `[writable]` The stream account (The stream contract account).
    /// 4. `[]` The beneficiary account (The beneficiary of money stream).
    /// 5.  [writable] The Money Streaming Program operating account (Fees account).
    /// 6.  [] The Money Streaming Program account.
    /// 7. `[]` The System Program account.
    /// 8. `[]` Rent sysvar account.
    CreateStream {
        stream_name: String,        
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        allocation_reserved: f64,
        allocation: f64,
        funded_on_utc: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64, // OPTIONAL
        cliff_vest_percent: f64, // OPTIONAL
        auto_pause_in_seconds: u64
    },

    /// Adds a specific amount of funds to a stream
    ///
    /// 0. `[signer]` The contributor account
    /// 1. `[writable]` The contributor token account
    /// 2. `[writable]` The contributor treasury pool token account (the account of the token issued by the treasury and owned by the contributor)
    /// 4. `[writable]` The treasury account (Stream treasury account).
    /// 5. `[writable]` The treasury token account.
    /// 3. `[]` The treasury associated token account
    /// 6. `[writable]` The treasury pool mint account (the mint of the treasury pool token)
    /// 7.  [writable] The Money Streaming Program operating account (Fees account).
    /// 8.  [] The Money Streaming Program account.
    /// 9. `[]` The Associated Token Program account.
    /// 10. `[]` The Token Program account.
    /// 11. `[]` The System Program account.
    /// 12. `[]` Rent sysvar account.
    AddFunds {
        amount: f64,
        allocation_type: u8,
        allocation_stream_address: Pubkey
    },

    /// Recovers a specific amount of funds from a previously funded stream
    ///
    /// 0. `[signer]` The contributor account
    /// 1. `[writable]` The contributor token account
    /// 2. `[writable]` The contributor treasury pool token account (the account of the token issued by the treasury and owned by the contributor)
    /// 3. `[]` The associated token mint account.
    /// 4. `[]` The treasury account.
    /// 5. `[writable]` The treasury token account.    
    /// 6. `[]` The treasury pool mint account (the mint of the treasury pool token)
    /// 7.  [writable] The Money Streaming Program operating account.
    /// 8.  [writable] The Money Streaming Program operating token account.
    /// 9.  [] The Money Streaming Program account.
    /// 10. `[]` The Token Program account.
    RecoverFunds {
        amount: f64
    },

    /// 0. `[signer]` The beneficiary account
    /// 1. `[writable]` The beneficiary token account (the recipient of the money)
    /// 2. `[]` The associated token mint account
    /// 3. `[]` The treasury account
    /// 4. `[writable]` The treasury token account
    /// 5. `[writable]` The stream account (The stream contract account).
    /// 6.  [writable] The Money Streaming Program operating token account.
    /// 7. `[]` The Money Streaming Program account.
    /// 8. `[]` The Token Program account.
    Withdraw { 
        amount: f64
    },

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream account (The stream contract account).
    /// 2. `[writable]` The Money Streaming Program operating account.
    /// 3. `[]` System Program account.
    PauseStream,

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream account (The stream contract account).
    /// 2. `[writable]` The Money Streaming Program operating account.
    /// 3. `[]` System Program account.
    ResumeStream,

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream terms account (Update proposal account).
    /// 2. `[]` The counterparty's account (if the initializer is the treasurer then it would be the beneficiary or vice versa)
    /// 3. `[writable]` The stream account (The stream contract account).
    /// 4.  [writable] The Money Streaming Program operating account (Fees account).
    /// 5. `[]` System Program account.
    ProposeUpdate {
        proposed_by: Pubkey,
        stream_name: String,
        treasurer_address: Pubkey,
        beneficiary_address: Pubkey,
        associated_token_address: Pubkey, // OPTIONAL
        rate_amount: f64,
        rate_interval_in_seconds: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: f64, // OPTIONAL
        cliff_vest_percent: f64, // OPTIONAL
        auto_pause_in_seconds: u64
    },

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream terms account (Update proposal account).
    /// 2. `[]` The counterparty's account (if the initializer is the treasurer then it would be the beneficiary or vice versa)
    /// 3. `[writable]` The stream account (The stream contract account). 
    /// 4.  [writable] The Money Streaming Program operating account (Fees account).
    /// 5. `[]` System Program account.
    AnswerUpdate {
        approve: bool
    },

    /// 0. `[signer, writable]` The initializer account (treasurer/beneficiary)
    /// 1. `[writable]` The treasurer account (the creator of the treasury)
    /// 2. `[writable]` The beneficiary token account (the recipient of the money)
    /// 3. `[]` The associated token mint account
    /// 4. `[writable]` The treasury account
    /// 5. `[writable]` The treasury token account
    /// 6. `[writable]` The stream account (The stream contract account).
    /// 7. `[writable]` The Money Streaming Program operating account.
    /// 8. `[writable]` The Money Streaming Program operating token account.
    /// 9. `[]` The Money Streaming Program account
    /// 10. `[]` The Token Program account.
    /// 11. `[]` System Program account.
    CloseStream {
        auto_close_treasury: bool
    },

    /// 0. `[signer]` The treasurer account (the creator of the treasury)
    /// 1. `[writable]` The treasury account
    /// 2. `[writable]` The treasury pool token mint account (The mint account of the treasury pool token issued by the treasury).
    /// 3. `[]` The Money Streaming Program operations account.
    /// 4. `[]` The Money Streaming Program account.
    /// 5. `[]` The Token Program account.    
    /// 6. `[]` System Program account.
    /// 7. `[]` SysvarRent account.
    CreateTreasury {
        slot: u64,
        label: String,
        treasury_type: u8
    },

    /// 0. `[signer]` The treasurer account (the creator of the treasury)
    /// 1. `[writable]` The treasurer token account
    /// 2. `[writable]` The treasurer treasury pool token account
    /// 3. `[]` The associated token account.
    /// 4. `[writable]` The treasury account
    /// 5. `[writable]` The treasury token account
    /// 6. `[writable]` The treasury pool mint account
    /// 7. `[]` The Money Streaming Operation account.
    /// 8. `[writable]` The Money Streaming Operation token account.
    /// 9. `[]` The Money Streaming Program account
    /// 10. `[]` The token program account
    CloseTreasury,

    /// 0. `[signer, writable]` The treasurer account (the creator of the treasury)
    /// 1. `[writable]` The treasury account
    /// 2. `[]` The treasury token account
    /// 3. `[]` The associated token mint account  
    /// 4. `[]` SysvarRent account.
    UpgradeTreasury,
}

impl StreamInstruction {

    pub fn unpack(instruction_data: &[u8]) -> Result<Self, StreamError> {

        let (&tag, result) = instruction_data
            .split_first()
            .ok_or(StreamError::InvalidStreamInstruction.into())?;
                
        Ok(match tag {

            0 => Self::unpack_create_stream(result)?,
            1 => Self::unpack_add_funds(result)?,
            2 => Self::unpack_recover_funds(result)?,
            3 => Self::unpack_withdraw(result)?,
            4 => Ok(Self::PauseStream)?,
            5 => Ok(Self::ResumeStream)?,
            6 => Self::unpack_propose_update(result)?,
            7 => Self::unpack_answer_update(result)?,
            8 => Self::unpack_close_stream(result)?,
            9 => Self::unpack_create_treasury(result)?,
            10 => Ok(Self::CloseTreasury)?,
            11 => Ok(Self::UpgradeTreasury)?,

            _ => return Err(StreamError::InvalidStreamInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {

            Self::CreateStream {
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

                buf.push(0);

                buf.extend_from_slice(stream_name.as_ref());
                buf.extend_from_slice(&rate_amount.to_le_bytes());
                buf.extend_from_slice(&rate_interval_in_seconds.to_le_bytes());
                buf.extend_from_slice(&allocation_reserved.to_le_bytes());
                buf.extend_from_slice(&allocation.to_le_bytes());
                buf.extend_from_slice(&funded_on_utc.to_le_bytes());
                buf.extend_from_slice(&start_utc.to_le_bytes());
                buf.extend_from_slice(&rate_cliff_in_seconds.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_amount.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_percent.to_le_bytes());
                buf.extend_from_slice(&auto_pause_in_seconds.to_le_bytes());               
            },

            &Self::AddFunds { 
                amount,
                allocation_type,
                allocation_stream_address

            } => {
                buf.push(1);

                buf.extend_from_slice(&amount.to_le_bytes());
                buf.extend_from_slice(&allocation_type.to_le_bytes());
                buf.extend_from_slice(&allocation_stream_address.as_ref());
            },

            &Self::RecoverFunds { amount } => {
                buf.push(2);
                buf.extend_from_slice(&amount.to_le_bytes());
            },

            &Self::Withdraw { amount } => {
                buf.push(3);
                buf.extend_from_slice(&amount.to_le_bytes());
            },

            &Self::PauseStream => buf.push(4),

            &Self::ResumeStream => buf.push(5),

            Self::ProposeUpdate {
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
                buf.push(6);

                buf.extend_from_slice(proposed_by.as_ref());
                buf.extend_from_slice(stream_name.as_ref());
                buf.extend_from_slice(treasurer_address.as_ref());
                buf.extend_from_slice(beneficiary_address.as_ref());
                buf.extend_from_slice(associated_token_address.as_ref());
                buf.extend_from_slice(&rate_amount.to_le_bytes());
                buf.extend_from_slice(&rate_interval_in_seconds.to_le_bytes());
                buf.extend_from_slice(&rate_cliff_in_seconds.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_amount.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_percent.to_le_bytes());
                buf.extend_from_slice(&auto_pause_in_seconds.to_le_bytes());                
            },

            &Self::AnswerUpdate { approve } => { 
                buf.push(7);

                let approve = match approve {
                    false => [0],
                    true => [1]
                };

                buf.push(approve[0] as u8);
            },

            &Self::CloseStream { auto_close_treasury } => {
                buf.push(8);

                let close_treasury = match auto_close_treasury {
                    false => [0],
                    true => [1]
                };

                buf.push(close_treasury[0] as u8);
            },
            
            Self::CreateTreasury {
                slot,
                label,
                treasury_type

            } => {

                buf.push(9);

                buf.extend_from_slice(&slot.to_le_bytes());
                buf.extend_from_slice(label.as_ref());
                buf.extend_from_slice(&treasury_type.to_le_bytes());
            },

            Self::CloseTreasury => buf.push(10),

            Self::UpgradeTreasury => buf.push(11),

        };

        buf
    }

    fn unpack_create_stream(input: &[u8]) -> Result<Self, StreamError> {

        let (stream_name, result) = unpack_string(input)?;
        let (rate_amount, result) = result.split_at(8);
        let rate_amount = unpack_f64(rate_amount)?;
        let (rate_interval_in_seconds, result) = result.split_at(8);
        let rate_interval_in_seconds = unpack_u64(rate_interval_in_seconds)?;
        let (allocation_reserved, result) = result.split_at(8);
        let allocation_reserved = unpack_f64(allocation_reserved)?;
        let (allocation, result) = result.split_at(8);
        let allocation = unpack_f64(allocation)?;
        let (funded_on_utc, result) = result.split_at(8);
        let funded_on_utc = unpack_u64(funded_on_utc)?;
        let (start_utc, result) = result.split_at(8);
        let start_utc = unpack_u64(start_utc)?;
        let (rate_cliff_in_seconds, result) = result.split_at(8);
        let rate_cliff_in_seconds = unpack_u64(rate_cliff_in_seconds)?;
        let (cliff_vest_amount, result) = result.split_at(8);
        let cliff_vest_amount = unpack_f64(cliff_vest_amount)?;
        let (cliff_vest_percent, result) = result.split_at(8);
        let cliff_vest_percent = unpack_f64(cliff_vest_percent)?;
        let (auto_pause_in_seconds, _result) = result.split_at(8);
        let auto_pause_in_seconds = unpack_u64(auto_pause_in_seconds)?;

        Ok(Self::CreateStream {
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
        })
    }

    fn unpack_add_funds(input: &[u8]) -> Result<Self, StreamError> {

        let (amount, result) = input.split_at(8);
        let amount = unpack_f64(amount)?;
        let (allocation_type, result) = result.split_at(1);
        let allocation_type = unpack_u8(allocation_type)?;
        let (allocation_stream_address, _result) = unpack_pubkey(result)?;

        Ok(Self::AddFunds { 
            amount,
            allocation_type,
            allocation_stream_address
        })
    }

    fn unpack_recover_funds(input: &[u8]) -> Result<Self, StreamError> {
        let (amount, _result) = input.split_at(8);
        let amount = unpack_f64(amount)?;

        Ok(Self::RecoverFunds { amount })
    }

    fn unpack_withdraw(input: &[u8]) -> Result<Self, StreamError> {

        let (amount, _result) = input.split_at(8);
        let amount = unpack_f64(amount)?;

        Ok(Self::Withdraw { amount })
    }

    fn unpack_propose_update(input: &[u8]) -> Result<Self, StreamError> {
        let (proposed_by, result) = unpack_pubkey(input)?;
        let (stream_name, result) = unpack_string(result)?;
        let (treasurer_address, result) = unpack_pubkey(result)?;
        let (beneficiary_address, result) = unpack_pubkey(result)?;
        let (associated_token_address, result) = unpack_pubkey(result)?;

        let (rate_amount, result) = result.split_at(8);
        let rate_amount = unpack_f64(rate_amount)?;

        let (rate_interval_in_seconds, result) = result.split_at(8);
        let rate_interval_in_seconds = unpack_u64(rate_interval_in_seconds)?;

        let (rate_cliff_in_seconds, result) = result.split_at(8);
        let rate_cliff_in_seconds = unpack_u64(rate_cliff_in_seconds)?;

        let (cliff_vest_amount, result) = result.split_at(8);
        let cliff_vest_amount = unpack_f64(cliff_vest_amount)?;

        let (cliff_vest_percent, result) = result.split_at(8);
        let cliff_vest_percent = unpack_f64(cliff_vest_percent)?;

        let (auto_pause_in_seconds, _result) = result.split_at(8);
        let auto_pause_in_seconds = unpack_u64(auto_pause_in_seconds)?;        

        Ok(Self::ProposeUpdate {
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
        })
    }

    fn unpack_answer_update(input: &[u8]) -> Result<Self, StreamError> {
        let (approve, _result) = input.split_at(1);
        let approve = match approve {
            [0] => false,
            [1] => true,
            _ => false
        };

        Ok(Self::AnswerUpdate { approve })
    }

    fn unpack_create_treasury(input: &[u8]) -> Result<Self, StreamError> {

        let (slot, result) = input.split_at(8);
        let slot = unpack_u64(slot)?;        
        let (label, result) = unpack_string(result)?;
        let (treasury_type, _result) = result.split_at(1);
        let treasury_type = unpack_u8(treasury_type)?;


        Ok(Self::CreateTreasury { 
            slot,
            label,
            treasury_type
        })
    }

    fn unpack_close_stream(input: &[u8]) -> Result<Self, StreamError> {
        let (auto_close_treasury, _result) = input.split_at(1);
        let auto_close_treasury = match auto_close_treasury {
            [0] => false,
            [1] => true,
            _ => false
        };

        Ok(Self::CloseStream { auto_close_treasury })
    }
 }

 pub fn create_stream(
    program_id: &Pubkey,
    treasurer_address: Pubkey,
    beneficiary_address: Pubkey,
    beneficiary_token_mint_address: Pubkey,
    treasury_address: Pubkey,
    stream_address: Pubkey,
    msp_ops_address: Pubkey,
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

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::CreateStream {
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

    }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(treasurer_address, true),
        AccountMeta::new_readonly(treasury_address, false),
        AccountMeta::new_readonly(beneficiary_token_mint_address, false),
        AccountMeta::new(stream_address, false),
        AccountMeta::new_readonly(beneficiary_address, false),
        AccountMeta::new(msp_ops_address, false),
        AccountMeta::new_readonly(*program_id, false),
        // AccountMeta::new_readonly(solana_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false)
    ];

    Ok(Instruction { 
        program_id: *program_id, 
        accounts, 
        data 
    })
 }

 pub fn add_funds(
    program_id: &Pubkey,
    stream_address: &Pubkey,
    treasury_address: &Pubkey,
    contribution_token_address: &Pubkey,
    amount: f64,
    allocation_type: u8,
    allocation_stream_address: Pubkey

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::AddFunds { 
        amount,
        allocation_type,
        allocation_stream_address

    }.pack();

    let accounts = vec![
        AccountMeta::new(*contribution_token_address, true),
        AccountMeta::new(*stream_address, false),
        AccountMeta::new_readonly(*treasury_address, false)
    ];

    Ok(Instruction { 
        program_id: *program_id, 
        accounts, 
        data 
    })
 }

 pub fn withdraw(
    program_id: &Pubkey,
    beneficiary_address: Pubkey,
    beneficiary_token_address: Pubkey,
    beneficiary_token_mint_address: Pubkey,
    treasury_address: Pubkey,
    treasury_token_address: Pubkey,
    stream_account_address: Pubkey,
    msp_ops_token_address: Pubkey,
    amount: f64,

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::Withdraw { amount }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(beneficiary_address, false),
        AccountMeta::new(beneficiary_token_address, false),
        AccountMeta::new(beneficiary_token_mint_address, false),
        AccountMeta::new(treasury_address, false),
        AccountMeta::new(treasury_token_address, false),
        AccountMeta::new(stream_account_address, false),
        AccountMeta::new(msp_ops_token_address, false),
        AccountMeta::new_readonly(*program_id, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction { 
        program_id: *program_id, 
        accounts, 
        data 
    })
 }

 pub fn close_stream(
    initializer_account_key: &Pubkey,
    stream_account_key: &Pubkey,
    counterparty_account_key: &Pubkey,
    treasury_account_key: &Pubkey,
    auto_close_treasury: bool,
    program_id: &Pubkey,

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::CloseStream { auto_close_treasury }.pack();
    let accounts = vec![
        AccountMeta::new(*initializer_account_key, true),
        AccountMeta::new(*stream_account_key, false),
        AccountMeta::new_readonly(*counterparty_account_key, false),
        AccountMeta::new_readonly(*treasury_account_key, false)
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }

 pub fn close_treasury(
    treasurer_account_address: Pubkey,
    treasurer_token_account_address: Pubkey,
    treasurer_treasury_pool_token_account_address: Pubkey,
    associated_token_mint_address: Pubkey,
    treasury_account_address: Pubkey,
    treasury_token_account_address: Pubkey,
    treasury_pool_mint_address: Pubkey,
    msp_ops_account_address: Pubkey,
    msp_ops_token_account_address: Pubkey,
    token_program_account_address: Pubkey,
    program_id: &Pubkey,

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::CloseTreasury.pack();
    let accounts = vec![
        AccountMeta::new(treasurer_account_address, true),
        AccountMeta::new(treasurer_token_account_address, false),
        AccountMeta::new(treasurer_treasury_pool_token_account_address, false),
        AccountMeta::new_readonly(associated_token_mint_address, false),
        AccountMeta::new(treasury_account_address, false),
        AccountMeta::new(treasury_token_account_address, false),
        AccountMeta::new(treasury_pool_mint_address, false),
        AccountMeta::new_readonly(msp_ops_account_address, false),
        AccountMeta::new(msp_ops_token_account_address, false),
        AccountMeta::new_readonly(*program_id, false),
        AccountMeta::new_readonly(token_program_account_address, false)
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }
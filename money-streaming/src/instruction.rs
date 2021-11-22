// Program API, (de)serializing instruction data

use std::{ mem::size_of, convert::TryInto };

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
    /// 4.  [writable] The Money Streaming Program operating account (Fees account).
    /// 5.  [] The Money Streaming Program account.
    /// 6. `[]` The System Program account.
    /// 7. `[]` Rent sysvar account.
    CreateStream {
        beneficiary_address: Pubkey,
        stream_name: String,        
        rate_amount: f64,
        rate_interval_in_seconds: u64,
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
    /// 2. `[writable]` The contributor treasury token account (the account of the token issued by the treasury and owned by the contributor)
    /// 3. `[]` The beneficiary mint account
    /// 4. `[]` The treasury account (Stream treasury account).
    /// 5. `[writable]` The treasury token account.    
    /// 6. `[]` The treasury mint account (the mint of the treasury pool token)
    /// 7. `[writable]` The stream account (The stream contract account).
    /// 8.  [writable] The Money Streaming Program operating account (Fees account).
    /// 9.  [] The Money Streaming Program account.
    /// 10. `[]` The Associated Token Program account.
    /// 11. `[]` The Token Program account.
    /// 12. `[]` The System Program account.
    /// 13. `[]` Rent sysvar account.
    AddFunds {
        contribution_amount: f64,
        funded_on_utc: u64,
        resume: bool
    },

    /// Recovers a specific amount of funds from a previously funded stream
    ///
    /// 0. `[signer]` The contributor account
    /// 1. `[writable]` The contributor token account
    /// 2. `[writable]` The contributor treasury token account (the account of the token issued by the treasury and owned by the contributor)
    /// 3. `[]` The contributor mint account
    /// 4. `[]` The treasury account (Stream treasury account).
    /// 5. `[writable]` The treasury token account.    
    /// 6. `[]` The treasury mint account (the mint of the treasury pool token)
    /// 7. `[writable]` The stream account (The stream contract account).
    /// 8.  [writable] The Money Streaming Program operating account (Fees account).
    /// 9.  [writable] The Money Streaming Program operating token account.
    /// 10.  [] The Money Streaming Program account.
    /// 11. `[]` The Token Program account.
    RecoverFunds {
        recover_amount: f64
    },

    /// 0. `[signer]` The beneficiary account
    /// 1. `[writable]` The beneficiary token account (the recipient of the money)
    /// 2. `[]` The beneficiary token mint account
    /// 3. `[]` The treasury account
    /// 4. `[writable]` The treasury token account
    /// 5. `[writable]` The stream account (The stream contract account).
    /// 6.  [writable] The Money Streaming Program operating account (Fees account).
    /// 7.  [writable] The Money Streaming Program operating token account.
    /// 8. `[]` The Money Streaming Program account.
    /// 9. `[]` The Token Program account.
    Withdraw { 
        withdrawal_amount: f64
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

    /// 0. `[signer]` The initializer account (treasurer/beneficiary)
    /// 1. `[writable]` The treasurer account (the creator of the treasury)
    /// 2. `[writable]` The beneficiary token account (the recipient of the money)
    /// 3. `[]` The beneficiary token mint account
    /// 4. `[writable]` The treasury account
    /// 5. `[writable]` The treasury token account
    /// 6. `[writable]` The stream account (The stream contract account).
    /// 7. `[writable]` The Money Streaming Program operating account (Fees account).
    /// 8. `[writable]` The Money Streaming Program operating token account.
    /// 9. `[writable]` The Money Streaming Program account
    /// 10. `[]` The Token Program account.
    /// 11. `[]` System Program account.
    CloseStream,

    /// 0. `[signer]` The treasurer account (the creator of the treasury)
    /// 1. `[writable]` The treasury account
    /// 2. `[writable]` The treasury token account (The token account of the treasury which the funds are going to be payed for)
    /// 3. `[writable]` The treasury token mint account (The mint account of the treasury token which the funds are going to be payed for).
    /// 4. `[writable]` The treasury mint account (The mint account of the treasury pool token issued by the treasury).
    /// 5. `[writable]` The Money Streaming Program operating account (Fees account).
    /// 6. `[writable]` The Money Streaming Protocol operating token account.
    /// 7. `[]` The Associated Token Program account.
    /// 8. `[]` The Token Program account.    
    /// 9. `[]` System Program account.
    /// 10. `[]` SysvarRent account.
    CreateTreasury {
        treasury_block_height: u64,
        treasury_base_address: Pubkey
    },

    /// 0. `[signer]` The treasurer account (the creator of the treasury)
    /// 1. `[writable]` The treasury account
    /// 2. `[writable]` The treasury token account
    /// 5. `[writable]` The Money Streaming Program operating account (Fees account).
    /// 6. `[writable]` The Money Streaming Protocol operating token account.
    /// 7. `[]` The Associated Token Program account.
    /// 8. `[]` The Token Program account.    
    /// 9. `[]` System Program account.
    /// 10. `[]` SysvarRent account.
    CreateTreasuryV2 {
        block_height: u64,
        base_address: Pubkey,
        tag: String,
        amount: f64,
        is_reserved: bool 
    },
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
            8 => Ok(Self::CloseStream)?,
            9 => Self::unpack_create_treasury(result)?,

            _ => return Err(StreamError::InvalidStreamInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {

            Self::CreateStream {
                beneficiary_address,
                stream_name,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent,
                auto_pause_in_seconds

            } => {

                buf.push(0);

                buf.extend_from_slice(beneficiary_address.as_ref());
                buf.extend_from_slice(stream_name.as_ref());
                buf.extend_from_slice(&rate_amount.to_le_bytes());
                buf.extend_from_slice(&rate_interval_in_seconds.to_le_bytes());
                buf.extend_from_slice(&start_utc.to_le_bytes());
                buf.extend_from_slice(&rate_cliff_in_seconds.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_amount.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_percent.to_le_bytes());
                buf.extend_from_slice(&auto_pause_in_seconds.to_le_bytes());               
            },

            &Self::AddFunds { 
                contribution_amount,
                funded_on_utc,
                resume

            } => {
                buf.push(1);

                buf.extend_from_slice(&contribution_amount.to_le_bytes());
                buf.extend_from_slice(&funded_on_utc.to_le_bytes());
                
                let resume = match resume {
                    false => [0],
                    true => [1]
                };

                buf.push(resume[0] as u8);
            },

            &Self::RecoverFunds { recover_amount } => {
                buf.push(2);
                buf.extend_from_slice(&recover_amount.to_le_bytes());
            },

            &Self::Withdraw { withdrawal_amount } => {
                buf.push(3);
                buf.extend_from_slice(&withdrawal_amount.to_le_bytes());
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

            Self::CloseStream => buf.push(8),
            
            Self::CreateTreasury {
                treasury_block_height,
                treasury_base_address

            } => {
                buf.push(9);

                buf.extend_from_slice(&treasury_block_height.to_le_bytes());
                buf.extend_from_slice(treasury_base_address.as_ref());
            },

            Self::CreateTreasuryV2 {
                block_height,
                base_address,
                tag,
                amount,
                is_reserved 

            } => {
                buf.push(10);

                buf.extend_from_slice(&block_height.to_le_bytes());
                buf.extend_from_slice(base_address.as_ref());
                buf.extend_from_slice(tag.as_ref());
                buf.extend_from_slice(&amount.to_le_bytes());

                let is_reserved = match is_reserved {
                    false => 0,
                    true => 1
                };

                buf.push(is_reserved as u8);
            },
        };

        buf
    }

    fn unpack_create_stream(input: &[u8]) -> Result<Self, StreamError> {

        let (beneficiary_address, result) = unpack_pubkey(input)?;
        let (stream_name, result) = unpack_string(result)?;

        let (rate_amount, result) = result.split_at(8);
        let rate_amount = unpack_f64(rate_amount)?;

        let (rate_interval_in_seconds, result) = result.split_at(8);
        let rate_interval_in_seconds = unpack_u64(rate_interval_in_seconds)?;

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
            beneficiary_address,
            stream_name,
            rate_amount,
            rate_interval_in_seconds,
            start_utc,
            rate_cliff_in_seconds,
            cliff_vest_amount,
            cliff_vest_percent,
            auto_pause_in_seconds
        })
    }

    fn unpack_add_funds(input: &[u8]) -> Result<Self, StreamError> {
        let (contribution_amount, result) = input.split_at(8);
        let contribution_amount = unpack_f64(contribution_amount)?;
        let (funded_on_utc, result) = result.split_at(8);
        let funded_on_utc = unpack_u64(funded_on_utc)?;

        let (resume, _result) = result.split_at(1);
        let resume = match resume {
            [0] => false,
            [1] => true,
            _ => false
        };

        Ok(Self::AddFunds { 
            contribution_amount,
            funded_on_utc,
            resume
        })
    }

    fn unpack_recover_funds(input: &[u8]) -> Result<Self, StreamError> {
        let (recover_amount, _result) = input.split_at(8);
        let recover_amount = unpack_f64(recover_amount)?;

        Ok(Self::RecoverFunds { recover_amount })
    }

    fn unpack_withdraw(input: &[u8]) -> Result<Self, StreamError> {
        let (withdrawal_amount, _result) = input.split_at(8);
        let withdrawal_amount = unpack_f64(withdrawal_amount)?;

        Ok(Self::Withdraw { withdrawal_amount })
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

        let (treasury_block_height, result) = input.split_at(8);
        let treasury_block_height = unpack_u64(treasury_block_height)?;

        let (treasury_base_address, _result) = unpack_pubkey(result)?;

        Ok(Self::CreateTreasury { 
            treasury_block_height,
            treasury_base_address
        })
    }

    fn unpack_create_treasury_v2(input: &[u8]) -> Result<Self, StreamError> {

        let (block_height, result) = input.split_at(8);
        let block_height = unpack_u64(block_height)?;

        let (base_address, result) = unpack_pubkey(result)?;
        let (tag, result) = unpack_string(result)?;

        let (amount, result) = input.split_at(8);
        let amount = unpack_f64(amount)?;

        let (is_reserved, _result) = result.split_at(1);
        let is_reserved = match is_reserved {
            [0] => false,
            [1] => true,
            _ => false
        };

        Ok(Self::CreateTreasuryV2 { 
            block_height,
            base_address,
            tag,
            amount,
            is_reserved
        })
    }
 }

 pub fn create_stream(
    program_id: &Pubkey,
    treasurer_address: Pubkey,
    beneficiary_address: Pubkey,
    beneficiary_mint_address: Pubkey,
    treasury_address: Pubkey,
    stream_address: Pubkey,
    msp_ops_address: Pubkey,
    stream_name: String,
    rate_amount: f64,
    rate_interval_in_seconds: u64,
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
        beneficiary_address,
        stream_name,
        rate_amount,
        rate_interval_in_seconds,
        start_utc,
        rate_cliff_in_seconds,
        cliff_vest_amount,
        cliff_vest_percent,
        auto_pause_in_seconds

    }.pack();

    let accounts = vec![
        AccountMeta::new_readonly(treasurer_address, true),
        AccountMeta::new_readonly(treasury_address, false),
        AccountMeta::new_readonly(beneficiary_mint_address, false),
        AccountMeta::new(stream_address, false),
        AccountMeta::new(msp_ops_address, false),
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
    contribution_token_address: Pubkey,
    contribution_amount: f64,
    funded_on_utc: u64,
    resume: bool

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::AddFunds { 
        contribution_amount,
        funded_on_utc,
        resume

    }.pack();

    let accounts = vec![
        AccountMeta::new(contribution_token_address, true),
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
    beneficiary_account_address: Pubkey,
    stream_account_address: Pubkey,
    treasury_account_address: Pubkey,
    withdrawal_amount: f64,

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::Withdraw { withdrawal_amount }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(beneficiary_account_address, false),
        AccountMeta::new(stream_account_address, false),
        AccountMeta::new_readonly(treasury_account_address, false)
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
    program_id: &Pubkey,

 ) -> Result<Instruction, StreamError> {

    if let Err(_error) = check_program_account(program_id) {
        return Err(StreamError::IncorrectProgramId.into());
    }

    let data = StreamInstruction::CloseStream.pack();
    let accounts = vec![
        AccountMeta::new(*initializer_account_key, true),
        AccountMeta::new(*stream_account_key, false),
        AccountMeta::new_readonly(*counterparty_account_key, false),
        AccountMeta::new_readonly(*treasury_account_key, false)
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }
// Program API, (de)serializing instruction data

use std::{
    mem::size_of,
    convert::TryInto,
    // fmt::Display,
};

use solana_program::{
    msg,
    pubkey::Pubkey,
    // program_error::ProgramError,
    instruction::{ AccountMeta, Instruction }
};

use crate::{
    check_program_account,
    error::StreamError
};

pub enum StreamInstruction {

    /// 0. `[signer]` The treasurer account (the creator of the money stream) 
    /// 1. `[]` The beneficiary account (the recipient of the money)
    /// 2. `[]` The treasury account (Money stream treasury account).
    /// 3. `[writable]` The stream account (Money stream state account).
    /// 4. `[]` The treasurer authority account (The owner of the account).
    CreateStream {
        stream_name: String,
        treasurer_address: Pubkey,
        beneficiary_withdrawal_address: Pubkey,
        escrow_token_address: Pubkey,
        treasury_address: Pubkey,
        funding_amount: u64, // OPTIONAL
        rate_amount: u64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64,
        cliff_vest_amount: u64, // OPTIONAL
        cliff_vest_percent: u64, // OPTIONAL
    },

    /// 0. `[signer]` The contributor token account
    /// 1. `[writable]` The stream account (Money stream state account).
    /// 2. `[]` The treasury account (Money stream treasury account).
    AddFunds {
        contribution_token_address: Pubkey,
        contribution_amount: u64
    },

    /// 0. `[signer]` The beneficiary account (the recipient of the money)
    /// 1. `[writable]` The stream account.
    /// 2. `[]` The treasury account.
    Withdraw { 
        withdrawal_amount: u64
    },

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream terms account (Update proposal account).
    /// 2. `[]` The counterparty's account (if the initializer is the treasurer then it would be the beneficiary or vice versa)
    /// 3. `[writable]` The stream account
    ProposeUpdate {
        proposed_by: Pubkey,
        stream_name: String,
        treasurer_address: Pubkey,
        beneficiary_withdrawal_address: Pubkey,
        escrow_token_address: Pubkey, // OPTIONAL
        treasury_address: Pubkey,
        rate_amount: u64,
        rate_interval_in_seconds: u64,
        start_utc: u64,
        rate_cliff_in_seconds: u64
    },

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream account.
    /// 2. `[]` The counterparty's account (if the initializer is the treasurer then it would be the beneficiary or vice versa)
    /// 3. `[writable]` The stream terms account (Update proposal account)
    AnswerUpdate {
        answer: bool
    },

    /// 0. `[signer]` The initializer of the transaction (treasurer or beneficiary)
    /// 1. `[writable]` The stream account (Money stream state account)
    /// 2. `[]` The counterparty's account (if the initializer is the treasurer then it would be the beneficiary or vice versa)
    /// 3. `[]` The treasury account (Money stream treasury account).
    /// 4. `[]` The MeanFi Operations account
    CloseStream,

    /// 0. `[signer]` The treasurer account (the creator of the money stream)
    /// 1. `[writable]` The treasury account (Money stream treasury account).
    CloseTreasury,
}

impl StreamInstruction {

    pub fn unpack(instruction_data: &[u8]) -> Result<Self, StreamError> {

        let (&tag, result) = instruction_data
            .split_first()
            .ok_or(StreamError::InvalidStreamInstruction.into())?;
        
        Ok(match tag {

            0 => Self::unpack_create_stream(result)?,
            1 => Self::unpack_add_funds(result)?,
            2 => Self::unpack_withdraw(result)?,
            3 => Self::unpack_propose_update(result)?,
            4 => Self::unpack_answer_update(result)?,
            5 => Ok(Self::CloseStream)?,
            6 => Ok(Self::CloseTreasury)?,

            _ => return Err(StreamError::InvalidStreamInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());

        match self {

            Self::CreateStream {
                stream_name,
                treasurer_address,
                beneficiary_withdrawal_address,
                escrow_token_address,
                treasury_address,
                funding_amount,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds,
                cliff_vest_amount,
                cliff_vest_percent

            } => {

                buf.push(0);

                buf.extend_from_slice(stream_name.as_ref());
                buf.extend_from_slice(treasurer_address.as_ref());
                buf.extend_from_slice(beneficiary_withdrawal_address.as_ref());
                buf.extend_from_slice(escrow_token_address.as_ref());
                buf.extend_from_slice(treasury_address.as_ref());
                buf.extend_from_slice(&funding_amount.to_le_bytes());
                buf.extend_from_slice(&rate_amount.to_le_bytes());
                buf.extend_from_slice(&rate_interval_in_seconds.to_le_bytes());
                buf.extend_from_slice(&start_utc.to_le_bytes());
                buf.extend_from_slice(&rate_cliff_in_seconds.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_amount.to_le_bytes());
                buf.extend_from_slice(&cliff_vest_percent.to_le_bytes());
            },

            &Self::AddFunds {
                contribution_amount,
                contribution_token_address

            } => {
                buf.push(1);

                buf.extend_from_slice(contribution_token_address.as_ref());
                buf.extend_from_slice(&contribution_amount.to_le_bytes());   
            },

            &Self::Withdraw { 
                withdrawal_amount
            } => {
                buf.push(2);
                buf.extend_from_slice(&withdrawal_amount.to_le_bytes());
            },

            Self::ProposeUpdate {
                proposed_by,
                stream_name,
                treasurer_address,
                treasury_address,
                beneficiary_withdrawal_address,
                escrow_token_address,
                rate_amount,
                rate_interval_in_seconds,
                start_utc,
                rate_cliff_in_seconds

            } => {
                buf.push(3);

                buf.extend_from_slice(proposed_by.as_ref());
                buf.extend_from_slice(stream_name.as_ref());
                buf.extend_from_slice(treasurer_address.as_ref());
                buf.extend_from_slice(treasury_address.as_ref());
                buf.extend_from_slice(beneficiary_withdrawal_address.as_ref());
                buf.extend_from_slice(escrow_token_address.as_ref());
                buf.extend_from_slice(&rate_amount.to_le_bytes());
                buf.extend_from_slice(&rate_interval_in_seconds.to_le_bytes());
                buf.extend_from_slice(&start_utc.to_le_bytes());
                buf.extend_from_slice(&rate_cliff_in_seconds.to_le_bytes());
            },

            &Self::AnswerUpdate { answer } => { 
                buf.push(4);

                let answer = match answer {
                    false => [0],
                    true => [1]
                };

                buf.push(answer[0] as u8);
            },

            &Self::CloseStream  => buf.push(5),

            &Self::CloseTreasury => buf.push(6),
        };

        buf
    }

    fn unpack_create_stream(input: &[u8]) -> Result<Self, StreamError> {

        let (stream_name, result) = Self::unpack_string(input)?;
        let (treasurer_address, result) = Self::unpack_pubkey(result)?;
        let (beneficiary_withdrawal_address, result) = Self::unpack_pubkey(result)?;
        let (escrow_token_address, result) = Self::unpack_pubkey(result)?; 
        let (treasury_address, result) = Self::unpack_pubkey(result)?; 

        let (funding_amount, result) = result.split_at(8);
        let funding_amount = Self::unpack_u64(funding_amount)?;

        let (rate_amount, result) = result.split_at(8);
        let rate_amount = Self::unpack_u64(rate_amount)?;

        let (rate_interval_in_seconds, result) = result.split_at(8);
        let rate_interval_in_seconds = Self::unpack_u64(rate_interval_in_seconds)?;

        let (start_utc, result) = result.split_at(8);
        let start_utc = Self::unpack_u64(start_utc)?;

        let (rate_cliff_in_seconds, result) = result.split_at(8);
        let rate_cliff_in_seconds = Self::unpack_u64(rate_cliff_in_seconds)?;

        let (cliff_vest_amount, result) = result.split_at(8);
        let cliff_vest_amount = Self::unpack_u64(cliff_vest_amount)?;

        let (cliff_vest_percent, _result) = result.split_at(8);
        let cliff_vest_percent = Self::unpack_u64(cliff_vest_percent)?;        

        Ok(Self::CreateStream {
            stream_name,
            treasurer_address,
            treasury_address,
            beneficiary_withdrawal_address,
            escrow_token_address,
            funding_amount,
            rate_amount,
            rate_interval_in_seconds,
            start_utc,
            rate_cliff_in_seconds,
            cliff_vest_amount,
            cliff_vest_percent
        })
    }

    fn unpack_add_funds(input: &[u8]) -> Result<Self, StreamError> {
        let (contribution_token_address, result) = Self::unpack_pubkey(input)?;
        let (contribution_amount, _result) = result.split_at(8);
        let contribution_amount = Self::unpack_u64(contribution_amount)?;

        Ok(Self::AddFunds { 
            contribution_token_address,
            contribution_amount
        })
    }

    fn unpack_withdraw(input: &[u8]) -> Result<Self, StreamError> {
        let (withdrawal_amount, _result) = input.split_at(8);
        let withdrawal_amount = Self::unpack_u64(withdrawal_amount)?;

        Ok(Self::Withdraw { withdrawal_amount })
    }

    fn unpack_propose_update(input: &[u8]) -> Result<Self, StreamError> {
        let (proposed_by, result) = Self::unpack_pubkey(input)?;
        let (stream_name, result) = Self::unpack_string(result)?;
        let (treasurer_address, result) = Self::unpack_pubkey(result)?;
        let (treasury_address, result) = Self::unpack_pubkey(result)?;
        let (beneficiary_withdrawal_address, result) = Self::unpack_pubkey(result)?;
        let (escrow_token_address, result) = Self::unpack_pubkey(result)?;

        let (rate_amount, result) = result.split_at(8);
        let rate_amount = Self::unpack_u64(rate_amount)?;

        let (rate_interval_in_seconds, result) = result.split_at(8);
        let rate_interval_in_seconds = Self::unpack_u64(rate_interval_in_seconds)?;

        let (start_utc, result) = result.split_at(8);
        let start_utc = Self::unpack_u64(start_utc)?;

        let (rate_cliff_in_seconds, _result) = result.split_at(8);
        let rate_cliff_in_seconds = Self::unpack_u64(rate_cliff_in_seconds)?;

        Ok(Self::ProposeUpdate {
            proposed_by,
            stream_name,
            treasurer_address,
            treasury_address,
            beneficiary_withdrawal_address,
            escrow_token_address,
            rate_amount,
            rate_interval_in_seconds,
            start_utc,
            rate_cliff_in_seconds
        })
    }

    fn unpack_answer_update(input: &[u8]) -> Result<Self, StreamError> {
        let (answer, _result) = input.split_at(1);
        let answer = match answer {
            [0] => false,
            [1] => true,
            _ => false
        };

        Ok(Self::AnswerUpdate { answer })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), StreamError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);

            Ok((pk, rest))
        } else {
            Err(StreamError::InvalidArgument.into())
        }
    }

    fn unpack_string(input: &[u8]) -> Result<(String, &[u8]), StreamError> {
        if input.len() >= 32 {
            let (bytes, rest) = input.split_at(32);
            Ok((String::from_utf8_lossy(bytes).to_string(), rest))
        } else {
            Err(StreamError::InvalidArgument.into())
        }
    }

    fn unpack_u64(input: &[u8]) -> Result<u64, StreamError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(StreamError::InvalidStreamInstruction)?;

        Ok(amount)
    }
 }

 pub fn create_stream(
    stream_account_key: &Pubkey,
    program_id: &Pubkey,
    stream_name: String,
    treasurer_address: Pubkey,
    beneficiary_withdrawal_address: Pubkey,
    escrow_token_address: Pubkey,
    treasury_address: Pubkey,
    funding_amount: u64,
    rate_amount: u64,
    rate_interval_in_seconds: u64,
    start_utc: u64,
    rate_cliff_in_seconds: u64,
    cliff_vest_amount: u64,
    cliff_vest_percent: u64,

 ) -> Result<Instruction, StreamError> {

    check_program_account(program_id);

    let data = StreamInstruction::CreateStream {
        stream_name,
        treasurer_address,
        beneficiary_withdrawal_address,
        escrow_token_address,
        treasury_address,
        funding_amount,
        rate_amount,
        rate_interval_in_seconds,
        start_utc,
        rate_cliff_in_seconds,
        cliff_vest_amount,
        cliff_vest_percent,

    }.pack();

    let accounts = vec![
        AccountMeta::new(treasurer_address, true),
        AccountMeta::new_readonly(beneficiary_withdrawal_address, false),
        AccountMeta::new_readonly(treasury_address, false),
        AccountMeta::new(*stream_account_key, false),
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }

 pub fn add_funds(
    stream_account_key: &Pubkey,
    treasury_account_key: &Pubkey,
    program_id: &Pubkey,
    contribution_token_address: Pubkey,
    contribution_amount: u64,

 ) -> Result<Instruction, StreamError> {

    check_program_account(program_id);

    let data = StreamInstruction::AddFunds {
        contribution_token_address,
        contribution_amount,

    }.pack();

    let accounts = vec![
        AccountMeta::new(contribution_token_address, true),
        AccountMeta::new(*stream_account_key, false),
        AccountMeta::new_readonly(*treasury_account_key, false)
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }

 pub fn close_stream(
    initializer_account_key: &Pubkey,
    stream_account_key: &Pubkey,
    counterparty_account_key: &Pubkey,
    treasury_account_key: &Pubkey,
    program_id: &Pubkey,

 ) -> Result<Instruction, StreamError> {

    check_program_account(program_id);

    let data = StreamInstruction::CloseStream.pack();
    let accounts = vec![
        AccountMeta::new(*initializer_account_key, true),
        AccountMeta::new(*stream_account_key, false),
        AccountMeta::new_readonly(*counterparty_account_key, false),
        AccountMeta::new_readonly(*treasury_account_key, false)
    ];

    Ok(Instruction { program_id: *program_id, accounts, data })
 }

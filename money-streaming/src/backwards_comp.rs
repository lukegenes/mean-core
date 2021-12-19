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

/// Stream V0
#[derive(Clone, Debug)]
pub struct Stream {
    pub initialized: bool,
    pub stream_name: String,
    pub treasurer_address: Pubkey,
    pub rate_amount: f64,
    pub rate_interval_in_seconds: u64,
    pub funded_on_utc: u64,
    pub start_utc: u64,
    pub rate_cliff_in_seconds: u64,
    pub cliff_vest_amount: f64,
    pub cliff_vest_percent: f64,
    pub beneficiary_address: Pubkey,
    pub beneficiary_associated_token: Pubkey,
    pub treasury_address: Pubkey,
    pub treasury_estimated_depletion_utc: u64,
    pub total_deposits: f64,
    pub total_withdrawals: f64,
    pub escrow_vested_amount_snap: f64,
    pub escrow_vested_amount_snap_block_height: u64,
    pub escrow_vested_amount_snap_block_time: u64,
    pub stream_resumed_block_height: u64,
    pub stream_resumed_block_time: u64,
    pub auto_pause_in_seconds: u64
}
impl Pack for Stream {
    const LEN: usize = 289;
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Stream::LEN];
        let (
            initialized,
            stream_name,
            treasurer_address,
            rate_amount,
            rate_interval_in_seconds,
            funded_on_utc,
            start_utc,
            rate_cliff_in_seconds,
            cliff_vest_amount,
            cliff_vest_percent,
            beneficiary_address,
            beneficiary_associated_token,
            treasury_address,
            treasury_estimated_depletion_utc,
            total_deposits,
            total_withdrawals,
            escrow_vested_amount_snap,
            escrow_vested_amount_snap_block_height,
            escrow_vested_amount_snap_block_time,
            stream_resumed_block_height,
            stream_resumed_block_time,
            auto_pause_in_seconds
            
        ) = array_refs![input, 1, 32, 32, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        Ok(Stream {
            initialized, 
            stream_name: String::from_utf8_lossy(stream_name).to_string(),
            treasurer_address: Pubkey::new_from_array(*treasurer_address),                   
            rate_amount: f64::from_le_bytes(*rate_amount),
            rate_interval_in_seconds: u64::from_le_bytes(*rate_interval_in_seconds),
            funded_on_utc: u64::from_le_bytes(*funded_on_utc),
            start_utc: u64::from_le_bytes(*start_utc),
            rate_cliff_in_seconds: u64::from_le_bytes(*rate_cliff_in_seconds),
            cliff_vest_amount: f64::from_le_bytes(*cliff_vest_amount),
            cliff_vest_percent: f64::from_le_bytes(*cliff_vest_percent),
            beneficiary_address: Pubkey::new_from_array(*beneficiary_address),
            beneficiary_associated_token: Pubkey::new_from_array(*beneficiary_associated_token),
            treasury_address: Pubkey::new_from_array(*treasury_address), 
            treasury_estimated_depletion_utc: u64::from_le_bytes(*treasury_estimated_depletion_utc),
            total_deposits: f64::from_le_bytes(*total_deposits),
            total_withdrawals: f64::from_le_bytes(*total_withdrawals),
            escrow_vested_amount_snap: f64::from_le_bytes(*escrow_vested_amount_snap),
            escrow_vested_amount_snap_block_height: u64::from_le_bytes(*escrow_vested_amount_snap_block_height),
            escrow_vested_amount_snap_block_time: u64::from_le_bytes(*escrow_vested_amount_snap_block_time),
            stream_resumed_block_height: u64::from_le_bytes(*stream_resumed_block_height),
            stream_resumed_block_time: u64::from_le_bytes(*stream_resumed_block_time),
            auto_pause_in_seconds: u64::from_le_bytes(*auto_pause_in_seconds)
        })
    }
}

/// StreamV1
#[derive(Clone, Debug)]
pub struct StreamV1 {
    pub initialized: bool,
    pub stream_name: String,
    pub treasurer_address: Pubkey,
    pub rate_amount: f64,
    pub rate_interval_in_seconds: u64,
    pub funded_on_utc: u64,
    pub start_utc: u64,
    pub rate_cliff_in_seconds: u64,
    pub cliff_vest_amount: f64,
    pub cliff_vest_percent: f64,
    pub beneficiary_address: Pubkey,
    pub beneficiary_associated_token: Pubkey,
    pub treasury_address: Pubkey,
    pub treasury_estimated_depletion_utc: u64,
    pub allocation_reserved: f64,
    pub allocation_left: f64,
    pub escrow_vested_amount_snap: f64,
    pub escrow_vested_amount_snap_slot: u64,
    pub escrow_vested_amount_snap_block_time: u64,
    pub stream_resumed_slot: u64,
    pub stream_resumed_block_time: u64,
    pub auto_pause_in_seconds: u64,
    pub allocation_assigned: f64
}
impl Pack for StreamV1 {
    const LEN: usize = 500;
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {

        let input = array_ref![input, 0, StreamV1::LEN];
        let (
            initialized,
            stream_name,
            treasurer_address,
            rate_amount,
            rate_interval_in_seconds,
            funded_on_utc,
            start_utc,
            rate_cliff_in_seconds,
            cliff_vest_amount,
            cliff_vest_percent,
            beneficiary_address,
            beneficiary_associated_token,
            treasury_address,
            treasury_estimated_depletion_utc,
            allocation_reserved,
            allocation_left,
            escrow_vested_amount_snap,
            escrow_vested_amount_snap_slot,
            escrow_vested_amount_snap_block_time,
            stream_resumed_slot,
            stream_resumed_block_time,
            auto_pause_in_seconds,
            allocation_assigned,
            _additional_data
            
        ) = array_refs![input, 1, 32, 32, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 203];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        Ok(StreamV1 {
            initialized, 
            stream_name: String::from_utf8_lossy(stream_name).to_string(),
            treasurer_address: Pubkey::new_from_array(*treasurer_address),                   
            rate_amount: f64::from_le_bytes(*rate_amount),
            rate_interval_in_seconds: u64::from_le_bytes(*rate_interval_in_seconds),
            funded_on_utc: u64::from_le_bytes(*funded_on_utc),
            start_utc: u64::from_le_bytes(*start_utc),
            rate_cliff_in_seconds: u64::from_le_bytes(*rate_cliff_in_seconds),
            cliff_vest_amount: f64::from_le_bytes(*cliff_vest_amount),
            cliff_vest_percent: f64::from_le_bytes(*cliff_vest_percent),
            beneficiary_address: Pubkey::new_from_array(*beneficiary_address),
            beneficiary_associated_token: Pubkey::new_from_array(*beneficiary_associated_token),
            treasury_address: Pubkey::new_from_array(*treasury_address), 
            treasury_estimated_depletion_utc: u64::from_le_bytes(*treasury_estimated_depletion_utc),
            allocation_reserved: f64::from_le_bytes(*allocation_reserved),
            allocation_left: f64::from_le_bytes(*allocation_left),
            escrow_vested_amount_snap: f64::from_le_bytes(*escrow_vested_amount_snap),
            escrow_vested_amount_snap_slot: u64::from_le_bytes(*escrow_vested_amount_snap_slot),
            escrow_vested_amount_snap_block_time: u64::from_le_bytes(*escrow_vested_amount_snap_block_time),
            stream_resumed_slot: u64::from_le_bytes(*stream_resumed_slot),
            stream_resumed_block_time: u64::from_le_bytes(*stream_resumed_block_time),
            auto_pause_in_seconds: u64::from_le_bytes(*auto_pause_in_seconds),
            allocation_assigned: f64::from_le_bytes(*allocation_assigned),
        })
    }
}

/// Treasury V0
#[derive(Clone, Debug)]
pub struct Treasury {
    pub initialized: bool,
    pub treasury_block_height: u64,
    pub treasury_mint_address: Pubkey,
    pub treasury_base_address: Pubkey
}
impl Pack for Treasury {
    const LEN: usize = 73;
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, Treasury::LEN];
        let (
            initialized,
            treasury_block_height,
            treasury_mint_address,
            treasury_base_address

        ) = array_refs![input, 1, 8, 32, 32];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(TreasuryError::InvalidTreasuryData.into()),
        };

        Ok(Treasury {
            initialized,             
            treasury_block_height: u64::from_le_bytes(*treasury_block_height),
            treasury_mint_address: Pubkey::new_from_array(*treasury_mint_address),
            treasury_base_address: Pubkey::new_from_array(*treasury_base_address)
        })
    }
}

/// Treasury V1
#[derive(Clone, Debug)]
pub struct TreasuryV1 {
    pub initialized: bool,
    pub slot: u64,
    pub treasurer_address: Pubkey,
    pub associated_token_address: Pubkey,
    pub mint_address: Pubkey,
    pub label: String,
    pub balance: f64,
    pub allocation_reserved: f64,
    pub allocation_left: f64,
    pub streams_amount: u64,
    pub created_on_utc: u64,
    pub depletion_rate: f64,
    pub treasury_type: u8,
    pub auto_close: bool,
    pub allocation_assigned: f64
}

impl Pack for TreasuryV1 {
    const LEN: usize = 300;    
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {

        let input = array_ref![input, 0, TreasuryV1::LEN];
        let (
            initialized,
            slot,
            treasurer_address,
            associated_token_address,
            mint_address,            
            label,
            balance,
            allocation_reserved,
            allocation_left,
            streams_amount,
            created_on_utc,
            depletion_rate,
            treasury_type,
            auto_close,
            allocation_assigned,
            _additional_data

        ) = array_refs![input, 1, 8, 32, 32, 32, 32, 8, 8, 8, 8, 8, 8, 1, 1, 8, 105];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(TreasuryError::InvalidTreasuryData.into()),
        };

        let auto_close = match auto_close {
            [0] => false,
            [1] => true,
            _ => return Err(TreasuryError::InvalidTreasuryData.into()),
        };

        Ok(TreasuryV1 {
            initialized,             
            slot: u64::from_le_bytes(*slot),
            treasurer_address: Pubkey::new_from_array(*treasurer_address),
            associated_token_address: Pubkey::new_from_array(*associated_token_address),
            mint_address: Pubkey::new_from_array(*mint_address),
            label: String::from_utf8_lossy(label).to_string(),
            balance: f64::from_le_bytes(*balance),
            allocation_reserved: f64::from_le_bytes(*allocation_reserved),
            allocation_left: f64::from_le_bytes(*allocation_left),
            streams_amount: u64::from_le_bytes(*streams_amount),
            created_on_utc: u64::from_le_bytes(*created_on_utc),
            depletion_rate: f64::from_le_bytes(*depletion_rate),
            treasury_type: u8::from_le_bytes(*treasury_type),
            auto_close,
            allocation_assigned: f64::from_le_bytes(*allocation_assigned)
        })
    }
}
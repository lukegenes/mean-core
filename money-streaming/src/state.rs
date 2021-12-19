// Program objects, (de)serializing state

use std::{ string::String };

use solana_program::{
    pubkey::Pubkey,
    program_error::ProgramError,    
    program_pack::{ IsInitialized, Pack, Sealed }
};

use arrayref::{
    array_mut_ref,
    mut_array_refs,
    array_ref, 
    array_refs, 
};

use crate::error::{ StreamError, TreasuryError };

#[derive(PartialEq)]
pub enum StreamStatus 
{
    Scheduled = 0,
    Running = 1,
    Paused = 2
}

/// Stream

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

impl Sealed for Stream {}

impl IsInitialized for Stream {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for Stream {
    fn default() -> Self {
        Self {
            initialized: false,
            stream_name: String::default(),
            treasurer_address: Pubkey::default(),             
            rate_amount: 0.0,
            rate_interval_in_seconds: 0,
            funded_on_utc: 0,
            start_utc: 0,
            rate_cliff_in_seconds: 0,
            cliff_vest_amount: 0.0,
            cliff_vest_percent: 0.0,
            beneficiary_address: Pubkey::default(),
            beneficiary_associated_token: Pubkey::default(),
            treasury_address: Pubkey::default(), 
            treasury_estimated_depletion_utc: 0,
            total_deposits: 0.0,
            total_withdrawals: 0.0,
            escrow_vested_amount_snap: 0.0,
            escrow_vested_amount_snap_block_height: 0,
            escrow_vested_amount_snap_block_time: 0,
            stream_resumed_block_height: 0,
            stream_resumed_block_time: 0,
            auto_pause_in_seconds: 0
        }
    }
}

impl Pack for Stream {
    const LEN: usize = 289;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Stream::LEN];
        let (
            initialized_output,
            stream_name_output,
            treasurer_address_output,
            rate_amount_output,
            rate_interval_in_seconds_output,
            funded_on_utc_output,
            start_utc_output,
            rate_cliff_in_seconds_output,
            cliff_vest_amount_output,
            cliff_vest_percent_output,
            beneficiary_address_output,
            beneficiary_associated_token_output,
            treasury_address_output,
            treasury_estimated_depletion_utc_output,
            total_deposits_output,
            total_withdrawals_output,
            escrow_vested_amount_snap_output,
            escrow_vested_amount_snap_block_height_output,
            escrow_vested_amount_snap_block_time_output,
            stream_resumed_block_height_output,
            stream_resumed_block_time_output,
            auto_pause_in_seconds_output
            
        ) = mut_array_refs![output, 1, 32, 32, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8];

        let Stream {
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

        } = self;

        initialized_output[0] = *initialized as u8;
        stream_name_output.copy_from_slice(stream_name.as_ref());
        treasurer_address_output.copy_from_slice(treasurer_address.as_ref());
        *rate_amount_output = rate_amount.to_le_bytes();
        *rate_interval_in_seconds_output = rate_interval_in_seconds.to_le_bytes();
        *funded_on_utc_output = funded_on_utc.to_le_bytes();
        *start_utc_output = start_utc.to_le_bytes();
        *rate_cliff_in_seconds_output = rate_cliff_in_seconds.to_le_bytes();
        *cliff_vest_amount_output = cliff_vest_amount.to_le_bytes();
        *cliff_vest_percent_output = cliff_vest_percent.to_le_bytes();
        beneficiary_address_output.copy_from_slice(beneficiary_address.as_ref());
        beneficiary_associated_token_output.copy_from_slice(beneficiary_associated_token.as_ref());
        treasury_address_output.copy_from_slice(treasury_address.as_ref());
        *treasury_estimated_depletion_utc_output = treasury_estimated_depletion_utc.to_le_bytes();
        *total_deposits_output = total_deposits.to_le_bytes();
        *total_withdrawals_output = total_withdrawals.to_le_bytes();
        *escrow_vested_amount_snap_output = escrow_vested_amount_snap.to_le_bytes();
        *escrow_vested_amount_snap_block_height_output = escrow_vested_amount_snap_block_height.to_le_bytes();
        *escrow_vested_amount_snap_block_time_output = escrow_vested_amount_snap_block_time.to_le_bytes();
        *stream_resumed_block_height_output = stream_resumed_block_height.to_le_bytes();
        *stream_resumed_block_time_output = stream_resumed_block_time.to_le_bytes();
        *auto_pause_in_seconds_output = auto_pause_in_seconds.to_le_bytes();
    }
    
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

impl Sealed for StreamV1 {}

impl IsInitialized for StreamV1 {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
} 

impl Default for StreamV1 {
    fn default() -> Self {
        Self {
            initialized: false,
            stream_name: String::default(),
            treasurer_address: Pubkey::default(),             
            rate_amount: 0.0,
            rate_interval_in_seconds: 0,
            funded_on_utc: 0,
            start_utc: 0,
            rate_cliff_in_seconds: 0,
            cliff_vest_amount: 0.0,
            cliff_vest_percent: 0.0,
            beneficiary_address: Pubkey::default(),
            beneficiary_associated_token: Pubkey::default(),
            treasury_address: Pubkey::default(), 
            treasury_estimated_depletion_utc: 0,
            allocation_reserved: 0.0,
            allocation_left: 0.0,
            escrow_vested_amount_snap: 0.0,
            escrow_vested_amount_snap_slot: 0,
            escrow_vested_amount_snap_block_time: 0,
            stream_resumed_slot: 0,
            stream_resumed_block_time: 0,
            auto_pause_in_seconds: 0,
            allocation_assigned: 0.0
        }
    }
}

impl Pack for StreamV1 {
    const LEN: usize = 500;

    fn pack_into_slice(&self, output: &mut [u8]) {

        let output = array_mut_ref![output, 0, StreamV1::LEN];
        let (
            initialized_output,
            stream_name_output,
            treasurer_address_output,
            rate_amount_output,
            rate_interval_in_seconds_output,
            funded_on_utc_output,
            start_utc_output,
            rate_cliff_in_seconds_output,
            cliff_vest_amount_output,
            cliff_vest_percent_output,
            beneficiary_address_output,
            beneficiary_associated_token_output,
            treasury_address_output,
            treasury_estimated_depletion_utc_output,
            allocation_reserved_output,
            allocation_left_output,
            escrow_vested_amount_snap_output,
            escrow_vested_amount_snap_slot_output,
            escrow_vested_amount_snap_block_time_output,
            stream_resumed_slot_output,
            stream_resumed_block_time_output,
            auto_pause_in_seconds_output,
            allocation_assigned_output,
            _additional_data
            
        ) = mut_array_refs![output, 1, 32, 32, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 203];

        let StreamV1 {
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
            allocation_assigned

        } = self;

        initialized_output[0] = *initialized as u8;
        stream_name_output.copy_from_slice(stream_name.as_ref());
        treasurer_address_output.copy_from_slice(treasurer_address.as_ref());
        *rate_amount_output = rate_amount.to_le_bytes();
        *rate_interval_in_seconds_output = rate_interval_in_seconds.to_le_bytes();
        *funded_on_utc_output = funded_on_utc.to_le_bytes();
        *start_utc_output = start_utc.to_le_bytes();
        *rate_cliff_in_seconds_output = rate_cliff_in_seconds.to_le_bytes();
        *cliff_vest_amount_output = cliff_vest_amount.to_le_bytes();
        *cliff_vest_percent_output = cliff_vest_percent.to_le_bytes();
        beneficiary_address_output.copy_from_slice(beneficiary_address.as_ref());
        beneficiary_associated_token_output.copy_from_slice(beneficiary_associated_token.as_ref());
        treasury_address_output.copy_from_slice(treasury_address.as_ref());
        *treasury_estimated_depletion_utc_output = treasury_estimated_depletion_utc.to_le_bytes();
        *allocation_reserved_output = allocation_reserved.to_le_bytes();
        *allocation_left_output = allocation_left.to_le_bytes();
        *escrow_vested_amount_snap_output = escrow_vested_amount_snap.to_le_bytes();
        *escrow_vested_amount_snap_slot_output = escrow_vested_amount_snap_slot.to_le_bytes();
        *escrow_vested_amount_snap_block_time_output = escrow_vested_amount_snap_block_time.to_le_bytes();
        *stream_resumed_slot_output = stream_resumed_slot.to_le_bytes();
        *stream_resumed_block_time_output = stream_resumed_block_time.to_le_bytes();
        *auto_pause_in_seconds_output = auto_pause_in_seconds.to_le_bytes();
        *allocation_assigned_output = allocation_assigned.to_le_bytes();
    }
    
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

/// Treasury

#[derive(Clone, Debug)]
pub struct Treasury {
    pub initialized: bool,
    pub treasury_block_height: u64,
    pub treasury_mint_address: Pubkey,
    pub treasury_base_address: Pubkey
}

impl Sealed for Treasury {}

impl IsInitialized for Treasury {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self {
            initialized: false,
            treasury_block_height: 0,
            treasury_mint_address: Pubkey::default(),
            treasury_base_address: Pubkey::default()
        }
    }
}

impl Pack for Treasury {
    const LEN: usize = 73;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, Treasury::LEN];
        let (
            initialized_output,
            treasury_block_height_output,
            treasury_mint_address_output,
            treasury_base_address_output
            
        ) = mut_array_refs![output, 1, 8, 32, 32];

        let Treasury {
            initialized,
            treasury_block_height,
            treasury_mint_address,
            treasury_base_address

        } = self;

        initialized_output[0] = *initialized as u8;
        *treasury_block_height_output = treasury_block_height.to_le_bytes();
        treasury_mint_address_output.copy_from_slice(treasury_mint_address.as_ref());
        treasury_base_address_output.copy_from_slice(treasury_base_address.as_ref());
    }
    
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

impl Sealed for TreasuryV1 {}

impl IsInitialized for TreasuryV1 {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for TreasuryV1 {
    fn default() -> Self {
        Self {
            initialized: false,
            slot: 0,
            treasurer_address: Pubkey::default(),
            associated_token_address: Pubkey::default(),
            mint_address: Pubkey::default(),
            label: String::default(),
            balance: 0.0,
            allocation_reserved: 0.0,
            allocation_left: 0.0,
            created_on_utc: 0,
            streams_amount: 0,    
            depletion_rate: 0.0,
            treasury_type: 0,
            auto_close: false,
            allocation_assigned: 0.0
        }
    }
}

impl Pack for TreasuryV1 {
    const LEN: usize = 300;

    fn pack_into_slice(&self, output: &mut [u8]) {

        let output = array_mut_ref![output, 0, TreasuryV1::LEN];
        let (
            initialized_output,
            slot_output,            
            treasurer_address_output,
            associated_token_address_output,
            mint_address_output,
            label_output,
            balance_output,
            allocation_reserved_output,
            allocation_left_output,
            streams_amount_output,
            created_on_utc_output,
            depletion_rate_output,
            treasury_type_output,
            auto_close_output,
            allocation_assigned_output,
            _additional_data
            
        ) = mut_array_refs![output, 1, 8, 32, 32, 32, 32, 8, 8, 8, 8, 8, 8, 1, 1, 8, 105];

        let TreasuryV1 {
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
            allocation_assigned

        } = self;

        initialized_output[0] = *initialized as u8;
        *slot_output = slot.to_le_bytes();
        treasurer_address_output.copy_from_slice(treasurer_address.as_ref());
        associated_token_address_output.copy_from_slice(associated_token_address.as_ref());
        mint_address_output.copy_from_slice(mint_address.as_ref());   
        label_output.copy_from_slice(label.as_ref());
        *balance_output = balance.to_le_bytes();
        *allocation_reserved_output = allocation_reserved.to_le_bytes();
        *allocation_left_output = allocation_left.to_le_bytes();
        *streams_amount_output = streams_amount.to_le_bytes();
        *created_on_utc_output = created_on_utc.to_le_bytes();
        *depletion_rate_output = depletion_rate.to_le_bytes();
        *treasury_type_output = treasury_type.to_le_bytes();
        auto_close_output[0] = *auto_close as u8;
        *allocation_assigned_output = allocation_assigned.to_le_bytes();
    }
    
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
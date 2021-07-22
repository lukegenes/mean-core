// Program objects, (de)serializing state

use std::string::String;

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

pub const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";
// pub const MSP_ACCOUNT_ADDRESS: &str = "So11111111111111111111111111111111111111112";
// pub const MSP_ACCOUNT_KEY: &Pubkey = &Pubkey::new(String::from("So11111111111111111111111111111111111111112").as_ref());
pub const LAMPORTS_PER_SOL: u64 = 1000000000;
pub const TREASURY_MINT_DECIMALS: u8 = 6;

#[derive(Clone, Debug)]
pub struct StreamTerms {
    pub initialized: bool,
    pub proposed_by: Pubkey,
    pub stream_id: Pubkey,
    pub stream_name: String,
    pub treasurer_address: Pubkey,
    pub beneficiary_address: Pubkey,
    pub associated_token_address: Pubkey,
    pub rate_amount: f64,
    pub rate_interval_in_seconds: u64,
    pub rate_cliff_in_seconds: u64,
    pub cliff_vest_amount: f64,
    pub cliff_vest_percent: f64,
    pub auto_pause_in_seconds: u64
}

impl Sealed for StreamTerms {}

impl IsInitialized for StreamTerms {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for StreamTerms {
    fn default() -> Self {
        Self {
            initialized: false,
            proposed_by: Pubkey::default(),
            stream_id: Pubkey::default(),
            stream_name: String::default(),
            treasurer_address: Pubkey::default(),
            beneficiary_address: Pubkey::default(),
            associated_token_address: Pubkey::default(),
            rate_amount: 0.0,
            rate_interval_in_seconds: 0,
            rate_cliff_in_seconds: 0,
            cliff_vest_amount: 0.0,
            cliff_vest_percent: 100.0,
            auto_pause_in_seconds: 0
        }
    }
}

impl Pack for StreamTerms {
    const LEN: usize = 241;

    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, StreamTerms::LEN];
        let (
            initialized_output,
            proposed_by_output,
            stream_id_output,
            stream_name_output,
            treasurer_address_output,
            beneficiary_address_output,
            associated_token_address_output,
            rate_amount_output,
            rate_interval_in_seconds_output,
            rate_cliff_in_seconds_output,
            cliff_vest_amount_output,
            cliff_vest_percent_output,
            auto_pause_in_seconds_output
            
        ) = mut_array_refs![output, 1, 32, 32, 32, 32, 32, 32, 8, 8, 8, 8, 8, 8];

        let StreamTerms {
            initialized,
            proposed_by,
            stream_id,
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

        } = self;

        initialized_output[0] = *initialized as u8;
        proposed_by_output.copy_from_slice(proposed_by.as_ref());
        stream_id_output.copy_from_slice(stream_id.as_ref());
        stream_name_output.copy_from_slice(stream_name.as_ref());
        treasurer_address_output.copy_from_slice(treasurer_address.as_ref());
        beneficiary_address_output.copy_from_slice(beneficiary_address.as_ref());
        associated_token_address_output.copy_from_slice(associated_token_address.as_ref());
        *rate_amount_output = rate_amount.to_le_bytes();
        *rate_interval_in_seconds_output = rate_interval_in_seconds.to_le_bytes();
        *rate_cliff_in_seconds_output = rate_cliff_in_seconds.to_le_bytes();
        *cliff_vest_amount_output = cliff_vest_amount.to_le_bytes();
        *cliff_vest_percent_output = cliff_vest_percent.to_le_bytes();
        *auto_pause_in_seconds_output = auto_pause_in_seconds.to_le_bytes();
    }
    
    fn unpack_from_slice(input: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![input, 0, StreamTerms::LEN];
        let (
            initialized,
            proposed_by,
            stream_id,
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
            
        ) = array_refs![input, 1, 32, 32, 32, 32, 32, 32, 8, 8, 8, 8, 8, 8];

        let initialized = match initialized {
            [0] => false,
            [1] => true,
            _ => return Err(StreamError::InvalidStreamData.into()),
        };

        Ok(StreamTerms {
            initialized, 
            proposed_by: Pubkey::new_from_array(*proposed_by),
            stream_id: Pubkey::new_from_array(*stream_id),
            stream_name: String::from_utf8_lossy(stream_name).to_string(),
            treasurer_address: Pubkey::new_from_array(*treasurer_address),
            beneficiary_address: Pubkey::new_from_array(*beneficiary_address),
            associated_token_address: Pubkey::new_from_array(*associated_token_address),
            rate_amount: f64::from_le_bytes(*rate_amount),
            rate_interval_in_seconds: u64::from_le_bytes(*rate_interval_in_seconds),
            rate_cliff_in_seconds: u64::from_le_bytes(*rate_cliff_in_seconds),
            cliff_vest_amount: f64::from_le_bytes(*cliff_vest_amount),
            cliff_vest_percent: f64::from_le_bytes(*cliff_vest_percent),
            auto_pause_in_seconds: u64::from_le_bytes(*auto_pause_in_seconds)
        })
    }
}

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
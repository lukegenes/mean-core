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
use crate::error::{ StreamError };

#[derive(Clone, Debug)]
pub struct StreamV2 {
    pub version: u8,
    pub initialized: bool,
    pub name: String,
    pub treasurer_address: Pubkey,
    pub rate_amount_units: u64,
    pub rate_interval_in_seconds: u64,
    pub start_utc: u64,
    pub rate_cliff_in_seconds: u64,
    pub cliff_vest_amount_units: u64,
    pub cliff_vest_percent: f64,
    pub beneficiary_address: Pubkey,
    pub beneficiary_associated_token: Pubkey,
    pub treasury_address: Pubkey,
    pub allocation_assigned_units: u64,
    pub allocation_reserved_units: u64,

    pub total_withdrawals_units: u64,
    pub last_withdrawal_units: u64,
    pub last_withdrawal_slot: u64,
    pub last_withdrawal_block_time: u64,
    pub last_stop_withdrawable_units_snap: u64, 
    pub last_stop_slot: u64,
    pub last_stop_block_time: u64,
    pub last_resume_slot: u64,
    pub last_resume_block_time: u64
}

impl StreamV2 {
    fn get_est_depletion_time() -> Result<u64, StreamError>{
        //Est. Depletion = GetEstDepletion() {return start_utc + allocation_assigned/rate}
        Ok(0)
    }
    fn get_funds_sent_to_beneficiary() -> Result<u64, StreamError>{
        //Funds sent to recipient = total_withdrawals + GetWithdrawableAmount()
        Ok(0)
    }
    fn get_funds_left_in_account() -> Result<u64, StreamError>{
        //Funds left in account = allocation_assigned - total_withdrawals - GetWithdrawableAmount()
        Ok(0)
    }
    fn get_withdrawable_amount() -> Result<u64, StreamError>{
        //we did this one already
        Ok(0)
    }
    fn get_beneficiary_remaining_allocation() -> Result<u64, StreamError>{
        let remaining_allocation = allocation_assigned.checked_sub(total_withdrawals).ok_or(StreamError::Overflow)?;
        Ok(remaining_allocation)
    }
}

impl Default for StreamV2 {
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

impl Pack for StreamV2 {
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
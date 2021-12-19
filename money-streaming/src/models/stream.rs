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
    pub cliff_vest_amount_units: u64,
    pub cliff_vest_percent: f64,    
    pub beneficiary_address: Pubkey,
    pub beneficiary_associated_token: Pubkey,
    pub treasury_address: Pubkey,    
    pub allocation_assigned_units: u64,
    pub allocation_reserved_units: u64,
    //withdrawal tracking
    pub total_withdrawals_units: u64,
    pub last_withdrawal_units: u64,
    pub last_withdrawal_slot: u64,
    pub last_withdrawal_block_time: u64,    
    //how can a stream STOP? -> There are 2 ways: 
    //1) by a Manual Action (recordable when it happens) or 
    //2) by Running Out Of Funds (not recordable when it happens, needs to be calculated)
    pub last_manual_stop_withdrawable_units_snap: u64, 
    pub last_manual_stop_slot: u64,
    pub last_manual_stop_block_time: u64,
    //how can a RESUME take place? -> ONLY by a Manual Action
    pub last_manual_resume_allocation_change_units: u64,
    pub last_manual_resume_slot: u64,
    pub last_manual_resume_block_time: u64,
    //the total seconds that have been paused since the start_utc 
    //set when resume is called manually
    pub last_known_total_seconds_in_paused_status: u64 
}

impl StreamV2 {
    fn primitive_get_cliff_units<'info> () -> Result<StreamStatus, StreamError> {
        if self.cliff_vest_amount_units > 0 {
            return Ok(self.cliff_vest_amount_units);
        }
        if self.cliff_vest_percent > 0 {
            let cliff_units = cliff_vest_percent * allocation_assigned / 100f64;
            return Ok(cliff_units);
        }
        Ok(0)
    }

    fn primitive_is_manually_paused<'info>() -> bool {
        if self.last_manual_stop_block_time <= 0 {
            return false;
        }
        return self.last_manual_stop_block_time > self.last_manual_resume_block_time;
    }

    fn primitive_get_streamed_units_per_second<'info>() -> u64 {
        if self.rate_interval_in_seconds <= 0 {
            return 0;
        }
        let streamed_units_per_second = self.rate_amount_units / (stream.rate_interval_in_seconds as f64);
        return streamed_units_per_second;
    }

    fn get_status<'info>(clock: &Clock) -> Result<StreamStatus, StreamError> {
        let now = clock.unix_timestamp as u64 * 1000u64;
        
        //scheduled
        if self.start_utc > now {
            return Ok(StreamStatus::Scheduled);
        }
        
        //manually paused
        let is_manual_pause = primitive_is_manually_paused();
        if is_manual_pause {
            return Ok(StreamStatus::Paused);
        }
        
        //running or automatically paused (ran out of funds)
        let streamed_units_per_second = primitive_get_streamed_units_per_second();
        let cliff_units = primitive_get_cliff_units(now);
        let seconds_since_start = now - self.start_utc;
        let non_stop_earning_units = cliff_units + (streamed_units_per_second * seconds_since_start);
        let missed_earning_units_while_paused = streamed_units_per_second * self.last_known_total_seconds_in_paused_status;
        let entitled_earnings = non_stop_earning_units - missed_earning_units_while_paused;

        //running
        if self.allocation_assigned > entitled_earnings {
            return Ok(StreamStatus::Running);
        }
        
        //automatically paused (ran out of funds)
        return Ok(StreamStatus::Paused);
    }

    fn get_est_depletion_time(clock: &Clock) -> Result<u64, StreamError>{
        if rate_amount_per_second <= 0 {
            return Ok(clock.unix_timestamp as u64);
        }
        let cliff_units = primitive_get_cliff_units(now);
        let streamable_units = allocation_assigned - cliff_units;
        let duration_span_seconds = (streamable_units / rate_amount_per_second) + self.last_known_total_seconds_in_paused_status;
        let est_depletion_time = start_utc.checked_add(duration_span_seconds).ok_or(StreamError::Overflow)?;
        Ok(est_depletion_time)
    }

    fn get_funds_sent_to_beneficiary() -> Result<u64, StreamError>{
        let withdrawable = get_beneficiary_withdrawable_amount();
        let funds_sent = total_withdrawals.checked_add(withdrawable).ok_or(StreamError::Overflow)?;
        Ok(funds_sent)
    }

    fn get_funds_left_in_account() -> Result<u64, StreamError>{
        let withdrawable = get_beneficiary_withdrawable_amount();
        let funds_left_in_account = allocation_assigned
            .checked_sub(total_withdrawals).unwrap()
            .checked_sub(withdrawable).ok_or(StreamError::Overflow)
        Ok(funds_left_in_account)
    }

    fn get_beneficiary_remaining_allocation() -> Result<u64, StreamError>{
        let remaining_allocation = allocation_assigned.checked_sub(total_withdrawals).ok_or(StreamError::Overflow)?;
        Ok(remaining_allocation)?
    }

    fn get_beneficiary_withdrawable_amount<'info>(clock: &Clock) -> Result<u64, StreamError> {

        let unused_allocation = self.allocation_assigned - self.withdrawn_amount;
        if unused_allocation <= 0 {
            return Ok(0);
        }

        let status = get_status(clock)?;

        //Check if SCHEDULED
        if status == StreamStatus::Scheduled{
            return Ok(0);
        }
    
        //Check if PAUSED
        if status == StreamStatus::Paused {
            let is_manual_pause = primitive_is_manually_paused();
            let withdrawable_while_paused = match is_manual_pause {
                true => self.last_manual_stop_withdrawable_units_snap,
                _ => self.allocation_assigned - self.withdrawn_amount
            };
            return Ok(cmp::max(withdrawable_while_paused, 0));
        }
    
        //Check if RUNNING
        if self.rate_interval_in_seconds <= 0 || stream.rate_amount <= 0.0 {
            return Err(StreamError::InvalidArgument.into());
        }
        let streamed_units_per_second = primitive_get_streamed_units_per_second();
        let cliff_units = primitive_get_cliff_units();
        let now = clock.unix_timestamp as u64 * 1000u64;
        let seconds_since_start = now - self.start_utc;

        let non_stop_earning_units = cliff_units + (streamed_units_per_second * seconds_since_start);
        let missed_units_while_paused = streamed_units_per_second * self.last_known_total_seconds_in_paused_status;
        let entitled_earnings_units = non_stop_earning_units - missed_units_while_paused;

        let withdrawable_units_while_running = entitled_earnings_units - self.withdrawn_amount;
       
        let withdrawable = cmp::min(unused_allocation, withdrawable_units_while_running);
        return Ok(withdrawable);
    }
}

impl Default for StreamV2 {
    fn default() -> Self {
        Self {
            version: 2,
            initialized: false,
            name: String::default(),
            treasurer_address: Pubkey::default(),             
            rate_amount_units: 0,
            rate_interval_in_seconds: 0,
            start_utc: 0,
            cliff_vest_amount_units: 0,
            cliff_vest_percent: 0.0,
            beneficiary_address: Pubkey::default(),
            beneficiary_associated_token: Pubkey::default(),
            treasury_address: Pubkey::default(), 
            allocation_assigned_units: 0,
            allocation_reserved_units: 0,
            total_withdrawals_units: 0,
            last_withdrawal_units: 0,
            last_withdrawal_slot: 0,
            last_withdrawal_block_time: 0,
            last_manual_stop_withdrawable_units_snap: 0,
            last_manual_stop_slot: 0,
            last_manual_stop_block_time: 0,
            last_manual_resume_allocation_change_units: 0,
            last_manual_resume_slot: 0,
            last_manual_resume_block_time: 0,
            last_known_total_seconds_in_paused_status: 0
        }
    }
}

impl Pack for StreamV2 {
    const LEN: usize = 500;

    fn pack_into_slice(&self, output: &mut [u8]) {

        let output = array_mut_ref![output, 0, StreamV1::LEN];
        let (version_out, initialized_out, name_out, 
            treasurer_address_out,rate_amount_units_out, rate_interval_in_seconds_out, 
            start_utc_out, cliff_vest_amount_out, cliff_vest_percent_out, 
            beneficiary_address_out, beneficiary_associated_token_out, treasury_address_out, 
            allocation_assigned_units_out, allocation_reserved_units_out, total_withdrawals_units_out,
            last_withdrawal_units_out, last_withdrawal_slot_out, 
            last_withdrawal_block_time_out, last_manual_stop_withdrawable_units_snap_out, 
            last_manual_stop_slot_out, last_manual_stop_block_time_out,
            last_manual_resume_allocation_change_units_out, last_manual_resume_slot_out,
            last_manual_resume_block_time_out, last_known_total_seconds_in_paused_status_out
        ) = mut_array_refs![output, 1, 32, 32, 8, 8, 8, 8, 8, 8, 8, 32, 32, 32, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 203];

        let StreamV2 {
            version, initialized, name, 
            treasurer_address, rate_amount, rate_interval_in_seconds, 
            start_utc, cliff_vest_amount, cliff_vest_percent,
            beneficiary_address, beneficiary_associated_token, treasury_address,
            allocation_assigned_units, allocation_reserved_units, total_withdrawals,
            last_withdrawal_units, last_withdrawal_slot, 
            last_withdrawal_block_time, last_manual_stop_withdrawable_units_snap, 
            last_manual_stop_slot, last_manual_stop_block_time,
            last_manual_resume_allocation_change_units, last_manual_resume_slot,
            last_manual_resume_block_time, last_known_total_seconds_in_paused_status
        } = self;

        initialized_out[0] = *initialized as u8;
        name_out.copy_from_slice(name.as_ref());
        treasurer_address_out.copy_from_slice(treasurer_address.as_ref());
        *rate_amount_units_out = rate_amount_units.to_le_bytes();
        *rate_interval_in_seconds_out = rate_interval_in_seconds.to_le_bytes();
        *funded_on_utc_out = funded_on_utc.to_le_bytes();
        *start_utc_out = start_utc.to_le_bytes();
        *rate_cliff_in_seconds_out = rate_cliff_in_seconds.to_le_bytes();
        *cliff_vest_amount_out = cliff_vest_amount.to_le_bytes();
        *cliff_vest_percent_out = cliff_vest_percent.to_le_bytes();
        beneficiary_address_out.copy_from_slice(beneficiary_address.as_ref());
        beneficiary_associated_token_out.copy_from_slice(beneficiary_associated_token.as_ref());
        treasury_address_out.copy_from_slice(treasury_address.as_ref());
        *treasury_estimated_depletion_utc_out = treasury_estimated_depletion_utc.to_le_bytes();
        *allocation_reserved_out = allocation_reserved.to_le_bytes();
        *allocation_left_out = allocation_left.to_le_bytes();
        *escrow_vested_amount_snap_out = escrow_vested_amount_snap.to_le_bytes();
        *escrow_vested_amount_snap_slot_out = escrow_vested_amount_snap_slot.to_le_bytes();
        *escrow_vested_amount_snap_block_time_out = escrow_vested_amount_snap_block_time.to_le_bytes();
        *stream_resumed_slot_out = stream_resumed_slot.to_le_bytes();
        *stream_resumed_block_time_out = stream_resumed_block_time.to_le_bytes();
        *auto_pause_in_seconds_out = auto_pause_in_seconds.to_le_bytes();
        *allocation_assigned_out = allocation_assigned.to_le_bytes();
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
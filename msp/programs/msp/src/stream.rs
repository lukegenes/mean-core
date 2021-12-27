use anchor_lang::prelude::*;
use std::cmp;
use crate::errors::*;
use crate::enums::*;

#[account(BorshSerialize, BorshDeserialize)]
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
    //increment when resume is called manually
    pub last_known_total_seconds_in_paused_status: u64 
}

impl StreamV2 {

    fn primitive_get_cliff_units<'info>(&self) -> u64 {
        let mut cliff_units = self.cliff_vest_amount_units;
        if self.cliff_vest_percent > 0f64 {
            cliff_units = (self.cliff_vest_percent * self.allocation_assigned_units as f64 / 100f64) as u64;
        }
        return cliff_units;
    }

    fn primitive_is_manually_paused<'info>(&self) -> bool {
        if self.last_manual_stop_block_time == 0 {
            return false;
        }
        return self.last_manual_stop_block_time > self.last_manual_resume_block_time;
    }

    fn primitive_get_streamed_units_per_second<'info>(&self) -> u64 {
        if self.rate_interval_in_seconds == 0 {
            return 0;
        }
        return self.rate_amount_units / self.rate_interval_in_seconds;
    }

    fn get_status<'info>(&self) -> Result<StreamStatus> {
        let now = Clock::get()?.unix_timestamp as u64 * 1000u64;
        //scheduled
        if self.start_utc > now {
            return Ok(StreamStatus::Scheduled);
        }        
        //manually paused
        let is_manual_pause = self.primitive_is_manually_paused();
        if is_manual_pause {
            return Ok(StreamStatus::Paused);
        }    
        //running or automatically paused (ran out of funds)
        let streamed_units_per_second = self.primitive_get_streamed_units_per_second();
        let cliff_units = self.primitive_get_cliff_units();
        let seconds_since_start = now.checked_sub(self.start_utc).ok_or(ErrorCode::Overflow)?;
        let non_stop_earning_units = cliff_units
            .checked_add(streamed_units_per_second * seconds_since_start)
            .ok_or(ErrorCode::Overflow)?;

        let missed_earning_units_while_paused = streamed_units_per_second
            .checked_mul(self.last_known_total_seconds_in_paused_status)
            .ok_or(ErrorCode::Overflow)?;

        let entitled_earnings = non_stop_earning_units
            .checked_sub(missed_earning_units_while_paused)
            .ok_or(ErrorCode::Overflow)?;

        //running
        if self.allocation_assigned_units > entitled_earnings {
            return Ok(StreamStatus::Running);
        }        
        //automatically paused (ran out of funds)
        Ok(StreamStatus::Paused)
    }

    fn get_est_depletion_time(&self) -> Result<u64> {        
        let clock = Clock::get()?;
        if self.rate_interval_in_seconds == 0 {
            return Ok(clock.unix_timestamp as u64 * 1000u64); // now
        }
        let cliff_units = self.primitive_get_cliff_units();
        let streamable_units = self.allocation_assigned_units.checked_sub(cliff_units).ok_or(ErrorCode::Overflow)?;
        let duration_span_seconds = (streamable_units / self.rate_interval_in_seconds)
            .checked_add(self.last_known_total_seconds_in_paused_status)
            .ok_or(ErrorCode::Overflow)?;

        let est_depletion_time = self.start_utc.checked_add(duration_span_seconds).ok_or(ErrorCode::Overflow)?;
        Ok(est_depletion_time)
    }

    fn get_funds_sent_to_beneficiary(&self) -> Result<u64> {
        let withdrawable = self.get_beneficiary_withdrawable_amount()?;
        let funds_sent = self.total_withdrawals_units
            .checked_add(withdrawable)
            .ok_or(ErrorCode::Overflow)?;            
        Ok(funds_sent)
    }

    fn get_funds_left_in_account(&self) -> Result<u64> {
        let withdrawable = self.get_beneficiary_withdrawable_amount()?;
        let funds_left_in_account = self.allocation_assigned_units
            .checked_sub(self.total_withdrawals_units).unwrap()
            .checked_sub(withdrawable).ok_or(ErrorCode::Overflow)?;
        Ok(funds_left_in_account)
    }

    fn get_beneficiary_remaining_allocation(&self) -> Result<u64> {
        let remaining_allocation = self.allocation_assigned_units
            .checked_sub(self.total_withdrawals_units)
            .ok_or(ErrorCode::Overflow)?;
        Ok(remaining_allocation)
    }

    fn get_beneficiary_withdrawable_amount<'info>(&self) -> Result<u64> {

        let clock = Clock::get()?;
        let unused_allocation = self.get_beneficiary_remaining_allocation()?;
        if unused_allocation == 0 {
            return Ok(0);
        }

        let status = self.get_status()?;
        //Check if SCHEDULED
        if status == StreamStatus::Scheduled{
            return Ok(0);
        }    
        //Check if PAUSED
        if status == StreamStatus::Paused {
            let is_manual_pause = self.primitive_is_manually_paused();
            let withdrawable_while_paused = match is_manual_pause {
                true => self.last_manual_stop_withdrawable_units_snap,
                _ => self.allocation_assigned_units
                        .checked_sub(self.total_withdrawals_units)
                        .ok_or(ErrorCode::Overflow)?
            };
            return Ok(withdrawable_while_paused);
        }    
        //Check if RUNNING
        if self.rate_interval_in_seconds == 0 || self.rate_amount_units == 0 {
            return Err(ErrorCode::InvalidArgument.into());
        }
        let streamed_units_per_second = self.primitive_get_streamed_units_per_second();
        let cliff_units = self.primitive_get_cliff_units();
        let now = clock.unix_timestamp as u64 * 1000u64;
        let seconds_since_start = now.checked_sub(self.start_utc).ok_or(ErrorCode::Overflow)?;
        let non_stop_earning_units = cliff_units
            .checked_add(streamed_units_per_second * seconds_since_start)
            .ok_or(ErrorCode::Overflow)?;

        let missed_units_while_paused = streamed_units_per_second
            .checked_mul(self.last_known_total_seconds_in_paused_status)
            .ok_or(ErrorCode::Overflow)?;

        let entitled_earnings_units = non_stop_earning_units
            .checked_sub(missed_units_while_paused)
            .ok_or(ErrorCode::Overflow)?;

        let withdrawable_units_while_running = entitled_earnings_units
            .checked_sub(self.total_withdrawals_units)
            .ok_or(ErrorCode::Overflow)?;
       
        let withdrawable = cmp::min(unused_allocation, withdrawable_units_while_running);
        Ok(withdrawable)
    }
    
}
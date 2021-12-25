use anchor_lang::prelude::*;

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
    //set when resume is called manually
    pub last_known_total_seconds_in_paused_status: u64 
}
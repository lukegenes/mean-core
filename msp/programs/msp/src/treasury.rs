use anchor_lang::prelude::*;

#[account(BorshSerialize, BorshDeserialize)]
pub struct TreasuryV2 {
    pub initialized: bool,
    pub version: u8,
    pub bump: u8,
    pub slot: u64,
    pub name: String,        
    pub treasurer_address: Pubkey,
    pub associated_token_address: Pubkey,
    pub mint_address: Pubkey,
    pub labels: Vec<String>,  //max 5 labels per treasury 
    pub last_known_balance_units: u64,
    pub last_known_balance_slot: u64,
    pub last_known_balance_block_time: u64,

    pub allocation_reserved_units: u64,
    pub allocation_assigned_units: u64,
    pub total_withdrawals_units: u64,

    pub total_streams: u64,
    pub created_on_utc: u64,
    pub depletion_units_per_second: f64,
    pub treasury_type: u8,
    pub auto_close: bool   
}

impl TreasuryV2 {

    fn get_est_depletion_time<'info>(&self) -> u64 {
        //Est. Depletion = GetEstDepletion() {
        //    var amountLeft = allocation_assigned_units - total_withdrawals_units;
        //    var secondsLeft = amountLeft / depletion_units_per_second
        //    return now_utc + secondsLeft;
        return 0;
    }
}
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
use crate::error::{ TreasuryError };

#[derive(Clone, Debug)]
pub struct TreasuryV2 {
    pub version: u8,
    pub initialized: bool,
    pub slot: u64,
    pub name: String,        
    pub treasurer_address: Pubkey,
    pub associated_token_address: Pubkey,
    pub mint_address: Pubkey,
    pub labels: [String; 5],  //max 5 labels per treasury 
    pub last_known_balance_units: u64,
    pub last_known_balance_slot: u64,
    pub last_known_balance_block_time: u64,

    pub allocation_reserved_units: u64,
    pub allocation_assigned_units: f64,
    pub total_withdrawals_units: u64,

    pub total_streams: u64,
    pub created_on_utc: u64,
    pub depletion_units_per_second: f64,
    pub treasury_type: TreasuryType,
    pub auto_close: bool   
}

impl TreasuryV2 {
    fn get_est_depletion_time() -> Result<u64, StreamError>{
        //Est. Depletion = GetEstDepletion() {
        //    var amountLeft = allocation_assigned_units - total_withdrawals_units;
        //    var secondsLeft = amountLeft / depletion_units_per_second
        //    return now_utc + secondsLeft;
        Ok(0)
    }
}

impl IsInitialized for TreasuryV2 {
    fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl Default for TreasuryV2 {
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

impl Pack for TreasuryV2 {
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
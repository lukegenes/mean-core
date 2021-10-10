use anchor_lang::prelude::*;
use crate::state::*;

// #[account]
// pub struct PoolInfo {
//     pub chain_id: u64,
//     pub account: Pubkey,
//     pub protocol_account: Pubkey,
//     pub amm_account: Pubkey,
//     pub tokens: Vec<Pubkey>
// }

pub struct SaberClient {
    pub protocol_account: Pubkey
}

impl<'info> LpClient<'info> for SaberClient {

    fn get_pool_info(self: &Self) -> ProgramResult{
        Ok(())
    }
}

impl<'info> Client<'info> for SaberClient {

    fn get_protocol_account(self: &Self) -> Pubkey{
        SABER.parse().unwrap()
    }

    fn get_exchange_info(
        self: &Self,
        amount: f64, 
        slippage: f64

    ) -> ProgramResult{
        Ok(())
    }

    fn execute_swap(
        self: &Self,
        amount_in: f64,
        amount_out: f64,
        slippage: f64,
        fee_amount: f64

    ) -> ProgramResult{
        Ok(())
    }

}



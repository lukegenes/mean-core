use anchor_lang::prelude::*;

pub mod errors;
pub mod utils;
pub mod state;
pub mod data;
pub mod saber;
pub mod orca;
pub mod raydium;

use crate::state::*;
// use crate::utils::*;

declare_id!("B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3");

#[program]
pub mod hla {
    use super::*;

    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        from_amount: u64,
        min_out_amount: u64,
        _slippage: u64

    ) -> ProgramResult {

        let rem_accs_iter = &mut ctx.remaining_accounts.iter();

        let _pool_account_info = next_account_info(rem_accs_iter)?;
        // let pool_account_address = &pool_account_info.key.to_string();
        let protocol_account_info = next_account_info(rem_accs_iter)?;
        let protocol_account_address = &protocol_account_info.key.to_string();

        // let _verified = veriry_pool(
        //     pool_account_address.as_str(),
        //     protocol_account_address.as_str()
        // );

        let swap_info = SwapInfo {
            accounts: ctx.accounts.clone(),
            remaining_accounts: ctx.remaining_accounts.to_vec(),
            from_amount,
            min_out_amount
        };

        match protocol_account_address.as_str() {

            ORCA => orca::swap(swap_info),
            SABER => saber::swap(swap_info),
            RAYDIUM => raydium::swap(swap_info),
            // MERCURIAL => mercurial::swap(swap_info),
            // SERUM => serum::swap(swap_info),
    
            _ => return Err(errors::ErrorCode::PoolNotFound.into())
        }
    }
}

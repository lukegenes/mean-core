use anchor_lang::prelude::*;

pub mod errors;
pub mod utils;
pub mod state;
pub mod data;
pub mod saber;
pub mod orca;

use crate::state::*;

declare_id!("B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3");

#[program]
pub mod hla {
    use super::*;

    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        from_amount: u64,
        min_out_amount: u64,
        _slippage: u8

    ) -> ProgramResult {

        let rem_accs_iter = &mut ctx.remaining_accounts.iter();
        let pool_account = next_account_info(rem_accs_iter)?;
        let pool_account_address = &pool_account.key.to_string();
        let pool_info = utils::get_pool(pool_account_address.as_str())?;

        if pool_info.account.ne(pool_account.key.to_string().as_str())
        {
            return Err(errors::ErrorCode::InvalidPool.into());
        }

        let protocol_account = next_account_info(rem_accs_iter)?;        

        if pool_info.protocol_account.ne(protocol_account.key.to_string().as_str())
        {
            return Err(errors::ErrorCode::InvalidProtocol.into());
        }

        let amm_account = next_account_info(rem_accs_iter)?;   

        if pool_info.amm_account.ne(amm_account.key.to_string().as_str())
        {
            return Err(errors::ErrorCode::InvalidAmm.into());
        }

        let swap_info = SwapInfo {
            accounts: ctx.accounts.clone(),
            remaining_accounts: ctx.remaining_accounts.to_vec(),
            from_amount,
            min_out_amount
        };

        let _result = match protocol_account.key.to_string().as_str() {

            SABER => saber::swap(swap_info),
            ORCA => orca::swap(swap_info),
    
            _ => return Err(errors::ErrorCode::PoolNotFound.into()),
        };

        Ok(())
    }
}

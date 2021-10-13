use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint};

pub mod errors;
pub mod utils;
pub mod state;
pub mod data;
pub mod saber;
pub mod orca;

use crate::state::*;

declare_id!("EPa4WdYPcGGdwEbq425DMZukU2wDUE1RWAGrPbRYSLRE");

#[program]
pub mod hla {
    use super::*;

    pub fn swap<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, Swap<'info>>,
        from_amount: u64,
        min_out_amount: u64,
        slippage: u8

    ) -> ProgramResult {

        let hla_ops_account_key: Pubkey = state::HLA_OPS.parse().unwrap();

        if hla_ops_account_key.ne(ctx.accounts.hla_ops_account.key)
        {
            return Err(errors::ErrorCode::InvalidOpsAccount.into());
        }

        let remaining_accounts = ctx.remaining_accounts.clone();
        let rem_accs_iter = &mut remaining_accounts.iter();
        let pool_account = next_account_info(rem_accs_iter)?;
        let pool_info = utils::get_pool(&pool_account.key)?;

        if pool_info.account.ne(pool_account.key)
        {
            return Err(errors::ErrorCode::InvalidPool.into());
        }

        let protocol_account = next_account_info(rem_accs_iter)?;        

        if pool_info.protocol_account.ne(protocol_account.key)
        {
            return Err(errors::ErrorCode::InvalidProtocol.into());
        }

        let amm_account = next_account_info(rem_accs_iter)?;    

        if pool_info.amm_account.ne(amm_account.key)
        {
            return Err(errors::ErrorCode::InvalidAmm.into());
        }

        let accounts = Swap {
            vault_account: ctx.accounts.vault_account.clone(),
            from_token_mint: ctx.accounts.from_token_mint.clone(),
            from_token_account: ctx.accounts.from_token_account.clone(),
            to_token_mint: ctx.accounts.to_token_mint.clone(),
            to_token_account: ctx.accounts.to_token_account.clone(),
            hla_ops_account: ctx.accounts.hla_ops_account.clone(),
            hla_ops_token_account: ctx.accounts.hla_ops_token_account.clone(),
            token_program_account: ctx.accounts.token_program_account.clone()
        };

        let swap_info = SwapInfo {
            accounts,
            remaining_accounts: ctx.remaining_accounts.to_vec(),
            from_amount,
            min_out_amount
        };

        match pool_account.key.to_string().as_str() {

            SABER => { saber::swap(swap_info) },
            ORCA => { orca::swap(swap_info) },
    
            _ => return Err(errors::ErrorCode::PoolNotFound.into()),
        }
    }
}

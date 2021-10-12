use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint};

pub mod errors;
pub mod utils;
pub mod state;
pub mod data;
pub mod saber;

use crate::saber::*;

declare_id!("5sEgjVKG4pNUrjU1EVmKRsEAmsB9f2ujJn2H1ZxX2UQs");

#[program]
pub mod hla {
    use super::*;

    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        from_amount: u64,
        min_out_amount: u64,
        slippage: u8

    ) -> ProgramResult {

        let hla_ops_account_key: Pubkey = state::HLA_OPS.parse().unwrap();

        if hla_ops_account_key.ne(ctx.accounts.hla_ops_account.key)
        {
            return Err(errors::ErrorCode::InvalidOpsAccount.into());
        }

        let rem_accs_iter = &mut ctx.remaining_accounts.clone().iter();
        let pool_account = next_account_info(rem_accs_iter)?.to_account_info();
        let pool_info = utils::get_pool(&pool_account.key)?;

        if pool_info.account.ne(pool_account.key)
        {
            return Err(errors::ErrorCode::InvalidPool.into());
        }

        let protocol_account = next_account_info(rem_accs_iter)?.to_account_info();        

        if pool_info.protocol_account.ne(protocol_account.key)
        {
            return Err(errors::ErrorCode::InvalidProtocol.into());
        }

        let amm_account = next_account_info(rem_accs_iter)?.to_account_info();    

        if pool_info.amm_account.ne(amm_account.key)
        {
            return Err(errors::ErrorCode::InvalidAmm.into());
        }

        let mut accounts = vec![
            ctx.accounts.vault_account.to_account_info(),
            ctx.accounts.from_token_account.to_account_info(),
            ctx.accounts.to_token_account.to_account_info()
        ];

        accounts.extend_from_slice(ctx.remaining_accounts);

        match pool_account.key.to_string().as_str() {

            SABER => {
                saber::swap(
                    accounts,
                    from_amount as f64,
                    min_out_amount as f64
                )
            },
    
            _ => return Err(errors::ErrorCode::PoolNotFound.into()),
        }
    }
}

#[derive(Accounts)]
#[
    instruction(
        from_amount: u64, 
        min_out_amount: u64, 
        slippage: u8
    )
]
pub struct Swap<'info> {
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    pub vault_account: AccountInfo<'info>,
    pub from_token_mint: CpiAccount<'info, Mint>,
    pub from_token_account: CpiAccount<'info, TokenAccount>,
    pub to_token_mint: CpiAccount<'info, Mint>,
    pub to_token_account: CpiAccount<'info, TokenAccount>,
    pub hla_ops_account: AccountInfo<'info>,
    pub hla_ops_token_account: Account<'info, TokenAccount>
}

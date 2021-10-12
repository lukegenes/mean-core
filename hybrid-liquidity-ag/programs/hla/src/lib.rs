use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint};
use stable_swap_anchor::{SwapInfo};

pub mod errors;
pub mod utils;
pub mod state;
pub mod data;
pub mod saber;

declare_id!("5sEgjVKG4pNUrjU1EVmKRsEAmsB9f2ujJn2H1ZxX2UQs");

#[program]
pub mod hla {
    use super::*;

    pub fn swap(
        ctx: Context<Swap>,
        from_amount: u64,
        min_out_amount: u64,
        slippage: u8

    ) -> ProgramResult {

        let pool_account_key = ctx.accounts.pool_account.key;
        let pool_info = utils::get_pool(&pool_account_key)?;

        if pool_info.protocol_account.ne(ctx.accounts.protocol_account.key)
        {
            return Err(errors::ErrorCode::InvalidProtocol.into());
        }

        if pool_info.amm_account.ne(ctx.accounts.amm_account.key)
        {
            return Err(errors::ErrorCode::InvalidAmm.into());
        }

        let protocol_key = pool_account_key.clone();

        match protocol_key.to_string().as_str() {

            SABER => {

                let data = ctx
                    .accounts
                    .pool_account
                    .deserialize_data::<u8>()
                    .unwrap()
                    .clone();

                let mut data_vec = &vec![data]; 
                let swap_info = SwapInfo::try_deserialize(&mut data_vec.as_slice())?;

                saber::swap(
                    &swap_info,
                    from_amount as f64,
                    min_out_amount as f64,
                    slippage
                );
            },
    
            _ => { },
        }

        Ok(())
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
    pub pool_account: AccountInfo<'info>,
    pub protocol_account: AccountInfo<'info>,    
    pub amm_account: AccountInfo<'info>,
    pub vault_account: AccountInfo<'info>,
    pub from_token_mint: CpiAccount<'info, Mint>,
    pub from_token_account: CpiAccount<'info, TokenAccount>,
    pub to_token_mint: CpiAccount<'info, Mint>,
    pub to_token_account: CpiAccount<'info, TokenAccount>
}

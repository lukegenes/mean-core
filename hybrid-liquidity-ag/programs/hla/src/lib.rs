use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Mint};

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
        amount_in: u64,
        slippage: u8

    ) -> ProgramResult {
        msg!("{:?}", data::get_pools());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount_in: u64, slippage: u8)]
pub struct Swap<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub from_account: AccountInfo<'info>,
    pub from_token: AccountInfo<'info>,
    pub to_account: AccountInfo<'info>,
    pub to_token: AccountInfo<'info>
}

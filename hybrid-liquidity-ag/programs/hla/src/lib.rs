use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Mint};

mod errors;
mod utils;
mod state;
mod data;

declare_id!("5sEgjVKG4pNUrjU1EVmKRsEAmsB9f2ujJn2H1ZxX2UQs");

#[program]
pub mod hla {
    use super::*;

    pub fn swap(ctx: Context<Swap>) -> ProgramResult {
        msg!("{:?}", data::get_pools());
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(amount_in: u64, slippage: u64)]
pub struct Swap<'info> {
    #[account(signer)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub from_account: Account<'info, TokenAccount>,
    pub from_token: Account<'info, Mint>,
    #[account(mut)]
    pub to_account: Account<'info, TokenAccount>,
    pub to_token: Account<'info, Mint>
}

#[account]
pub struct SwapInfo {
    pub owner: Pubkey,
    pub from_account: Pubkey,
    pub from_token: Pubkey,
    pub to_account: Pubkey,
    pub to_token: Pubkey,
    pub amount_in: u64,
    pub slippage: u64
}

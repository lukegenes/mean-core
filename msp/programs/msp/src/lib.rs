use anchor_lang::prelude::*;

pub mod errors;
pub mod enums;
pub mod constants;
pub mod stream;
pub mod treasury;
pub mod instructions;

use crate::instructions::*;

pub mod fee_treasury {
    solana_program::declare_id!("3TD6SWY9M1mLY2kZWJNavPLhwXvcRsWdnZLRaMzERJBw");
}

#[program]
pub mod msp {

    declare_id!("H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko");

    use super::*;

    pub fn create_treasury(ctx: Context<CreateTreasury>) -> ProgramResult {
        Ok(())
    }

    pub fn create_stream(ctx: Context<CreateStream>) -> ProgramResult {
        Ok(())
    }

    pub fn add_funds(ctx: Context<AddFunds>) -> ProgramResult {
        Ok(())
    }
}

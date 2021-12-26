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

    // Create Treasury
    pub fn create_treasury(_ctx: Context<CreateTreasury>) -> ProgramResult {
        Ok(())
    }

    // Create Stream
    pub fn create_stream(_ctx: Context<CreateStream>) -> ProgramResult {
        Ok(())
    }

    // Add Funds
    pub fn add_funds(_ctx: Context<AddFunds>) -> ProgramResult {
        Ok(())
    }

    // Withdraw
    pub fn withdraw(_ctx: Context<Withdraw>) -> ProgramResult {
        Ok(())
    }

    // Pause Stream
    pub fn pause_stream(_ctx: Context<PauseOrResumeStream>) -> ProgramResult {
        Ok(())
    }

    // Resume Stream
    pub fn resume_stream(_ctx: Context<PauseOrResumeStream>) -> ProgramResult {
        Ok(())
    }

    // Close Stream
    pub fn close_stream(_ctx: Context<CloseStream>) -> ProgramResult {
        Ok(())
    }

    // Close Treasury
    pub fn close_treasury(_ctx: Context<CloseTreasury>) -> ProgramResult {
        Ok(())
    }

    // Refresh Treasury Balance
    pub fn refresh_treasury_balance(_ctx: Context<RefreshTreasuryBalance>) -> ProgramResult {
        Ok(())
    }
}

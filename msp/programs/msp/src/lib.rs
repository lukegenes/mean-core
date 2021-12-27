use anchor_lang::prelude::*;
use anchor_spl::token::*;

pub mod errors;
pub mod enums;
pub mod constants;
pub mod stream;
pub mod treasury;
pub mod instructions;

use crate::instructions::*;
use crate::constants::*;

pub mod fee_treasury {
    solana_program::declare_id!("3TD6SWY9M1mLY2kZWJNavPLhwXvcRsWdnZLRaMzERJBw");
}

#[program]
pub mod msp {

    declare_id!("H6wJxgkcc93yeUFnsZHgor3Q3pSWgGpEysfqKrwLtMko");

    use super::*;

    // Create Treasury
    pub fn create_treasury(
        ctx: Context<CreateTreasury>,
        slot: u64,
        bump: u8,
        name: String,
        treasury_type: u8,
        auto_close: bool,
        
    ) -> ProgramResult {
        
        // Initialize Treasury
        let treasury = &mut ctx.accounts.treasury;
        treasury.version = 2;
        treasury.bump = bump;
        treasury.slot = slot;
        treasury.treasurer_address = ctx.accounts.treasurer.key();
        treasury.mint_address = ctx.accounts.treasury_mint.key();
        treasury.name = name;
        treasury.labels = Vec::new();
        treasury.last_known_balance_units = 0;
        treasury.last_known_balance_slot = 0;
        treasury.last_known_balance_block_time = 0;
        treasury.allocation_reserved_units = 0;
        treasury.allocation_assigned_units = 0;
        treasury.total_withdrawals_units = 0;
        treasury.total_streams = 0;
        treasury.created_on_utc = Clock::get()?.unix_timestamp as u64 * 1000u64;
        treasury.depletion_units_per_second = 0.0;
        treasury.treasury_type = treasury_type;
        treasury.auto_close = auto_close;
        treasury.initialized = true;

        // Initialize Treasury Mint
        let cpi_accounts = InitializeMint {
            mint: ctx.accounts.treasury_mint.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        initialize_mint(
            cpi_ctx, 
            ctx.accounts.treasury_mint.decimals,
            &ctx.accounts.treasury.key(), 
            Some(&ctx.accounts.treasury.key())
        )?;

        // Fee
        let fee_lamports = CREATE_TREASURY_FLAT_FEE * LAMPORTS_PER_SOL as f64;
        let pay_fee_ix = solana_program::system_instruction::transfer(
            &ctx.accounts.treasurer.key(),
            &ctx.accounts.fee_treasury.key(),
            fee_lamports as u64
        );

        solana_program::program::invoke(&pay_fee_ix, &[
            ctx.accounts.treasurer.to_account_info(),
            ctx.accounts.fee_treasury.to_account_info(),
            ctx.accounts.system_program.to_account_info()
        ])
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

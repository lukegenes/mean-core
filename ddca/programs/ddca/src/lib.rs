use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, TokenAccount, Token, Transfer, CloseAccount };
use anchor_spl::associated_token::AssociatedToken;
use hybrid_liquidity_ag::cpi::accounts::Swap;
use std::cmp;

// Constants
pub mod ddca_operating_account {
    solana_program::declare_id!("6u1Hc9AqC6AvpYDQcFjhMVqAwcQ83Kn5TVm6oWMjDDf1");
}

// hybrid liquidity aggregator program
pub mod hla_program {
    solana_program::declare_id!("EPa4WdYPcGGdwEbq425DMZukU2wDUE1RWAGrPbRYSLRE");
}
pub mod hla_ops_accountsss {
    solana_program::declare_id!("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
}

// pub const CREATE_FLAT_LAMPORT_FEE: u64 = 10000;
// pub const ADD_FUNDS_PERCENT_TOKEN_FEE: f64 = 0.003;
// pub const CREATE_WITH_FUNDS_PERCENT_TOKEN_FEE: f64 = 0.003;
pub const WITHDRAW_PERCENT_TOKEN_FEE: f64 = 0.005;
// pub const STOP_FLAT_LAMPORT_FEE: u64 = 10000;
// pub const START_FLAT_LAMPORT_FEE: u64 = 10000;
pub const LAMPORTS_PER_SOL: u64 = 1000000000;
pub const SINGLE_SWAP_MINIMUM_LAMPORT_GAS_FEE: u64 = 20000000; //20 million

declare_id!("3nmm1awnyhABJdoA25MYVksxz1xnpUFeepJJyRTZfsyD");

#[program]
pub mod ddca {
    use super::*;

    pub fn create(
        ctx: Context<CreateInputAccounts>,
        block_height: u64, 
        pda_bump: u8,
        deposit_amount: u64,
        amount_per_swap: u64,
        interval_in_seconds: u64,
    ) -> ProgramResult {

        let start_ts = Clock::get()?.unix_timestamp as u64;

        ctx.accounts.ddca_account.owner_acc_addr = *ctx.accounts.owner_account.key;
        ctx.accounts.ddca_account.from_mint = *ctx.accounts.from_mint.as_ref().key; //ctx.accounts.from_token_account.mint;
        ctx.accounts.ddca_account.from_mint_decimals = ctx.accounts.from_mint.decimals;
        ctx.accounts.ddca_account.from_tacc_addr =  *ctx.accounts.from_token_account.to_account_info().key; //*ctx.accounts.from_token_account.as_ref().key;
        ctx.accounts.ddca_account.to_mint = *ctx.accounts.to_mint.as_ref().key;
        ctx.accounts.ddca_account.to_mint_decimals = ctx.accounts.to_mint.decimals;
        ctx.accounts.ddca_account.to_tacc_addr =  *ctx.accounts.to_token_account.to_account_info().key;
        ctx.accounts.ddca_account.block_height = block_height;
        ctx.accounts.ddca_account.pda_bump = pda_bump;
        ctx.accounts.ddca_account.total_deposits_amount = deposit_amount;
        ctx.accounts.ddca_account.amount_per_swap = amount_per_swap;
        ctx.accounts.ddca_account.interval_in_seconds = interval_in_seconds;
        ctx.accounts.ddca_account.start_ts = start_ts;

        // if from_initial_amount == 0 {
        //     // transfer LAMPORT flat fees to the ddca operating account
        //     msg!("flat fees: transfering {} lamports from owner to operating account", CREATE_FLAT_LAMPORT_FEE);
        //     let ix = anchor_lang::solana_program::system_instruction::transfer(
        //         ctx.accounts.owner_account.key,
        //         ctx.accounts.operating_account.key,
        //         CREATE_FLAT_LAMPORT_FEE,
        //     );

        //     anchor_lang::solana_program::program::invoke(
        //         &ix,
        //         &[
        //             ctx.accounts.owner_account.to_account_info(),
        //             ctx.accounts.operating_account.to_account_info(),
        //         ],
        //     )?;

        //     return Ok(());
        // }

        if deposit_amount % amount_per_swap != 0 {
            return Err(ErrorCode::InvalidAmounts.into());
        }
        let swap_count: u64 = deposit_amount / amount_per_swap;
        if swap_count == 1 {
            return Err(ErrorCode::InvalidSwapsCount.into());
        }

        // // transfer Token percentage fee to the ddca operating account
        // let from_mint_decimals = ctx.accounts.from_mint.decimals;
        // // msg!("from_mint_decimals: {}", from_mint_decimals);
        // let add_funds_fee = spl_token::ui_amount_to_amount(from_initial_amount as f64 * CREATE_WITH_FUNDS_PERCENT_TOKEN_FEE, from_mint_decimals);
        // msg!("'create with funds' fee: {}", add_funds_fee);
        // if add_funds_fee > 0 {
        //     token::transfer(
        //         ctx.accounts.into_transfer_fee_to_operating_context(),
        //         add_funds_fee,
        //     )?;
        // }

        // transfer enough SOL gas budget to the ddca account to pay future recurring swaps fees (network + amm fees)
        let recurring_lamport_fees = swap_count * SINGLE_SWAP_MINIMUM_LAMPORT_GAS_FEE;
        msg!("transfering {} lamports ({} SOL) from owner to ddca account for next {} swaps", recurring_lamport_fees, recurring_lamport_fees as f64 / LAMPORTS_PER_SOL as f64, swap_count);
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.owner_account.key,
            ctx.accounts.ddca_account.as_ref().key,
            recurring_lamport_fees,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.owner_account.to_account_info(),
                ctx.accounts.ddca_account.to_account_info(),
            ],
        )?;

        // transfer Token initial amount to ddca 'from' token account
        msg!("Depositing: {} of mint: {} into the ddca", deposit_amount, ctx.accounts.from_mint.key());
        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            deposit_amount,
        )?;
        
        Ok(())
    }

    pub fn wake_and_swap<'info>(
        ctx: Context<'_, '_, '_, 'info, WakeAndSwapInputAccounts<'info>>,
        swap_min_out_amount: u64,
        swap_slippage: u8,
    ) -> ProgramResult {

        // if ctx.remaining_accounts.len() == 0 {
        //     return Err(ProgramError::Custom(1)); // TODO: create proper error
        // }

        // check paused
        if ctx.accounts.ddca_account.is_paused {
            return Err(ErrorCode::DdcaIsPaused.into());
        }

        // check balance
        if ctx.accounts.from_token_account.amount < ctx.accounts.ddca_account.amount_per_swap {
            return Err(ErrorCode::InsufficientBalanceForSwap.into());
        }

        // check schedule
        let start_ts = ctx.accounts.ddca_account.start_ts;
        let interval = ctx.accounts.ddca_account.interval_in_seconds;
        let last_ts = ctx.accounts.ddca_account.last_completed_swap_ts;
        let now_ts = Clock::get()?.unix_timestamp as u64;
        let max_diff_in_secs = cmp::min(interval / 100, 3600); // +/-1% up to 3600 sec (ok for min interval = 5 min)
        let prev_checkpoint = (now_ts - start_ts) / interval;
        let prev_ts = start_ts + prev_checkpoint * interval;
        let next_checkpoint = prev_checkpoint + 1;
        let next_ts = start_ts + next_checkpoint * interval;
        let checkpoint_ts: u64;
        msg!("DDCA schedule: {{ start_ts: {}, interval: {}, last_ts: {}, now_ts: {}, max_diff_in_secs: {}, low: {}, high: {}, low_ts: {}, high_ts: {} }}",
                                start_ts, interval, last_ts, now_ts, max_diff_in_secs, prev_checkpoint, next_checkpoint, prev_ts, next_ts);

        if last_ts != prev_ts && now_ts >= (prev_ts - max_diff_in_secs) && now_ts <= (prev_ts + max_diff_in_secs) {
            checkpoint_ts = prev_ts;
            msg!("valid schedule");
        }
        else if last_ts != next_ts && now_ts >= (next_ts - max_diff_in_secs) && now_ts <= (next_ts + max_diff_in_secs) {
            checkpoint_ts = next_ts;
            msg!("valid schedule");
        }
        else {
            return Err(ErrorCode::InvalidSwapSchedule.into());
        }
        ctx.accounts.ddca_account.last_completed_swap_ts = checkpoint_ts;
        
        msg!("Executing scheduled swap at {}", checkpoint_ts);
        solana_program::log::sol_log_compute_units();

        // call hla to execute the first swap
        let hla_cpi_program = ctx.accounts.hla_program.clone();
        let hla_cpi_accounts = Swap {
            hla_ops_account: ctx.accounts.hla_operating_account.clone(),
            hla_ops_token_account: ctx.accounts.hla_operating_from_token_account.to_account_info().clone(),
            vault_account: ctx.accounts.ddca_account.to_account_info().clone(),
            from_token_account: ctx.accounts.from_token_account.to_account_info().clone(),
            from_token_mint: ctx.accounts.from_mint.to_account_info().clone(),
            to_token_account: ctx.accounts.to_token_account.to_account_info().clone(),
            to_token_mint: ctx.accounts.to_mint.to_account_info().clone(),
            token_program_account: ctx.accounts.token_program.to_account_info().clone(),
        };

        let seeds = &[
            ctx.accounts.ddca_account.owner_acc_addr.as_ref(),
            &ctx.accounts.ddca_account.block_height.to_be_bytes(),
            b"ddca-seed",
            &[ctx.accounts.ddca_account.pda_bump],
        ];

        let seeds_sign = &[&seeds[..]];

        let hla_cpi_ctx = CpiContext::new(hla_cpi_program, hla_cpi_accounts)
        .with_signer(seeds_sign)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        hybrid_liquidity_ag::cpi::swap(hla_cpi_ctx, ctx.accounts.ddca_account.amount_per_swap, swap_min_out_amount, swap_slippage);

        
        Ok(())
    }

    pub fn close(ctx: Context<CloseInputAccounts>) -> ProgramResult {
        // Transferring from vault token account to vault onwer token account
        let seeds = &[
            ctx.accounts.owner_account.key.as_ref(),
            &ctx.accounts.ddca_account.block_height.to_be_bytes(),
            b"ddca-seed",
            &[ctx.accounts.ddca_account.pda_bump],
        ];

        // from
        if ctx.accounts.ddca_from_token_account.amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_from_to_owner_context()
                    .with_signer(&[&seeds[..]]),
                ctx.accounts.ddca_from_token_account.amount,
            )?;
        }

        token::close_account(
            ctx.accounts
                .into_close_from_context()
                .with_signer(&[&seeds[..]]),
        )?;

        // to
        if ctx.accounts.ddca_to_token_account.amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_to_owner_context()
                    .with_signer(&[&seeds[..]]),
                ctx.accounts.ddca_to_token_account.amount,
            )?;
        }

        token::close_account(
            ctx.accounts
                .into_close_to_context()
                .with_signer(&[&seeds[..]]),
        )?;

        Ok(())
    }
}

// DERIVE ACCOUNTS

#[derive(Accounts)]
#[instruction(
    block_height: u64, 
    pda_bump: u8,
    deposit_amount: u64,
    amount_per_swap: u64,
    interval_in_seconds: u64,
    )]
pub struct CreateInputAccounts<'info> {
    // owner
    #[account(mut)]
    pub owner_account: Signer<'info>,
    #[account(
        mut,
        constraint = owner_from_token_account.amount >= deposit_amount
    )]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    // ddca
    #[account(
        init, 
        seeds = [
            owner_account.key().as_ref(), 
            &block_height.to_be_bytes(), 
            b"ddca-seed"
            ],
        bump = pda_bump,
        payer = owner_account, 
        space = 8 + DdcaAccount::LEN,
        constraint = amount_per_swap > 0,
        constraint = interval_in_seconds >= 300, // minimum inverval is 5 min
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    pub from_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = from_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(constraint = from_mint.key() != to_mint.key())]
    pub to_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = to_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = ddca_operating_account::ID)]
    pub operating_account: AccountInfo<'info>,
    #[account(
        mut,
        //TODO: uncomment when https://github.com/project-serum/anchor/pull/843 is released
        // associated_token::mint = from_mint, 
        // associated_token::authority = ddca_operating_account, 
    )]
    pub operating_from_token_account: Box<Account<'info, TokenAccount>>,
    // system and spl
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct WakeAndSwapInputAccounts<'info> {
    // ddca
    #[account(mut)]
    pub ddca_account: Account<'info, DdcaAccount>,
    #[account(constraint = from_mint.key() == ddca_account.from_mint)]
    pub from_mint:  Account<'info, Mint>,
    #[account(mut)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(constraint = to_mint.key() == ddca_account.to_mint)]
    pub to_mint:  Account<'info, Mint>, 
    #[account(mut)]
    pub to_token_account: Box<Account<'info, TokenAccount>>,
    // Hybrid Liquidity Aggregator
    #[account(address = hla_program::ID)]
    pub hla_program: AccountInfo<'info>,
    #[account(mut, address = hla_ops_accountsss::ID)]
    pub hla_operating_account: AccountInfo<'info>,
    #[account(mut)]
    pub hla_operating_from_token_account: Box<Account<'info, TokenAccount>>,
    // system and spl
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct CloseInputAccounts<'info> {
    pub owner_account: Signer<'info>,
    #[account(mut)]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = ddca_account.owner_acc_addr == *owner_account.key,
        close = owner_account,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    #[account(
        mut,
        // close = owner_account,
    )]
    pub ddca_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        // close = owner_account,
    )]
    pub ddca_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(address = ddca_operating_account::ID)]
    pub operating_account: AccountInfo<'info>,
    #[account( mut)]
    pub operating_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account( mut)]
    pub operating_to_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

// ACCOUNT STRUCTS

#[account]
pub struct DdcaAccount {
    pub owner_acc_addr: Pubkey, //32 bytes
    pub from_mint: Pubkey, //32 bytes
    pub from_mint_decimals: u8, //1 bytes
    pub from_tacc_addr: Pubkey, //32 bytes
    pub to_mint: Pubkey, //32 bytes
    pub to_mint_decimals: u8, //1 bytes
    pub to_tacc_addr: Pubkey, //32 bytes
    pub block_height: u64, //8 bytes
    pub pda_bump: u8, //1 byte
    pub total_deposits_amount: u64, //8 bytes
    pub amount_per_swap: u64, //8 bytes
    pub start_ts: u64, //8 bytes
    pub interval_in_seconds: u64, //8 bytes
    pub last_completed_swap_ts: u64, //8 bytes
    pub is_paused: bool, //8 bytes
}

impl DdcaAccount {
    pub const LEN: usize = 32 + 32 + 1 + 32 + 32 + 1 + 32 + 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1;
}

//UTILS IMPL

impl<'info> CreateInputAccounts<'info> {
    fn into_transfer_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_from_token_account.to_account_info().clone(),
            to: self
                .from_token_account
                .to_account_info()
                .clone(),
            authority: self.owner_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn into_transfer_fee_to_operating_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_from_token_account.to_account_info().clone(),
            to: self
                .operating_from_token_account
                .to_account_info()
                .clone(),
            authority: self.owner_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> CloseInputAccounts<'info> {
    // from
    fn into_transfer_from_to_owner_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.ddca_from_token_account.to_account_info().clone(),
            to: self
                .owner_from_token_account
                .to_account_info()
                .clone(),
            authority: self.ddca_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn into_close_from_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.ddca_from_token_account.to_account_info().clone(),
            destination: self.owner_account.to_account_info().clone(),
            authority: self.ddca_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
    
    // to
    fn into_transfer_to_to_owner_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.ddca_to_token_account.to_account_info().clone(),
            to: self
                .owner_to_token_account
                .to_account_info()
                .clone(),
            authority: self.ddca_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn into_close_to_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.ddca_to_token_account.to_account_info().clone(),
            destination: self.owner_account.to_account_info().clone(),
            authority: self.ddca_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Deposit amount must be a multiple of the amount per swap")]
    InvalidAmounts,
    #[msg("The number of recurring swaps must be greater than 1")]
    InvalidSwapsCount,
    #[msg("This DDCA is paused")]
    DdcaIsPaused,
    #[msg("Insufficient balance for swap")]
    InsufficientBalanceForSwap,
    #[msg("This DDCA is not schedule for the provided time")]
    InvalidSwapSchedule,
}

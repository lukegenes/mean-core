use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, TokenAccount, Token, Transfer, CloseAccount };
use anchor_spl::associated_token::AssociatedToken;
use hla::cpi::accounts::Swap;
use std::cmp;
use std::convert::TryFrom;

// Constants
pub mod ddca_operating_account {
    solana_program::declare_id!("6u1Hc9AqC6AvpYDQcFjhMVqAwcQ83Kn5TVm6oWMjDDf1");
}

// hybrid liquidity aggregator program
// pub mod hla_program {
//     solana_program::declare_id!("B6gLd2uyVQLZMdC1s9C4WR7ZP9fMhJNh7WZYcsibuzN3");
// }

pub const WITHDRAW_TOKEN_FEE_NUMERATOR: u64 = 50;
pub const WITHDRAW_TOKEN_FEE_DENOMINATOR: u64 = 10000;
pub const LAMPORTS_PER_SOL: u64 = 1000000000;
pub const SINGLE_SWAP_MINIMUM_LAMPORT_GAS_FEE: u64 = 20000000; //20 million
pub const SWAP_MAX_PERCENT_SLIPPAGE: u64 = 100; // 1 %

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

        if deposit_amount % amount_per_swap != 0 {
            return Err(ErrorCode::InvalidAmounts.into());
        }
        
        let swap_count: u64 = deposit_amount.checked_div(amount_per_swap).unwrap();
        if swap_count == 1 {
            return Err(ErrorCode::InvalidSwapsCount.into());
        }

        let start_ts = Clock::get()?.unix_timestamp as u64;
        let current_slot = Clock::get()?.slot;

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
        ctx.accounts.ddca_account.created_slot = current_slot;
        ctx.accounts.ddca_account.last_deposit_slot = current_slot;
        ctx.accounts.ddca_account.owner_acc_addr = *ctx.accounts.owner_account.key;
        ctx.accounts.ddca_account.wake_acc_addr = *ctx.accounts.wake_account.key;

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
        msg!("depositing: {} of mint: {} into the ddca", deposit_amount, ctx.accounts.from_mint.key());
        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            deposit_amount,
        )?;

        // fund wake account for next swap
        let signature_fee_plus_rent = Rent::get()?.minimum_balance(0) + 2 * Fees::get()?.fee_calculator.lamports_per_signature;
        msg!("transfering {} lamports ({} SOL) from ddca to wake account for next swap", signature_fee_plus_rent, signature_fee_plus_rent as f64 / LAMPORTS_PER_SOL as f64);

        **ctx.accounts.ddca_account.to_account_info().try_borrow_mut_lamports()? -= signature_fee_plus_rent;
        **ctx.accounts.wake_account.to_account_info().try_borrow_mut_lamports()? += signature_fee_plus_rent;
        
        Ok(())
    }

    pub fn wake_and_swap<'info>(
        ctx: Context<'_, '_, '_, 'info, WakeAndSwapInputAccounts<'info>>,
        swap_min_out_amount: u64,
        swap_slippage: u64,
    ) -> ProgramResult {

        // check paused
        if ctx.accounts.ddca_account.is_paused {
            return Err(ErrorCode::DdcaIsPaused.into());
        }

        // check slippage non-negative and up to max %
        if swap_slippage == 0 || swap_slippage > SWAP_MAX_PERCENT_SLIPPAGE {
            return Err(ErrorCode::InvalidSwapSlippage.into());
        }

        // check balance
        if ctx.accounts.from_token_account.amount < ctx.accounts.ddca_account.amount_per_swap {
            return Err(ErrorCode::InsufficientBalanceForSwap.into());
        }

        // check schedule
        let start_ts = ctx.accounts.ddca_account.start_ts;
        let interval = ctx.accounts.ddca_account.interval_in_seconds;
        let last_completed_ts = ctx.accounts.ddca_account.last_completed_swap_ts;
        let now_ts = Clock::get()?.unix_timestamp as u64;
        let max_delta_in_secs = cmp::max(cmp::min(interval / 20, 7200), 300); // +/-5% min: 5 min, max: 2h
        let prev_checkpoint = (now_ts - start_ts) / interval;
        let prev_ts = start_ts + prev_checkpoint * interval;
        let next_checkpoint = prev_checkpoint + 1;
        let next_ts = start_ts + next_checkpoint * interval;
        let checkpoint_ts: u64;

        #[cfg(feature = "devnet")]
        msg!("sts: {}, its: {}, lts: {}, nts: {}, mds: {}, lo: {}, hi: {}, lots: {}, hits: {} }}", 
        start_ts, interval, last_completed_ts, now_ts, max_delta_in_secs, prev_checkpoint, next_checkpoint, prev_ts, next_ts);

        if now_ts >= (prev_ts - max_delta_in_secs) && now_ts <= (prev_ts + max_delta_in_secs) {
            checkpoint_ts = prev_ts;
        }
        else if now_ts >= (next_ts - max_delta_in_secs) && now_ts <= (next_ts + max_delta_in_secs) {
            checkpoint_ts = next_ts;
        }
        else {
            return Err(ErrorCode::InvalidSwapSchedule.into());
        }

        if last_completed_ts == checkpoint_ts {
            return Err(ErrorCode::SwapAlreadyCompleted.into());
        }

        // Token balances before the trade.
        let from_amount_before = token::accessor::amount(&ctx.accounts.from_token_account.to_account_info())?;
        let to_amount_before = token::accessor::amount(&ctx.accounts.to_token_account.to_account_info())?;

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
        
        solana_program::log::sol_log_compute_units();
        hla::cpi::swap(hla_cpi_ctx, ctx.accounts.ddca_account.amount_per_swap, swap_min_out_amount, swap_slippage);

        // Token balances after the trade.
        let from_amount_after = token::accessor::amount(&ctx.accounts.from_token_account.to_account_info())?;
        let to_amount_after = token::accessor::amount(&ctx.accounts.to_token_account.to_account_info())?;
 
        //  Calculate the delta, i.e. the amount swapped.
        let from_amount_delta = from_amount_before.checked_sub(from_amount_after).unwrap();
        let to_amount_delta = to_amount_after.checked_sub(to_amount_before).unwrap();

        let swap_rate =  u64::try_from(
            (to_amount_delta as u128)
            .checked_mul(10u128.pow(ctx.accounts.ddca_account.from_mint_decimals.into())).unwrap()
            .checked_div(from_amount_delta.into()).unwrap()
        ).unwrap();

        ctx.accounts.ddca_account.swap_avg_rate = {
            if swap_rate >= ctx.accounts.ddca_account.swap_avg_rate {
                ctx.accounts.ddca_account.swap_avg_rate + 
                swap_rate
                .checked_sub(ctx.accounts.ddca_account.swap_avg_rate).unwrap()
                .checked_div(ctx.accounts.ddca_account.swap_count + 1).unwrap()
            }
            else {
                ctx.accounts.ddca_account.swap_avg_rate -
                ctx.accounts.ddca_account.swap_avg_rate
                .checked_sub(swap_rate).unwrap()
                .checked_div(ctx.accounts.ddca_account.swap_count + 1).unwrap()
            }
        };

        ctx.accounts.ddca_account.last_completed_swap_ts = checkpoint_ts;
        ctx.accounts.ddca_account.swap_count += 1;

        let total_swaps_count: u64 = ctx.accounts.ddca_account.total_deposits_amount
            .checked_div(ctx.accounts.ddca_account.amount_per_swap).unwrap();

        if ctx.accounts.ddca_account.swap_count == total_swaps_count {
            ctx.accounts.ddca_account.is_paused = true;
        } else {
            let lamports_per_signature = Fees::get()?.fee_calculator.lamports_per_signature;
            // msg!("transfering {} lamports ({} SOL) from ddca to wake account for next swap", lamports_per_signature, lamports_per_signature as f64 / LAMPORTS_PER_SOL as f64);

            **ctx.accounts.ddca_account.to_account_info().try_borrow_mut_lamports()? -= lamports_per_signature;
            **ctx.accounts.wake_account.to_account_info().try_borrow_mut_lamports()? += lamports_per_signature;
        }
        
        Ok(())
    }

    pub fn add_funds(
        ctx: Context<AddFundsInputAccounts>,
        deposit_amount: u64,
    ) -> ProgramResult {

        if deposit_amount % ctx.accounts.ddca_account.amount_per_swap != 0 {
            return Err(ErrorCode::InvalidAmounts.into());
        }
        
        let swap_count: u64 = deposit_amount.checked_div(ctx.accounts.ddca_account.amount_per_swap).unwrap();

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

        // transfer Token amount to ddca 'from' token account
        msg!("depositing: {} of mint: {} into the ddca", deposit_amount, ctx.accounts.ddca_account.from_mint);
        token::transfer(
            ctx.accounts.into_transfer_to_ddca_context(),
            deposit_amount,
        )?;

        Fees::get()?.fee_calculator.lamports_per_signature;

        ctx.accounts.ddca_account.total_deposits_amount += deposit_amount;
        ctx.accounts.ddca_account.last_deposit_ts = Clock::get()?.unix_timestamp as u64;
        ctx.accounts.ddca_account.last_deposit_slot =  Clock::get()?.slot;
        
        if ctx.accounts.ddca_account.is_paused {
            ctx.accounts.ddca_account.is_paused = false;
        }

        // fund wake account for next swap
        let signature_fee = Fees::get()?.fee_calculator.lamports_per_signature;
        // msg!("transfering {} lamports ({} SOL) from ddca to wake account for next swap", signature_fee, signature_fee as f64 / LAMPORTS_PER_SOL as f64);

        **ctx.accounts.ddca_account.to_account_info().try_borrow_mut_lamports()? -= signature_fee;
        **ctx.accounts.wake_account.to_account_info().try_borrow_mut_lamports()? += signature_fee;

        Ok(())
    }

    pub fn withdraw(
        ctx: Context<WithdrawInputAccounts>,
        withdraw_amount: u64,) -> ProgramResult {
        
        let seeds = &[
            ctx.accounts.owner_account.key.as_ref(),
            &ctx.accounts.ddca_account.block_height.to_be_bytes(),
            b"ddca-seed",
            &[ctx.accounts.ddca_account.pda_bump],
        ];

        // transfer withdraw fee from ddca 'to' token account to operating 'to' token account
        let to_withdraw_fee = withdraw_amount
            .checked_mul(WITHDRAW_TOKEN_FEE_NUMERATOR)
            .unwrap()
            .checked_div(WITHDRAW_TOKEN_FEE_DENOMINATOR)
            .unwrap();
        if to_withdraw_fee > 0 {
            msg!("'to' withdraw fee: {}", to_withdraw_fee);
            token::transfer(
                ctx.accounts.into_transfer_to_fee_to_operating_context()
                .with_signer(&[&seeds[..]]),
                to_withdraw_fee,
            )?;
        }

        // Transferring from ddca 'to' token account to onwer 'to' token account
        let to_token_amount = {
            if to_withdraw_fee > withdraw_amount {
                withdraw_amount
            }
            else { withdraw_amount - to_withdraw_fee }
        };
        if to_token_amount > 0 {
            msg!("transfering 'to' token blanace: {} to owner", to_token_amount);
            token::transfer(
                ctx.accounts
                    .into_transfer_to_to_owner_context()
                    .with_signer(&[&seeds[..]]),
                    to_token_amount,
            )?;
        }

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

        // transfer withdraw fee to the ddca operating account

        // from
        let from_withdraw_fee = ctx.accounts.ddca_from_token_account.amount
            .checked_mul(WITHDRAW_TOKEN_FEE_NUMERATOR)
            .unwrap()
            .checked_div(WITHDRAW_TOKEN_FEE_DENOMINATOR)
            .unwrap();
        msg!("withdraw 'from' fee: {}", from_withdraw_fee);
        if from_withdraw_fee > 0 {
            token::transfer(
                ctx.accounts.into_transfer_from_fee_to_operating_context()
                .with_signer(&[&seeds[..]]),
                from_withdraw_fee,
            )?;
        }

        let from_token_amount = {
            if from_withdraw_fee > ctx.accounts.ddca_from_token_account.amount {
                ctx.accounts.ddca_from_token_account.amount
            }
            else { ctx.accounts.ddca_from_token_account.amount - from_withdraw_fee }
        };
        msg!("transfering 'from' token blanace: {} to owner", from_token_amount);
        if from_token_amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_from_to_owner_context()
                    .with_signer(&[&seeds[..]]),
                    from_token_amount,
            )?;
        }

        token::close_account(
            ctx.accounts
                .into_close_from_context()
                .with_signer(&[&seeds[..]]),
        )?;

        // to
        let to_withdraw_fee = ctx.accounts.ddca_to_token_account.amount
            .checked_mul(WITHDRAW_TOKEN_FEE_NUMERATOR)
            .unwrap()
            .checked_div(WITHDRAW_TOKEN_FEE_DENOMINATOR)
            .unwrap();
        if to_withdraw_fee > 0 {
            msg!("'to' withdraw fee: {}", to_withdraw_fee);
            token::transfer(
                ctx.accounts.into_transfer_to_fee_to_operating_context()
                .with_signer(&[&seeds[..]]),
                to_withdraw_fee,
            )?;
        }

        let to_token_amount = {
            if to_withdraw_fee > ctx.accounts.ddca_to_token_account.amount {
                ctx.accounts.ddca_to_token_account.amount
            }
            else { ctx.accounts.ddca_to_token_account.amount - to_withdraw_fee }
        };
        if to_token_amount > 0 {
            msg!("transfering 'to' token blanace: {} to owner", to_token_amount);
            token::transfer(
                ctx.accounts
                    .into_transfer_to_to_owner_context()
                    .with_signer(&[&seeds[..]]),
                    to_token_amount,
            )?;
        }

        token::close_account(
            ctx.accounts
                .into_close_to_context()
                .with_signer(&[&seeds[..]]),
        )?;

        // defund wake account
        let lamports_to_return = ctx.accounts.wake_account.to_account_info().lamports();
        if lamports_to_return > 0 {

            msg!("returning {} lamports ({} SOL) from wake account to ddca account", lamports_to_return, lamports_to_return as f64 / LAMPORTS_PER_SOL as f64);
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.wake_account.key,
                ctx.accounts.ddca_account.as_ref().key,
                lamports_to_return,
            );

            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.wake_account.to_account_info(),
                    ctx.accounts.ddca_account.to_account_info(),
                ],
            )?;
        }

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
        space = 500, // 8 + DdcaAccount::LEN,
        constraint = amount_per_swap > 0,
        constraint = interval_in_seconds >= 3600, // minimum inverval: 1 hour
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
    #[account(mut)]
    pub wake_account: AccountInfo<'info>,
    // system and spl
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct WakeAndSwapInputAccounts<'info> {
    // // owner
    // #[account(mut)]
    // pub owner_account: Signer<'info>,
    #[account(
        mut,
        // address = ddca_account.wake_acc_addr,
        constraint = *wake_account.key == ddca_account.owner_acc_addr || *wake_account.key == ddca_account.wake_acc_addr,
    )]
    pub wake_account: Signer<'info>,
    // ddca
    #[account(
        mut,
        constraint = ddca_account.from_tacc_addr == *from_token_account.to_account_info().key,
        constraint = ddca_account.to_tacc_addr == *to_token_account.to_account_info().key,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    // #[account(constraint = from_mint.key() == ddca_account.from_mint)]
    pub from_mint:  Account<'info, Mint>,
    #[account(mut)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    // #[account(constraint = to_mint.key() == ddca_account.to_mint)]
    pub to_mint:  Account<'info, Mint>, 
    #[account(mut)]
    pub to_token_account: Box<Account<'info, TokenAccount>>,
    // Hybrid Liquidity Aggregator
    // #[account(address = hla_program::ID)]
    #[account(address = hla::ID)]
    pub hla_program: AccountInfo<'info>,
    #[account(mut)]
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
#[instruction(deposit_amount: u64)]
pub struct AddFundsInputAccounts<'info> {
    // owner
    #[account(mut)]
    pub owner_account: Signer<'info>,
    #[account(
        mut,
        constraint = deposit_amount > 0,
        constraint = owner_from_token_account.amount >= deposit_amount,
    )]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    // ddca
    #[account(
        mut,
        constraint = ddca_account.owner_acc_addr == *owner_account.key,
        constraint = ddca_account.from_tacc_addr == *from_token_account.to_account_info().key,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    #[account(mut)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = ddca_account.wake_acc_addr
    )]
    pub wake_account: AccountInfo<'info>,
    // system and spl
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(withdraw_amount: u64)]
pub struct WithdrawInputAccounts<'info> {
    pub owner_account: Signer<'info>,
    #[account(mut)]
    pub owner_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = ddca_account.owner_acc_addr == *owner_account.key,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    #[account(
        mut,
        constraint = withdraw_amount > 0,
        constraint = ddca_to_token_account.amount >= withdraw_amount,
        constraint = ddca_account.to_tacc_addr == *ddca_to_token_account.to_account_info().key,
    )]
    pub ddca_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(address = ddca_operating_account::ID)]
    pub operating_account: AccountInfo<'info>,
    #[account(
        mut,
        //TODO: uncomment when https://github.com/project-serum/anchor/pull/843 is released
        // associated_token::mint = from_mint, 
        // associated_token::authority = ddca_operating_account,
    )]
    pub operating_to_token_account: Box<Account<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct CloseInputAccounts<'info> {
    pub owner_account: Signer<'info>,
    #[account(mut)]
    pub wake_account: Signer<'info>,
    #[account(mut)]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub owner_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = ddca_account.owner_acc_addr == *owner_account.key,
        constraint = ddca_account.wake_acc_addr == *wake_account.key,
        constraint = ddca_account.from_tacc_addr == *ddca_from_token_account.to_account_info().key,
        constraint = ddca_account.to_tacc_addr == *ddca_to_token_account.to_account_info().key,
        close = owner_account,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    #[account(mut)]
    pub ddca_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub ddca_to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(address = ddca_operating_account::ID)]
    pub operating_account: AccountInfo<'info>,
    #[account(
        mut,
        //TODO: uncomment when https://github.com/project-serum/anchor/pull/843 is released
        // associated_token::mint = from_mint, 
        // associated_token::authority = ddca_operating_account,
    )]
    pub operating_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        //TODO: uncomment when https://github.com/project-serum/anchor/pull/843 is released
        // associated_token::mint = from_mint, 
        // associated_token::authority = ddca_operating_account,
    )]
    pub operating_to_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
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
    pub is_paused: bool, //1 bytes
    
    pub swap_count: u64, //8 bytes
    pub swap_avg_rate: u64, //8 bytes
    pub last_deposit_ts: u64, //8 bytes
    pub wake_acc_addr: Pubkey, //32 bytes
    pub created_slot: u64, //8 bytes
    pub last_deposit_slot: u64, //8 bytes
}

impl DdcaAccount {
    pub const LEN: usize = 32 + 32 + 1 + 32 + 32 + 1 + 32 + 8 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 32 + 8 + 8; // 6x32 + 11x8 + 4x1 = 284 (max = 492)
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
}

impl<'info> AddFundsInputAccounts<'info> {
    fn into_transfer_to_ddca_context(
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
}

fn to_transfer_from_ddca_cpi_context<'x, 'y, 'z, 'info>(
    ddca_source: &Box<Account<'info, TokenAccount>>, 
    dest: &Box<Account<'info, TokenAccount>>, 
    ddca: &Account<'info, DdcaAccount>, 
    token_program: &Program<'info, Token>
) -> CpiContext<'x, 'y, 'z, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: ddca_source.to_account_info().clone(),
            to: dest
                .to_account_info()
                .clone(),
            authority: ddca.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    
}

impl<'info> WithdrawInputAccounts<'info> {
    // to fee
    fn into_transfer_to_fee_to_operating_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        to_transfer_from_ddca_cpi_context(&self.ddca_to_token_account, &self.operating_to_token_account, &self.ddca_account, &self.token_program)
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
}

impl<'info> CloseInputAccounts<'info> {
    // from fee
    fn into_transfer_from_fee_to_operating_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        to_transfer_from_ddca_cpi_context(&self.ddca_from_token_account, &self.operating_from_token_account, &self.ddca_account, &self.token_program)
    }
    // to fee
    fn into_transfer_to_fee_to_operating_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        to_transfer_from_ddca_cpi_context(&self.ddca_to_token_account, &self.operating_to_token_account, &self.ddca_account, &self.token_program)
    }
    
    // from
    fn into_transfer_from_to_owner_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        to_transfer_from_ddca_cpi_context(&self.ddca_from_token_account, &self.owner_from_token_account, &self.ddca_account, &self.token_program)
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
        to_transfer_from_ddca_cpi_context(&self.ddca_to_token_account, &self.owner_to_token_account, &self.ddca_account, &self.token_program)
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
    #[msg("Invalid swap slippage")]
    InvalidSwapSlippage,
    #[msg("Invalid interval")]
    InvalidInterval,
    #[msg("A swap for this time window was already completed")]
    SwapAlreadyCompleted,
}

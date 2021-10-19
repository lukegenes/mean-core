use anchor_lang::prelude::*;
use anchor_spl::token::*;
use stable_swap_anchor::{Swap, SwapUserContext, SwapToken, SwapOutput};
use crate::errors::*;
use crate::utils::*;
use crate::state::{SwapInfo, AGGREGATOR_FEE};

pub fn swap<'info>(
    swap_info: SwapInfo<'info>

) -> ProgramResult {

    msg!("Intializing SABER Swap");
    let swap_ctx = get_swap_context(swap_info.clone())?;

    let _swap = stable_swap_anchor::swap(
        swap_ctx, 
        swap_info.from_amount, 
        swap_info.min_out_amount
    );

    // fee
    let fee_amount = (swap_info.from_amount as f64) * AGGREGATOR_FEE / 100f64;
    let transfer_ctx = get_transfer_context(swap_info.clone())?;

    transfer(
        transfer_ctx,
        fee_amount as u64
    )?;

    Ok(())
}

fn get_swap_context<'info>(
    swap_info: SwapInfo<'info>

) -> Result<CpiContext<'_, '_, '_, 'info, Swap<'info>>> {

    let acounts_iter = &mut swap_info.remaining_accounts.iter();
    let cpi_program_info = next_account_info(acounts_iter)?;
    let swap_account_info = next_account_info(acounts_iter)?;
    let swap_authority_info = next_account_info(acounts_iter)?;
    let token_program_info = next_account_info(acounts_iter)?;
    let reserve_input_account_info = next_account_info(acounts_iter)?;
    let reserve_output_account_info = next_account_info(acounts_iter)?;
    let admin_destination_info = next_account_info(acounts_iter)?;
    let clock_info = next_account_info(acounts_iter)?;

    let cpi_accounts = Swap {
        user: SwapUserContext {
            swap: swap_account_info.to_account_info(),
            swap_authority: swap_authority_info.to_account_info(),
            user_authority: swap_info.accounts.vault_account.to_account_info(), // CHECK
            token_program: token_program_info.to_account_info(),
            clock: clock_info.to_account_info()
        },
        input: SwapToken {
            user: swap_info.accounts.from_token_account.to_account_info(), // CHECK
            reserve: reserve_input_account_info.to_account_info()
        },
        output: SwapOutput {
            user_token: SwapToken {
                user: swap_info.accounts.to_token_account.to_account_info(), // CHECK
                reserve: reserve_output_account_info.to_account_info()
            },
            fees: admin_destination_info.to_account_info()
        }
    };

    Ok(CpiContext::new(
        cpi_program_info.to_account_info(), 
        cpi_accounts
    ))
}



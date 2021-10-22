use anchor_lang::prelude::*;
use anchor_spl::token::*;
use spl_token_swap::*;
use crate::utils::*;
use crate::state::*;

pub fn swap<'info>(
    swap_info: SwapInfo<'info>

) -> ProgramResult {

    let acounts_iter = &mut swap_info.remaining_accounts.iter();
    let pool_account_info = next_account_info(acounts_iter)?;
    let cpi_program_info = next_account_info(acounts_iter)?;
    let swap_account_info = next_account_info(acounts_iter)?;
    let swap_authority_info = next_account_info(acounts_iter)?;
    let pool_source_account_info = next_account_info(acounts_iter)?;
    let pool_destination_account_info = next_account_info(acounts_iter)?;
    let fee_account_info = next_account_info(acounts_iter)?;

    let fee_amount = (swap_info.from_amount as f64) * AGGREGATOR_PERCENT_FEE / 100f64;
    let swap_amount = (swap_info.from_amount as f64) - fee_amount;

    let swap_ix = spl_token_swap::instruction::swap(
        cpi_program_info.key,
        swap_info.accounts.token_program_account.key,
        swap_account_info.key,
        swap_authority_info.key,
        swap_info.accounts.vault_account.key,
        swap_info.accounts.from_token_account.key,
        pool_source_account_info.key,
        pool_destination_account_info.key,
        swap_info.accounts.to_token_account.key,
        pool_account_info.key,
        fee_account_info.key,
        None,
        spl_token_swap::instruction::Swap {
            amount_in: swap_amount as u64,
            minimum_amount_out: swap_info.min_out_amount as u64
        }
    )?;

    let _result = solana_program::program::invoke_signed(
        &swap_ix,
        &[
            cpi_program_info.to_account_info(),
            swap_info.accounts.token_program_account.to_account_info(),
            swap_account_info.to_account_info(),
            swap_authority_info.to_account_info(),
            swap_info.accounts.vault_account.to_account_info(),
            swap_info.accounts.from_token_account.to_account_info(),
            pool_source_account_info.to_account_info(),
            pool_destination_account_info.to_account_info(),
            swap_info.accounts.to_token_account.to_account_info(),
            pool_account_info.to_account_info(),
            fee_account_info.to_account_info()
        ],
        &[]
    );

    let transfer_ctx = get_transfer_context(swap_info)?;

    transfer(
        transfer_ctx,
        fee_amount as u64
    )?;

    Ok(())
}



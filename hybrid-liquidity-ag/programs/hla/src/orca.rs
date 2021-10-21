use anchor_lang::prelude::*;
use anchor_spl::token::*;
use spl_token_swap::*;
use crate::utils::*;
use crate::state::*;

pub fn swap<'info>(
    swap_info: SwapInfo<'info>

) -> ProgramResult {

    let acounts_iter = &mut swap_info.remaining_accounts.iter();
    let program_info = next_account_info(acounts_iter)?.to_account_info();
    let token_swap_info = next_account_info(acounts_iter)?.to_account_info();
    let swap_authority_info = next_account_info(acounts_iter)?.to_account_info();
    let pool_source_account_info = next_account_info(acounts_iter)?.to_account_info();
    let pool_destination_account_info = next_account_info(acounts_iter)?.to_account_info();
    let pool_mint_account_info = next_account_info(acounts_iter)?.to_account_info();
    let fee_account_info = next_account_info(acounts_iter)?.to_account_info();
    let host_fee_account_info = next_account_info(acounts_iter)?.to_account_info();
    let token_program_account_info = next_account_info(acounts_iter)?.to_account_info();

    let fee_amount = (swap_info.from_amount as f64) * AGGREGATOR_PERCENT_FEE / 100f64;
    let swap_amount = (swap_info.from_amount as f64) - fee_amount;
    let signer_seed: &[&[_]] = &[swap_info.accounts.vault_account.key.as_ref()];

    let swap_ix = spl_token_swap::instruction::swap(
        program_info.key,
        token_program_account_info.key,
        token_swap_info.key,
        swap_authority_info.key,
        swap_info.accounts.vault_account.key, // CHECK
        swap_info.accounts.from_token_account.to_account_info().key, // CHECK
        pool_source_account_info.key,
        pool_destination_account_info.key,
        swap_info.accounts.to_token_account.to_account_info().key, // CHECK
        pool_mint_account_info.key,
        fee_account_info.key,
        Some(host_fee_account_info.key),
        spl_token_swap::instruction::Swap {
            amount_in: swap_amount as u64,
            minimum_amount_out: swap_info.min_out_amount as u64
        }
    )?;

    let accounts = [
        program_info,
        token_program_account_info,
        token_swap_info,
        swap_authority_info,
        swap_info.accounts.vault_account.to_account_info(), // CHECK
        swap_info.accounts.from_token_account.to_account_info(), // CHECK
        pool_source_account_info,
        pool_destination_account_info,
        swap_info.accounts.to_token_account.to_account_info(), // CHECK
        pool_mint_account_info,
        fee_account_info,
        host_fee_account_info
    ];

    let _result = solana_program::program::invoke_signed(
        &swap_ix,
        &accounts,
        &[signer_seed],
    );

    let transfer_ctx = get_transfer_context(swap_info)?;

    transfer(
        transfer_ctx,
        fee_amount as u64
    )?;

    Ok(())
}



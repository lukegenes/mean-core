use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::state::*;
use crate::utils::*;
use std::mem::size_of;

use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub fn swap<'info>(
    swap_info: SwapInfo<'info>

) -> ProgramResult {

    let acounts_iter = &mut swap_info.remaining_accounts.iter();

    let _pool_account_info = next_account_info(acounts_iter)?;
    let cpi_program_info = next_account_info(acounts_iter)?;
    let amm_account_info = next_account_info(acounts_iter)?;
    let amm_authority_account_info = next_account_info(acounts_iter)?;
    let amm_open_orders_account_info = next_account_info(acounts_iter)?;
    let amm_target_orders_account_info = next_account_info(acounts_iter)?;
    let pool_coin_token_account_info = next_account_info(acounts_iter)?;
    let pool_pc_token_account_info = next_account_info(acounts_iter)?;
    let serum_program_info = next_account_info(acounts_iter)?;
    let serum_market_account_info = next_account_info(acounts_iter)?;
    let serum_bids_account_info = next_account_info(acounts_iter)?;
    let serum_asks_account_info = next_account_info(acounts_iter)?;
    let serum_event_queue_account_info = next_account_info(acounts_iter)?;
    let serum_coin_vault_account_info = next_account_info(acounts_iter)?;
    let serum_pc_vault_account_info = next_account_info(acounts_iter)?;
    let serum_vault_signer_account_info = next_account_info(acounts_iter)?;

    let fee_amount = (swap_info.from_amount as f64) * AGGREGATOR_PERCENT_FEE / 100f64;
    let swap_amount = (swap_info.from_amount as f64) - fee_amount;

    let swap_ix = raydium_swap(
        cpi_program_info.key,
        amm_account_info.key,
        amm_authority_account_info.key,
        amm_open_orders_account_info.key,
        amm_target_orders_account_info.key,
        pool_coin_token_account_info.key,
        pool_pc_token_account_info.key,        
        serum_program_info.key,
        serum_market_account_info.key,
        serum_bids_account_info.key,
        serum_asks_account_info.key,
        serum_event_queue_account_info.key,
        serum_coin_vault_account_info.key,
        serum_pc_vault_account_info.key,
        serum_vault_signer_account_info.key,
        swap_info.accounts.from_token_account.key,
        swap_info.accounts.to_token_account.key,
        swap_info.accounts.vault_account.key,
        swap_amount as u64,
        swap_info.min_out_amount as u64
    )?;

    let _result = solana_program::program::invoke_signed(
        &swap_ix,
        &[
            cpi_program_info.to_account_info(),
            amm_account_info.to_account_info(),
            amm_authority_account_info.to_account_info(),
            amm_open_orders_account_info.to_account_info(),
            amm_target_orders_account_info.to_account_info(),
            pool_coin_token_account_info.to_account_info(),
            pool_pc_token_account_info.to_account_info(),        
            serum_program_info.to_account_info(),
            serum_market_account_info.to_account_info(),
            serum_bids_account_info.to_account_info(),
            serum_asks_account_info.to_account_info(),
            serum_event_queue_account_info.to_account_info(),
            serum_coin_vault_account_info.to_account_info(),
            serum_pc_vault_account_info.to_account_info(),
            serum_vault_signer_account_info.to_account_info(),
            swap_info.accounts.from_token_account.to_account_info(),
            swap_info.accounts.to_token_account.to_account_info(),
            swap_info.accounts.vault_account.to_account_info()
        ],
        &[]
    );

    let transfer_ctx = get_transfer_context(swap_info.clone())?;

    transfer(
        transfer_ctx,
        fee_amount as u64
    )?;

    Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwapInstruction {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum AmmInstruction {
    Swap(SwapInstruction)
}

impl AmmInstruction {

    pub fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match &*self {
            Self::Swap(SwapInstruction{amount_in, minimum_amount_out}) => {
                buf.push(9);
                buf.extend_from_slice(&amount_in.to_le_bytes());
                buf.extend_from_slice(&minimum_amount_out.to_le_bytes());
            }
        }
        Ok(buf)
    }
}

/// Creates a 'raydium swap' instruction.
fn raydium_swap(
    program_id: &Pubkey,
    amm_id: &Pubkey,
    amm_authority: &Pubkey,
    amm_open_orders: &Pubkey,
    amm_target_orders: &Pubkey,
    pool_coin_token_account: &Pubkey,
    pool_pc_token_account: &Pubkey,
    serum_program_id: &Pubkey,
    serum_market: &Pubkey,
    serum_bids: &Pubkey,
    serum_asks: &Pubkey,
    serum_event_queue: &Pubkey,
    serum_coin_vault_account: &Pubkey,
    serum_pc_vault_account: &Pubkey,
    serum_vault_signer: &Pubkey,
    user_source_token_account: &Pubkey,
    user_destination_token_account: &Pubkey,
    user_source_owner: &Pubkey,
    amount_in: u64,
    minimum_amount_out: u64

) -> Result<Instruction, ProgramError> {

    let data = AmmInstruction::Swap(SwapInstruction{ amount_in, minimum_amount_out }).pack()?;
    let accounts = vec![
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new(*amm_id, false),
        AccountMeta::new_readonly(*amm_authority, false),
        AccountMeta::new(*amm_open_orders, false),
        AccountMeta::new(*amm_target_orders, false),
        AccountMeta::new(*pool_coin_token_account, false),
        AccountMeta::new(*pool_pc_token_account, false),
        AccountMeta::new_readonly(*serum_program_id, false),
        AccountMeta::new(*serum_market, false),
        AccountMeta::new(*serum_bids, false),
        AccountMeta::new(*serum_asks, false),
        AccountMeta::new(*serum_event_queue, false),
        AccountMeta::new(*serum_coin_vault_account, false),
        AccountMeta::new(*serum_pc_vault_account, false),
        AccountMeta::new_readonly(*serum_vault_signer, false),
        AccountMeta::new(*user_source_token_account, false),
        AccountMeta::new(*user_destination_token_account, false),
        AccountMeta::new_readonly(*user_source_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

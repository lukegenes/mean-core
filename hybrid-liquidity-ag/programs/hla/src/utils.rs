use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::state::*;
use crate::data::*;
use crate::errors::*;

pub fn get_pool(pool_account: &Pubkey) -> Result<PoolInfo> {
    let pools = get_pools();
    let lps = pools
        .iter()
        .filter(|p| p.account.eq(pool_account))
        .map(|p| (*p).clone())
        .collect::<Vec<PoolInfo>>();
    
    if lps.len() == 0
    {
        return Err(ErrorCode::PoolNotFound.into());
    }

    Ok(lps[0].clone())
}

pub fn get_transfer_context<'info>(
    swap_info: SwapInfo<'info>

) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {

    let cpi_program = swap_info.accounts.token_program_account.clone();
    let cpi_accounts = Transfer {
        from: swap_info.accounts.from_token_account.to_account_info().clone(),
        to: swap_info.accounts.hla_ops_token_account.to_account_info().clone(),
        authority: swap_info.accounts.hla_ops_account.clone()
    };

    Ok(CpiContext::new(
        cpi_program, 
        cpi_accounts
    ))
}
use anchor_lang::prelude::*;
use anchor_spl::token::*;
use crate::state::*;
use crate::errors::*;

pub fn get_transfer_context<'info>(
    swap_info: SwapInfo<'info>

) -> Result<CpiContext<'_, '_, '_, 'info, Transfer<'info>>> {

    let cpi_program = swap_info.accounts.token_program_account.to_account_info();
    let cpi_accounts = Transfer {
        from: swap_info.accounts.from_token_account.to_account_info(),
        to: swap_info.accounts.hla_ops_token_account.to_account_info(),
        authority: swap_info.accounts.vault_account.to_account_info()
    };

    Ok(CpiContext::new(
        cpi_program, 
        cpi_accounts
    ))
}
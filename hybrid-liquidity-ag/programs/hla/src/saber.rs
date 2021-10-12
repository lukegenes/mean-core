use anchor_lang::prelude::*;
use stable_swap_anchor::{Swap, SwapToken, SwapOutput, SwapUserContext};
use crate::errors::*;

pub fn swap<'info>(
    accounts: Vec<AccountInfo<'info>>,
    from_amount: f64,
    min_out_amount: f64

) -> ProgramResult {

    let swap_ctx = get_swap_context(accounts)?;

    stable_swap_anchor::swap(
        swap_ctx, 
        from_amount as u64, 
        min_out_amount as u64
    );

    Ok(())
}

fn get_swap_context<'a, 'b, 'c, 'info>(
    accounts: Vec<AccountInfo<'info>>

) -> Result<CpiContext<'a, 'b, 'c, 'info, Swap<'info>>> {

    let acounts_iter = &mut accounts.iter();
    let user_authority_info = next_account_info(acounts_iter)?.to_account_info();
    let user_input_account_info = next_account_info(acounts_iter)?.to_account_info();
    let user_output_account_info = next_account_info(acounts_iter)?.to_account_info();
    let cpi_program = next_account_info(acounts_iter)?.to_account_info();
    let swap_info = next_account_info(acounts_iter)?.to_account_info();
    let swap_authority_info = next_account_info(acounts_iter)?.to_account_info();
    let reserve_input_account_info = next_account_info(acounts_iter)?.to_account_info();
    let reserve_output_account_info = next_account_info(acounts_iter)?.to_account_info();
    let admin_destination_info = next_account_info(acounts_iter)?.to_account_info();
    let token_program_account_info = next_account_info(acounts_iter)?.to_account_info();
    let clock_account_info = next_account_info(acounts_iter)?.to_account_info();

    let cpi_accounts = Swap
    {
        user: SwapUserContext {
            swap: swap_info,
            swap_authority: swap_authority_info,
            user_authority: user_authority_info,
            token_program: token_program_account_info,
            clock: clock_account_info
        },
        input: SwapToken {
            user: user_input_account_info,
            reserve: reserve_input_account_info
        },
        output: SwapOutput {
            user_token: SwapToken {
                user: user_output_account_info,
                reserve: reserve_output_account_info
            },
            fees: admin_destination_info
        }
    };

    Ok(CpiContext::new(cpi_program, cpi_accounts))
}



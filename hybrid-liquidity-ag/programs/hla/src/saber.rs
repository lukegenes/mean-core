use anchor_lang::prelude::*;
use stable_swap_anchor::{self, SwapInfo};

pub fn swap(
    pool: &SwapInfo,
    from_amount: f64,
    min_out_amount: f64,
    slippage: u8

) -> ProgramResult{
    Ok(())
}



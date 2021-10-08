use anchor_lang::prelude::*;
use crate::state::*;
use crate::data::*;
use std::convert::TryInto;

pub fn get_client(protocol: Pubkey) -> ProgramResult {
    Ok(())
}

pub fn get_token_pair_pools(
    from: &Pubkey, 
    to: &Pubkey

) -> ProgramResult/*Vec<PoolInfo>*/ {

    // get_pools()
    //     .iter()
    //     .filter(|p| 
    //         p.tokens.iter().any(|t| t.eq(from)) && 
    //         p.tokens.iter().any(|t| t.eq(to))
    //     )
    //     .collect::<Vec<PoolInfo>>()

    Ok(())
}

pub fn get_optimal_pool(pools: Vec::<PoolInfo>) -> ProgramResult {
    Ok(())
}
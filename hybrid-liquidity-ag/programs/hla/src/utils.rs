use anchor_lang::prelude::*;
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
use anchor_lang::prelude::*;
use crate::state::*;
use crate::data::*;
use crate::errors::*;
use crate::saber::*;

pub fn get_client<'info>(protocol: &str) -> Box<Client<'info>> {

    match protocol {

        SABER => Box::new(SaberClient {
            protocol_account: SABER.parse().unwrap()
        }),

        _ => Box::new(SaberClient { 
            protocol_account: SABER.parse().unwrap()
        }),
    }
}

pub fn get_token_pair_pools(
    from: &Pubkey, 
    to: &Pubkey

) -> Vec<PoolInfo> {

    get_pools()
        .iter()
        .filter(|p| 
            p.tokens.iter().any(|t| t.eq(from)) && 
            p.tokens.iter().any(|t| t.eq(to))
        )
        .map(|p| (*p).clone())
        .collect()
}

pub fn get_optimal_pool(pools: Vec::<PoolInfo>) -> PoolInfo {
    pools[0].clone()
}
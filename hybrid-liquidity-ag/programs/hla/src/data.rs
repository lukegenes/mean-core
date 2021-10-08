use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::system_program;
use anchor_lang::prelude::*;
use crate::state::*;

pub fn get_pools() -> Vec<PoolInfo> {
    vec![
        PoolInfo {
            chain_id: 101,
            account: "2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf".parse().unwrap(),
            protocol_account: "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ".parse().unwrap(),
            amm_account: "YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe".parse().unwrap(),
            tokens: vec![
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".parse().unwrap(),
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse().unwrap()
            ]
        }
    ]
}
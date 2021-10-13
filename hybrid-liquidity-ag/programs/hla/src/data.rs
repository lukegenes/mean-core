use anchor_lang::prelude::*;
use crate::state::*;

pub fn get_pools() -> Vec<PoolInfo> {
    vec![
        PoolInfo {
            name: String::from("USDT-USDC Saber LP"),
            account: "2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf".parse().unwrap(),
            protocol_account: "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ".parse().unwrap(),
            amm_account: "YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe".parse().unwrap()
        },
        PoolInfo {
            name: String::from("USDC-USDT Orca LP"),
            account: "H2uzgruPvonVpCRhwwdukcpXK8TG17swFNzYFr2rtPxy".parse().unwrap(),
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".parse().unwrap(),
            amm_account: "F13xvvx45jVGd84ynK3c8T89UejQVxjCLtmHfPmAXAHP".parse().unwrap()
        },
        PoolInfo {
            name: String::from("SOL-USDC Orca LP"),
            account: "APDFRM3HMr8CAGXwKHiu2f5ePSpaiEJhaURwhsRrUUt9".parse().unwrap(),
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".parse().unwrap(),
            amm_account: "EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U".parse().unwrap()
        },
        PoolInfo {
            name: String::from("ETH-SOL Orca LP"),
            account: "71FymgN2ZUf7VvVTLE8jYEnjP3jSK1Frp2XT1nHs8Hob".parse().unwrap(),
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".parse().unwrap(),
            amm_account: "EuK3xDa4rWuHeMQCBsHf1ETZNiEQb5C476oE9u9kp8Ji".parse().unwrap()
        },
        PoolInfo {
            name: String::from("ETH-USDC Orca LP"),
            account: "3e1W6Aqcbuk2DfHUwRiRcyzpyYRRjg6yhZZcyEARydUX".parse().unwrap(),
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".parse().unwrap(),
            amm_account: "FgZut2qVQEyPBibaTJbbX2PxaMZvT1vjDebiVaDp5BWP".parse().unwrap()
        },
        PoolInfo {
            name: String::from("SOL-USDT Orca LP"),
            account: "FZthQCuYHhcfiDma7QrX7buDHwrZEd7vL8SjS6LQa3Tx".parse().unwrap(),
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".parse().unwrap(),
            amm_account: "Dqk7mHQBx2ZWExmyrR2S8X6UG75CrbbpK2FSBZsNYsw6".parse().unwrap()
        }
    ]
}
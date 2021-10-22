use crate::state::*;

pub fn get_pools<'info>() -> Vec<PoolInfo<'info>> {
    vec![
        PoolInfo {
            name: "USDT-USDC Saber LP",
            account: "2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf",
            protocol_account: "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ",
            amm_account: "YAkoNb6HKmSxQN9L8hiBE5tPJRsniSSMzND1boHmZxe"
            // account: "YakofBo4X3zMxa823THQJwZ8QeoU8pxPdFdxJs7JW57",
            // protocol_account: "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ",
            // amm_account: "VeNkoB1HvSP6bSeGybQDnx9wTWFsQb2NBCemeCDSuKL"
        },
        PoolInfo {
            name: "USDC-USDT Orca LP",
            account: "H2uzgruPvonVpCRhwwdukcpXK8TG17swFNzYFr2rtPxy",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
            amm_account: "F13xvvx45jVGd84ynK3c8T89UejQVxjCLtmHfPmAXAHP"
        },
        PoolInfo {
            name: "SOL-USDC Orca LP",
            account: "APDFRM3HMr8CAGXwKHiu2f5ePSpaiEJhaURwhsRrUUt9",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
            amm_account: "EGZ7tiLeH62TPV1gL8WwbXGzEPa9zmcpVnnkPKKnrE2U"
        },
        PoolInfo {
            name: "ETH-SOL Orca LP",
            account: "71FymgN2ZUf7VvVTLE8jYEnjP3jSK1Frp2XT1nHs8Hob",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
            amm_account: "EuK3xDa4rWuHeMQCBsHf1ETZNiEQb5C476oE9u9kp8Ji"
        },
        PoolInfo {
            name: "ETH-USDC Orca LP",
            account: "3e1W6Aqcbuk2DfHUwRiRcyzpyYRRjg6yhZZcyEARydUX",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
            amm_account: "FgZut2qVQEyPBibaTJbbX2PxaMZvT1vjDebiVaDp5BWP"
        },
        PoolInfo {
            name: "SOL-USDT Orca LP",
            account: "FZthQCuYHhcfiDma7QrX7buDHwrZEd7vL8SjS6LQa3Tx",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP",
            amm_account: "Dqk7mHQBx2ZWExmyrR2S8X6UG75CrbbpK2FSBZsNYsw6"
        }
    ]
}
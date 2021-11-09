use crate::state::*;
use std::vec::IntoIter;

pub fn get_pools<'info>() -> IntoIter<PoolInfo<'info>> {

    let data = vec![
        // USDT-USDC Saber LP
        PoolInfo {
            account: "2poo1w1DL6yd2WNTCnNTzDqkC6MBXq7axo77P16yrBuf",
            protocol_account: "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ"
        },
        // USDC-USDT Orca LP
        PoolInfo {
            account: "H2uzgruPvonVpCRhwwdukcpXK8TG17swFNzYFr2rtPxy",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // SOL-USDC Orca LP
        PoolInfo {
            account: "APDFRM3HMr8CAGXwKHiu2f5ePSpaiEJhaURwhsRrUUt9",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // ETH-SOL Orca LP
        PoolInfo {
            account: "71FymgN2ZUf7VvVTLE8jYEnjP3jSK1Frp2XT1nHs8Hob",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // ETH-USDC Orca LP
        PoolInfo {
            account: "3e1W6Aqcbuk2DfHUwRiRcyzpyYRRjg6yhZZcyEARydUX",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // SOL-USDT Orca LP
        PoolInfo {
            account: "FZthQCuYHhcfiDma7QrX7buDHwrZEd7vL8SjS6LQa3Tx",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // ORCA-USDC Orca LP
        PoolInfo {
            account: "n8Mpu28RjeYD7oUX3LG1tPxzhRZh3YYLRSHcHRdS3Zx",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // ORCA-SOL Orca LP
        PoolInfo {
            account: "2uVjAuRXavpM6h1scGQaxqb6HVaNRn6T2X7HHXTabz25",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // SBR-USDC Orca LP
        PoolInfo {
            account: "CS7fA5n4c2D82dUoHrYzS3gAqgqaoVSfgsr18kitp2xo",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // RAY-SOL Orca LP
        PoolInfo {
            account: "5kimD5W6yJpHRHCyPtnEyDsQRdiiJKivu5AqN3si82Jc",
            protocol_account: "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP"
        },
        // SOL-USDC LP
        PoolInfo {
            account: "8HoQnePLqPj4M7PUDzfw8e3Ymdwgc7NLGnaTUapubyvu",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // SOL-USDT LP
        PoolInfo {
            account: "Epm4KfTj4DMrvqn6Bwg2Tr2N8vhQuNbuK8bESFp4k33K",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // BTC-USDC LP
        PoolInfo {
            account: "2hMdRdVWZqetQsaHG8kQjdZinEMBz75vsoWTCob1ijXu",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // BTC-USDT LP
        PoolInfo {
            account: "DgGuvR9GSHimopo3Gc7gfkbKamLKrdyzWkq5yqA6LqYS",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // BTC-SRM LP
        PoolInfo {
            account: "AGHQxXb3GSzeiLTcLtXMS2D5GGDZxsB2fZYZxSB5weqB",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // ETH-USDC LP
        PoolInfo {
            account: "13PoKid6cZop4sj2GfoBeujnGfthUbTERdE5tpLCDLEY",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // ETH-USDT LP
        PoolInfo {
            account: "nPrB78ETY8661fUgohpuVusNCZnedYCgghzRJzxWnVb",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // RAY-USDC LP
        PoolInfo {
            account: "FbC6K13MzHvN42bXrtGaWsvZY9fxrackRSZcBGfjPc7m",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // RAY-USDT LP
        PoolInfo {
            account: "C3sT1R3nsw4AVdepvLTLKr5Gvszr7jufyBWUCvy4TUvT",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // RAY-SOL LP
        PoolInfo {
            account: "89ZKE4aoyfLBe2RuV6jM3JGNhaV18Nxh8eNtjRcndBip",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // RAY-ETH LP
        PoolInfo {
            account: "mjQH33MqZv5aKAbKHi8dG3g3qXeRQqq1GFcXceZkNSr",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // RAY-SRM LP
        PoolInfo {
            account: "7P5Thr9Egi2rvMmEuQkLn8x8e8Qro7u2U7yLD2tU2Hbe",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // SRM-USDC LP
        PoolInfo {
            account: "9XnZd82j34KxNLgQfz29jGbYdxsYznTWRpvZE3SRE7JG",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // SRM-USDT LP
        PoolInfo {
            account: "HYSAu42BFejBS77jZAZdNAWa3iVcbSRJSzp3wtqCbWwv",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // SRM-SOL LP
        PoolInfo {
            account: "AKJHspCwDhABucCxNLXUSfEzb7Ny62RqFtC9uNjJi4fq",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        },
        // SRM-ETH LP
        PoolInfo {
            account: "9VoY3VERETuc2FoadMSYYizF26mJinY514ZpEzkHMtwG",
            protocol_account: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        }
    ];

    data.into_iter()
}
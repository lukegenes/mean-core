use anchor_lang::prelude::*;

pub const HLA_OPS: &str = "FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX";
pub const RAYDIUM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
pub const SABER: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
pub const MERCURIAL: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";
pub const SERUM: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

#[derive(Clone, Debug)]
pub struct PoolInfo {
    pub account: Pubkey,
    pub protocol_account: Pubkey,
    pub amm_account: Pubkey
}
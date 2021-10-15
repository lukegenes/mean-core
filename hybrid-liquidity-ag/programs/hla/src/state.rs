use anchor_lang::prelude::*;
use anchor_spl::token::*;

pub const HLA_OPS: &str = "FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX";
pub const RAYDIUM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
pub const SABER: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
pub const MERCURIAL: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";
pub const SERUM: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
pub const AGGREGATOR_FEE: f64 = 0.05;

#[derive(Clone, Debug)]
pub struct PoolInfo {
    pub name: String,
    pub account: Pubkey,
    pub protocol_account: Pubkey,
    pub amm_account: Pubkey
}

#[derive(Accounts, Clone)]
#[
    instruction(
        from_amount: u64, 
        min_out_amount: u64, 
        slippage: u8
    )
]
pub struct Swap<'info> {
    #[account(mut)]
    pub vault_account: AccountInfo<'info>,
    pub from_token_mint: CpiAccount<'info, Mint>,
    pub from_token_account: CpiAccount<'info, TokenAccount>,
    pub to_token_mint: CpiAccount<'info, Mint>,
    pub to_token_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub hla_ops_account: AccountInfo<'info>,
    #[account(mut)]
    pub hla_ops_token_account: Account<'info, TokenAccount>,
    pub token_program_account: AccountInfo<'info>
}

#[derive(Clone)]
pub struct SwapInfo<'info> {
    pub accounts: Swap<'info>,
    pub remaining_accounts: Vec<AccountInfo<'info>>,
    pub from_amount: u64,
    pub min_out_amount: u64
}
use anchor_lang::prelude::*;

pub const MEAN_OPS: &str = "CLazQV1BhSrxfgRHko4sC8GYBU3DoHcX4xxRZd12Kohr";
pub const RAYDIUM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
pub const SABER: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
pub const MERCURIAL: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";
pub const SERUM: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

#[derive(Clone, Debug)]
pub struct PoolInfo {
    pub chain_id: u64,
    pub account: Pubkey,
    pub protocol_account: Pubkey,
    pub amm_account: Pubkey,
    pub tokens: Vec<Pubkey>
}

#[account]
#[derive(Debug)]
pub struct ExchangeInfo {
    pub amount_in: f64,
    pub amount_out: f64,
    pub minimum_amount_out: f64,
    pub out_price: f64,
    pub price_impact: f64,
    pub protocol_fee: f64,
    pub network_fee: f64
}

#[interface]
pub trait Client<'info, T: Accounts<'info>> {

    fn get_protocol_account(ctx: Context<T>) -> ProgramResult;

    fn get_exchange_info(
        ctx: Context<T>, 
        amount: f64, 
        slippage: f64

    ) -> ProgramResult;

    fn execute_swap(
        ctx: Context<T>,
        amount_in: f64,
        amount_out: f64,
        slippage: f64,
        fee_amount: f64

    ) -> ProgramResult;
}

#[interface]
pub trait LpClient<'info, T: Accounts<'info>> {

    fn get_pool_info(ctx: Context<T>) -> ProgramResult;
}
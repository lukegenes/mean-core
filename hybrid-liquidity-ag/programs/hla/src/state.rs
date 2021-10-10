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
    pub amount_in: u64,
    pub amount_out: u64,
    pub minimum_amount_out: u64,
    pub out_price: u64,
    pub price_impact: u64,
    pub protocol_fee: u64,
    pub network_fee: u64
}

#[account]
pub struct SwapInfo {
    pub owner: Pubkey,
    pub from_account: Pubkey,
    pub from_token: Pubkey,
    pub to_account: Pubkey,
    pub to_token: Pubkey,
    pub amount_in: u64,
    pub slippage: u8
}

pub trait Client<'info> {

    fn get_protocol_account(&self) -> Pubkey;

    fn get_exchange_info(
        &self,
        amount: f64, 
        slippage: f64

    ) -> ProgramResult;

    fn execute_swap(
        &self,
        amount_in: f64,
        amount_out: f64,
        slippage: f64,
        fee_amount: f64

    ) -> ProgramResult;
}

pub trait LpClient<'info> : Client<'info> {

    fn get_pool_info(&self) -> ProgramResult;

}
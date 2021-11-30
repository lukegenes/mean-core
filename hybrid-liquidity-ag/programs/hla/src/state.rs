use anchor_lang::prelude::*;

pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const RAYDIUM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const ORCA: &str = "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP";
pub const SABER: &str = "SSwpkEEcbUqx4vtoEByFjSkhKdCT862DNVb52nZg1UZ";
pub const MERCURIAL: &str = "MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky";
pub const SERUM: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";
pub const AGGREGATOR_PERCENT_FEE: f64 = 0.05;
pub const OPEN_ORDERS_LAYOUT_V2_LEN: u64 = 3228;

pub mod hla_ops_account {
    solana_program::declare_id!("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
}

#[derive(Clone, Debug)]
pub struct PoolInfo<'info> {
    pub account: &'info str,
    pub protocol_account: &'info str
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
    #[account(mut, signer)]
    pub vault_account: AccountInfo<'info>,
    pub from_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub from_token_account: AccountInfo<'info>,
    pub to_token_mint: AccountInfo<'info>,
    #[account(mut)]
    pub to_token_account: AccountInfo<'info>,
    #[account(mut, address = hla_ops_account::ID)]
    pub hla_ops_account: AccountInfo<'info>,
    #[account(mut)]
    pub hla_ops_token_account: AccountInfo<'info>,
    pub token_program_account: AccountInfo<'info>
}

#[derive(Clone)]
pub struct SwapInfo<'info> {
    pub accounts: Swap<'info>,
    pub remaining_accounts: Vec<AccountInfo<'info>>,
    pub from_amount: u64,
    pub min_out_amount: u64
}

#[derive(Accounts)]
pub struct SwapAccounts<'info> {
    pub market: MarketAccounts<'info>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub pc_wallet: AccountInfo<'info>,
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

impl<'info> From<&SwapAccounts<'info>> for OrderbookClient<'info> {
    fn from(accounts: &SwapAccounts<'info>) -> OrderbookClient<'info> {
        OrderbookClient {
            market: accounts.market.clone(),
            authority: accounts.authority.to_account_info(),
            pc_wallet: accounts.pc_wallet.to_account_info(),
            dex_program: accounts.dex_program.to_account_info(),
            token_program: accounts.token_program.to_account_info(),
            rent: accounts.rent.to_account_info(),
        }
    }
}

#[derive(Accounts, Clone)]
pub struct MarketAccounts<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub open_orders: AccountInfo<'info>,
    #[account(mut)]
    pub request_queue: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub order_payer_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub coin_vault: AccountInfo<'info>,
    #[account(mut)]
    pub pc_vault: AccountInfo<'info>,
    pub vault_signer: AccountInfo<'info>,
    #[account(mut)]
    pub coin_wallet: AccountInfo<'info>,
}

#[derive(Clone)]
pub struct OrderbookClient<'info> {
    pub market: MarketAccounts<'info>,
    pub authority: AccountInfo<'info>,
    pub pc_wallet: AccountInfo<'info>,
    pub dex_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub rent: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExchangeRate {
    pub rate: u64,
    pub from_decimals: u8,
    pub quote_decimals: u8,
    pub strict: bool,
}
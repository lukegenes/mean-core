use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("Unknown error")]
    Unknown,

    #[msg("HLA Operations account is not valid")]
    InvalidOpsAccount,    

    #[msg("Pool not found")]
    PoolNotFound,

    #[msg("Pool is not valid")]
    InvalidPool,    

    #[msg("Protocol is not valid")]
    InvalidProtocol,

    #[msg("Amm is not valid")]
    InvalidAmm,

    #[msg("Slippage exceeded")]
    SlippageExceeded,  
}
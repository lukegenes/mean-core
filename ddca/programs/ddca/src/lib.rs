use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, TokenAccount, Token, Transfer, CloseAccount };
use anchor_spl::associated_token::AssociatedToken;
use hybrid_liquidity_ag::cpi::accounts::Swap;

// Constants
pub mod ddca_operating_account {
    solana_program::declare_id!("3oSfkjQZKCneYvsCTZc9HViGAPqR8pYr4h9YeGB5ZxHf");
}

// hybrid liquidity aggregator program
pub mod hla_program {
    solana_program::declare_id!("EPa4WdYPcGGdwEbq425DMZukU2wDUE1RWAGrPbRYSLRE");
}
pub mod hla_ops_accountsss {
    solana_program::declare_id!("FZMd4pn9FsvMC55D4XQfaexJvKBtQpVuqMk5zuonLRDX");
}

// pub const CREATE_FLAT_LAMPORT_FEE: u64 = 10000;
// pub const ADD_FUNDS_PERCENT_TOKEN_FEE: f64 = 0.003;
// pub const CREATE_WITH_FUNDS_PERCENT_TOKEN_FEE: f64 = 0.003;
pub const WITHDRAW_PERCENT_TOKEN_FEE: f64 = 0.005;
// pub const STOP_FLAT_LAMPORT_FEE: u64 = 10000;
// pub const START_FLAT_LAMPORT_FEE: u64 = 10000;
pub const LAMPORTS_PER_SOL: u64 = 1000000000;
pub const SINGLE_SWAP_MINIMUM_LAMPORT_GAS_FEE: u64 = 20000000; //20 million

declare_id!("3nmm1awnyhABJdoA25MYVksxz1xnpUFeepJJyRTZfsyD");

#[program]
pub mod ddca {
    use super::*;

    // pub fn create<'a, 'b, 'c, 'info>(
    //     ctx: Context<'a, 'b, 'c, 'info, CreateInputAccounts<'info>>,
    pub fn create<'info>(
        ctx: Context<'_, '_, '_, 'info, CreateInputAccounts<'info>>,
        block_height: u64, 
        pda_bump: u8,
        from_initial_amount: u64,
        from_amount_per_swap: u64,
        interval_in_seconds: u64,
        first_swap_min_out_amount: u64,
        first_swap_slippage: u8,
    ) -> ProgramResult {


        let start_ts = Clock::get()?.unix_timestamp as u64;
        
        // for i in &block_height.to_be_bytes() {
        //     msg!("{}", i);
        // }

        // if ctx.remaining_accounts.len() == 0 {
        //     return Err(ProgramError::Custom(1)); // Arbitrary error. TODO: create proper error
        // }

        ctx.accounts.ddca_account.owner_acc_addr = *ctx.accounts.owner_account.key;
        ctx.accounts.ddca_account.from_mint = *ctx.accounts.from_mint.as_ref().key; //ctx.accounts.from_token_account.mint;
        ctx.accounts.ddca_account.from_tacc_addr =  *ctx.accounts.from_token_account.to_account_info().key; //*ctx.accounts.from_token_account.as_ref().key;
        ctx.accounts.ddca_account.to_mint = *ctx.accounts.to_mint.as_ref().key;
        ctx.accounts.ddca_account.to_tacc_addr =  *ctx.accounts.to_token_account.to_account_info().key;
        ctx.accounts.ddca_account.block_height = block_height;
        ctx.accounts.ddca_account.pda_bump = pda_bump;
        ctx.accounts.ddca_account.from_initial_amount = from_initial_amount;
        ctx.accounts.ddca_account.from_amount_per_swap = from_amount_per_swap;
        ctx.accounts.ddca_account.interval_in_seconds = interval_in_seconds;
        ctx.accounts.ddca_account.start_ts = start_ts;

        // if from_initial_amount == 0 {
        //     // transfer LAMPORT flat fees to the ddca operating account
        //     msg!("flat fees: transfering {} lamports from owner to operating account", CREATE_FLAT_LAMPORT_FEE);
        //     let ix = anchor_lang::solana_program::system_instruction::transfer(
        //         ctx.accounts.owner_account.key,
        //         ctx.accounts.operating_account.key,
        //         CREATE_FLAT_LAMPORT_FEE,
        //     );

        //     anchor_lang::solana_program::program::invoke(
        //         &ix,
        //         &[
        //             ctx.accounts.owner_account.to_account_info(),
        //             ctx.accounts.operating_account.to_account_info(),
        //         ],
        //     )?;

        //     return Ok(());
        // }

        if from_initial_amount % from_amount_per_swap != 0 {
            return Err(ErrorCode::InvalidAmounts.into());
        }
        
        let swap_count: u64 = (from_initial_amount / from_amount_per_swap) - 1; // -1: first swap will occur in the current transaction
        if swap_count == 0 {
            return Err(ErrorCode::InvalidSwapsCount.into());
        }

        // // transfer Token percentage fee to the ddca operating account
        // let from_mint_decimals = ctx.accounts.from_mint.decimals;
        // // msg!("from_mint_decimals: {}", from_mint_decimals);
        // let add_funds_fee = spl_token::ui_amount_to_amount(from_initial_amount as f64 * CREATE_WITH_FUNDS_PERCENT_TOKEN_FEE, from_mint_decimals);
        // msg!("'create with funds' fee: {}", add_funds_fee);
        // if add_funds_fee > 0 {
        //     token::transfer(
        //         ctx.accounts.into_transfer_fee_to_operating_context(),
        //         add_funds_fee,
        //     )?;
        // }

        // transfer enough SOL gas budget to the ddca account to pay future recurring swaps fees (network + amm fees)
        let recurring_lamport_fees = swap_count * SINGLE_SWAP_MINIMUM_LAMPORT_GAS_FEE;
        msg!("transfering {} lamports ({} SOL) from owner to ddca account for next {} swaps", recurring_lamport_fees, recurring_lamport_fees as f64 / LAMPORTS_PER_SOL as f64, swap_count);
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            ctx.accounts.owner_account.key,
            ctx.accounts.ddca_account.as_ref().key,
            recurring_lamport_fees,
        );

        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.owner_account.to_account_info(),
                ctx.accounts.ddca_account.to_account_info(),
            ],
        )?;

        // transfer Token initial amount to ddca 'from' token account // TODO: Enable later
        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            from_initial_amount,
        )?;

        // call hla to execute the first swap
        let hla_cpi_program = ctx.accounts.hla_program.clone();
        let hla_cpi_accounts = Swap {
            hla_ops_account: ctx.accounts.hla_operating_account.clone(),
            hla_ops_token_account: ctx.accounts.hla_operating_from_token_account.to_account_info().clone(),
            vault_account: ctx.accounts.ddca_account.to_account_info().clone(),
            from_token_account: ctx.accounts.from_token_account.to_account_info().clone(),
            from_token_mint: ctx.accounts.from_mint.to_account_info().clone(),
            to_token_account: ctx.accounts.to_token_account.to_account_info().clone(),
            to_token_mint: ctx.accounts.to_mint.to_account_info().clone(),
            token_program_account: ctx.accounts.token_program.to_account_info().clone(),
        };

        let seeds = &[
            ctx.accounts.owner_account.key.as_ref(),
            &ctx.accounts.ddca_account.block_height.to_be_bytes(),
            b"ddca-seed",
            &[ctx.accounts.ddca_account.pda_bump],
        ];

        let seeds_sign = &[&seeds[..]];

        let hla_cpi_ctx = CpiContext::new(hla_cpi_program, hla_cpi_accounts)
        .with_signer(seeds_sign)
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        hybrid_liquidity_ag::cpi::swap(hla_cpi_ctx, from_amount_per_swap, first_swap_min_out_amount, first_swap_slippage);

        
        Ok(())
    }
}

// DERIVE ACCOUNTS

#[derive(Accounts, Clone)]
#[instruction(
    block_height: u64, 
    pda_bump: u8,
    from_initial_amount: u64,
    from_amount_per_swap: u64,
    )]
pub struct CreateInputAccounts<'info> {
    // owner
    #[account(mut)]
    pub owner_account: Signer<'info>,
    #[account(
        mut,
        constraint = owner_from_token_account.amount >= from_initial_amount  // TODO: enable later when I have enough balance
    )]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    // ddca
    #[account(
        init, 
        seeds = [
            owner_account.key().as_ref(), 
            &block_height.to_be_bytes(), 
            b"ddca-seed"
            ],
        bump = pda_bump,
        payer = owner_account, 
        space = 8 + DdcaAccount::LEN,
        constraint = from_amount_per_swap > 0,
    )]
    pub ddca_account: Account<'info, DdcaAccount>,
    pub from_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = from_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(constraint = from_mint.key() != to_mint.key())]
    pub to_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = to_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub to_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = ddca_operating_account::ID)]
    pub operating_account: AccountInfo<'info>,
    #[account(
        mut,
        //TODO: uncomment when https://github.com/project-serum/anchor/pull/843 is released
        // associated_token::mint = from_mint, 
        // associated_token::authority = ddca_operating_account, 
    )]
    pub operating_from_token_account: Box<Account<'info, TokenAccount>>,
    // Hybrid Liquidity Aggregator
    #[account(address = hla_program::ID)]
    pub hla_program: AccountInfo<'info>,
    #[account(mut, address = hla_ops_accountsss::ID)]
    pub hla_operating_account: AccountInfo<'info>,
    #[account(mut)]
    pub hla_operating_from_token_account: Box<Account<'info, TokenAccount>>,
    // system and spl
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// ACCOUNT STRUCTS

#[account]
pub struct DdcaAccount {
    pub owner_acc_addr: Pubkey, //32 bytes
    pub from_mint: Pubkey, //32 bytes
    pub from_tacc_addr: Pubkey, //32 bytes
    pub to_mint: Pubkey, //32 bytes
    pub to_tacc_addr: Pubkey, //32 bytes
    pub block_height: u64, //8 bytes
    pub pda_bump: u8, //1 byte
    pub from_initial_amount: u64, //8 bytes
    pub from_amount_per_swap: u64, //8 bytes
    pub start_ts: u64, //8 bytes
    pub interval_in_seconds: u64, //8 bytes
    pub last_completed_swap_ts: u64, //8 bytes
}

impl DdcaAccount {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 8 + 8 + 8 + 8;
}

//UTILS IMPL

impl<'info> CreateInputAccounts<'info> {
    fn into_transfer_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_from_token_account.to_account_info().clone(),
            to: self
                .from_token_account
                .to_account_info()
                .clone(),
            authority: self.owner_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }

    fn into_transfer_fee_to_operating_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.owner_from_token_account.to_account_info().clone(),
            to: self
                .operating_from_token_account
                .to_account_info()
                .clone(),
            authority: self.owner_account.to_account_info().clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Deposit amount must be a multiple of the amount per swap")]
    InvalidAmounts,
    #[msg("The number of recurring swaps must be greater than 1")]
    InvalidSwapsCount,
}

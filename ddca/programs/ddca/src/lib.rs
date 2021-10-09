use anchor_lang::prelude::*;
use anchor_spl::token::{ self, Mint, TokenAccount, Token, Transfer, CloseAccount };
use anchor_spl::associated_token::AssociatedToken;

declare_id!("3nmm1awnyhABJdoA25MYVksxz1xnpUFeepJJyRTZfsyD");

#[program]
pub mod ddca {
    use super::*;

    pub fn create_ddca(
        ctx: Context<CreateInputAccounts>,
        block_height: u64, 
        pda_bump: u8,
        from_initial_amount: u64,
        from_amount_per_swap: u64,
        interval_in_seconds: u64,
        // last_completed_swap_ts: u64,
    ) -> ProgramResult {

        // for i in &block_height.to_be_bytes() {
        //     msg!("{}", i);
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
        // ctx.accounts.ddca_account.last_completed_swap_ts = last_completed_swap_ts;

        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            from_initial_amount,
        )?;

        Ok(())
    }
}

// DERIVE ACCOUNTS

#[derive(Accounts)]
#[instruction(
    block_height: u64, 
    pda_bump: u8,
    from_initial_amount: u64,
    from_amount_per_swap: u64,
    // interval_in_seconds: u64,
    // last_completed_swap_ts: u64,
    )]
pub struct CreateInputAccounts<'info> {
    pub owner_account: Signer<'info>,
    #[account(
        mut,
        constraint = owner_from_token_account.amount >= from_initial_amount
    )]
    pub owner_from_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init, 
        seeds = [
            owner_account.key().as_ref(), 
            &block_height.to_be_bytes(), 
            b"ddca-seed"
            ],
        bump = pda_bump,
        payer = owner_account, 
        space = 8 + DdcaAccount::LEN)]
    pub ddca_account: Account<'info, DdcaAccount>,
    pub from_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = from_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub from_token_account: Account<'info, TokenAccount>,
    pub to_mint:  Account<'info, Mint>, 
    #[account(
        init, 
        associated_token::mint = to_mint, 
        associated_token::authority = ddca_account, 
        payer = owner_account)]
    pub to_token_account: Account<'info, TokenAccount>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: AccountInfo<'info>,
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
    // pub frequency: u8, //TODO
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
}

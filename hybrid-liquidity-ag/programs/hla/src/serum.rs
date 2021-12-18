// use anchor_lang::prelude::*;
// use anchor_spl::dex;
// use anchor_spl::token::*;
// use solana_program::{system_instruction, program::invoke_signed, pubkey::Pubkey};
// use std::num::NonZeroU64;
// use crate::errors::*;
// use crate::utils::*;
// use crate::state::{
//     SwapInfo, 
//     SwapAccounts, 
//     MarketAccounts, 
//     OrderbookClient, 
//     Side, 
//     AGGREGATOR_PERCENT_FEE, 
//     OPEN_ORDERS_LAYOUT_V2_LEN
// };

// use anchor_spl::dex::serum_dex::{
//     instruction::SelfTradeBehavior,
//     matching::{OrderType, Side as SerumSide},
//     state::{MarketState}
// };

// pub fn swap<'info>(
//     swap_info: SwapInfo<'info>

// ) -> ProgramResult {

//     let acounts_iter = &mut swap_info.remaining_accounts.iter();
    
//     let market_account_info = next_account_info(acounts_iter)?;
//     let cpi_program_info = next_account_info(acounts_iter)?;
//     let open_orders_account_info = next_account_info(acounts_iter)?;
//     let request_queue_account_info = next_account_info(acounts_iter)?;
//     let event_queue_account_info = next_account_info(acounts_iter)?;
//     let bids_account_info = next_account_info(acounts_iter)?;
//     let asks_account_info = next_account_info(acounts_iter)?;
//     let coin_vault_account_info = next_account_info(acounts_iter)?;
//     let pc_vault_account_info = next_account_info(acounts_iter)?;    
//     let vault_signer_info = next_account_info(acounts_iter)?;
//     let _hla_account_info = next_account_info(acounts_iter)?;
//     let system_account_info = next_account_info(acounts_iter)?;
//     let rent_info = next_account_info(acounts_iter)?;
//     let rent = &Rent::from_account_info(rent_info)?;

//     // create open orders account
//     let (open_orders_address, nounce) = Pubkey::find_program_address(
//         &[market_account_info.key.as_ref()],
//         cpi_program_info.key
//     );

//     if open_orders_address.ne(open_orders_account_info.key)
//     {
//         return Err(ErrorCode::Unknown.into());
//     }

//     let open_orders_account_balance = rent.minimum_balance(OPEN_ORDERS_LAYOUT_V2_LEN as usize);
//     let create_open_orders_account_ix = system_instruction::create_account(
//         swap_info.accounts.vault_account.key,
//         open_orders_account_info.key,
//         open_orders_account_balance,
//         OPEN_ORDERS_LAYOUT_V2_LEN,
//         cpi_program_info.key
//     );

//     let _create_open_orders = invoke_signed(
//         &create_open_orders_account_ix, 
//         &[
//             swap_info.accounts.vault_account.to_account_info(),
//             open_orders_account_info.to_account_info(),
//             cpi_program_info.to_account_info(),
//             system_account_info.to_account_info()
//         ],
//         &[&[open_orders_account_info.key.as_ref(), &[nounce]]]
//     );

//     // get swap context
//     // let swap_ctx = get_swap_context(swap_info.clone())?;
//     let cpi_accounts = SwapAccounts {
//         market: MarketAccounts {
//             market: market_account_info.to_account_info(),
//             open_orders: open_orders_account_info.to_account_info(),
//             request_queue: request_queue_account_info.to_account_info(),
//             event_queue: event_queue_account_info.to_account_info(),
//             bids: bids_account_info.to_account_info(),
//             asks: asks_account_info.to_account_info(),
//             order_payer_token_account: swap_info.accounts.from_token_account.to_account_info(),
//             coin_vault: coin_vault_account_info.to_account_info(),
//             pc_vault: pc_vault_account_info.to_account_info(),
//             vault_signer: vault_signer_info.to_account_info(),
//             coin_wallet: swap_info.accounts.from_token_account.to_account_info()
//         },
//         authority: swap_info.accounts.vault_account.to_account_info(),
//         pc_wallet: swap_info.accounts.from_token_account.to_account_info(),
//         dex_program: cpi_program_info.to_account_info(),
//         token_program: swap_info.accounts.token_program_account.to_account_info(),
//         rent: rent_info.to_account_info()
//     };

//     let swap_ctx = CpiContext::new(
//         cpi_program_info.to_account_info(), 
//         cpi_accounts
//     );

//     let mut side = Side::Bid;

//     if swap_info.accounts.from_token_account.key.eq(coin_vault_account_info.key) &&
//        swap_info.accounts.to_token_account.key.eq(pc_vault_account_info.key) 
//     {
//         side = Side::Ask;
//     }

//     // get fees and swap amount
//     let fee_amount = (swap_info.from_amount as f64) * AGGREGATOR_PERCENT_FEE / 100f64;
//     let swap_amount = (swap_info.from_amount as f64) - fee_amount;

//     match side {
//         Side::Bid => (&swap_ctx.accounts.pc_wallet, &swap_ctx.accounts.market.coin_wallet),
//         Side::Ask => (&swap_ctx.accounts.market.coin_wallet, &swap_ctx.accounts.pc_wallet),
//     };

//     let orderbook: OrderbookClient<'info> = (&swap_ctx.accounts).into();

//     match side {
//         Side::Bid => orderbook.buy(swap_amount as u64, None)?,
//         Side::Ask => orderbook.sell(swap_amount as u64, None)?,
//     };

//     let _settle = orderbook.settle(None);
//     let transfer_ctx = get_transfer_context(swap_info.clone())?;

//     transfer(
//         transfer_ctx,
//         fee_amount as u64
//     )?;

//     Ok(())
// }

// // fn get_swap_context<'info>(
// //     swap_info: SwapInfo<'info>

// // ) -> Result<CpiContext<'_, '_, '_, 'info, SwapAccounts<'info>>> {

// //     let acounts_iter = &mut swap_info.remaining_accounts.iter();
    
// //     let market_account_info = next_account_info(acounts_iter)?;
// //     let cpi_program_info = next_account_info(acounts_iter)?;
// //     let open_orders_account_info = next_account_info(acounts_iter)?;
// //     let request_queue_account_info = next_account_info(acounts_iter)?;
// //     let event_queue_account_info = next_account_info(acounts_iter)?;
// //     let bids_account_info = next_account_info(acounts_iter)?;
// //     let asks_account_info = next_account_info(acounts_iter)?;
// //     let coin_vault_account_info = next_account_info(acounts_iter)?;
// //     let pc_vault_account_info = next_account_info(acounts_iter)?;    
// //     let vault_signer_info = next_account_info(acounts_iter)?;
// //     let _system_account_info = next_account_info(acounts_iter)?;
// //     let rent_info = next_account_info(acounts_iter)?;

// //     let cpi_accounts = SwapAccounts {
// //         market: MarketAccounts {
// //             market: market_account_info.to_account_info(),
// //             open_orders: open_orders_account_info.to_account_info(),
// //             request_queue: request_queue_account_info.to_account_info(),
// //             event_queue: event_queue_account_info.to_account_info(),
// //             bids: bids_account_info.to_account_info(),
// //             asks: asks_account_info.to_account_info(),
// //             order_payer_token_account: swap_info.accounts.from_token_account.to_account_info(),
// //             coin_vault: coin_vault_account_info.to_account_info(),
// //             pc_vault: pc_vault_account_info.to_account_info(),
// //             vault_signer: vault_signer_info.to_account_info(),
// //             coin_wallet: swap_info.accounts.from_token_account.to_account_info()
// //         },
// //         authority: swap_info.accounts.vault_account.to_account_info(),
// //         pc_wallet: swap_info.accounts.from_token_account.to_account_info(),
// //         dex_program: cpi_program_info.to_account_info(),
// //         token_program: swap_info.accounts.token_program_account.to_account_info(),
// //         rent: rent_info.to_account_info()
// //     };

// //     Ok(CpiContext::new(
// //         cpi_program_info.to_account_info(), 
// //         cpi_accounts
// //     ))
// // }

// impl<'info> OrderbookClient<'info> {

//     fn sell(
//         &self,
//         base_amount: u64,
//         srm_msrm_discount: Option<AccountInfo<'info>>

//     ) -> ProgramResult {

//         let limit_price = 1;
//         let max_native_pc_qty = u64::MAX;
//         let max_coin_qty = {
//             let market = MarketState::load(&self.market.market, &dex::ID)?;
//             base_amount.checked_div(market.coin_lot_size).unwrap()
//         };

//         let client_order_id = 0;
//         let limit = 65535;
//         let mut ctx = CpiContext::new(
//             self.dex_program.to_account_info(), 
//             self.clone().into()
//         );

//         if let Some(srm_msrm_discount) = srm_msrm_discount {
//             ctx = ctx.with_remaining_accounts(vec![srm_msrm_discount]);
//         }

//         dex::new_order_v3(
//             ctx,
//             Side::Ask.into(),
//             NonZeroU64::new(limit_price).unwrap(),
//             NonZeroU64::new(max_coin_qty).unwrap(),
//             NonZeroU64::new(max_native_pc_qty).unwrap(),
//             SelfTradeBehavior::DecrementTake,
//             OrderType::ImmediateOrCancel,
//             client_order_id,
//             limit,
//         )
//     }

//     fn buy(
//         &self,
//         quote_amount: u64,
//         srm_msrm_discount: Option<AccountInfo<'info>>

//     ) -> ProgramResult {

//         let limit_price = u64::MAX;
//         let max_coin_qty = u64::MAX;
//         let max_native_pc_qty = quote_amount;
//         let client_order_id = 0;
//         let limit = 65535;
//         let mut ctx = CpiContext::new(
//             self.dex_program.to_account_info(), 
//             self.clone().into()
//         );

//         if let Some(srm_msrm_discount) = srm_msrm_discount {
//             ctx = ctx.with_remaining_accounts(vec![srm_msrm_discount]);
//         }

//         dex::new_order_v3(
//             ctx,
//             Side::Bid.into(),
//             NonZeroU64::new(limit_price).unwrap(),
//             NonZeroU64::new(max_coin_qty).unwrap(),
//             NonZeroU64::new(max_native_pc_qty).unwrap(),
//             SelfTradeBehavior::DecrementTake,
//             OrderType::ImmediateOrCancel,
//             client_order_id,
//             limit,
//         )
//     }

//     fn settle(&self, referral: Option<AccountInfo<'info>>) -> ProgramResult {

//         let settle_accs = dex::SettleFunds {
//             market: self.market.market.to_account_info(),
//             open_orders: self.market.open_orders.to_account_info(),
//             open_orders_authority: self.authority.to_account_info(),
//             coin_vault: self.market.coin_vault.to_account_info(),
//             pc_vault: self.market.pc_vault.to_account_info(),
//             coin_wallet: self.market.coin_wallet.to_account_info(),
//             pc_wallet: self.pc_wallet.to_account_info(),
//             vault_signer: self.market.vault_signer.to_account_info(),
//             token_program: self.token_program.to_account_info(),
//         };

//         let mut ctx = CpiContext::new(
//             self.dex_program.to_account_info(), 
//             settle_accs
//         );

//         if let Some(referral) = referral {
//             ctx = ctx.with_remaining_accounts(vec![referral]);
//         }

//         dex::settle_funds(ctx)
//     }
// }

// impl<'info> From<OrderbookClient<'info>> for dex::NewOrderV3<'info> {
//     fn from(c: OrderbookClient<'info>) -> dex::NewOrderV3<'info> {
//         dex::NewOrderV3 {
//             market: c.market.market.to_account_info(),
//             open_orders: c.market.open_orders.to_account_info(),
//             request_queue: c.market.request_queue.to_account_info(),
//             event_queue: c.market.event_queue.to_account_info(),
//             market_bids: c.market.bids.to_account_info(),
//             market_asks: c.market.asks.to_account_info(),
//             order_payer_token_account: c.market.order_payer_token_account.to_account_info(),
//             open_orders_authority: c.authority.to_account_info(),
//             coin_vault: c.market.coin_vault.to_account_info(),
//             pc_vault: c.market.pc_vault.to_account_info(),
//             token_program: c.token_program.to_account_info(),
//             rent: c.rent.to_account_info(),
//         }
//     }
// }

// impl From<Side> for SerumSide {
//     fn from(side: Side) -> SerumSide {
//         match side {
//             Side::Bid => SerumSide::Bid,
//             Side::Ask => SerumSide::Ask,
//         }
//     }
// }
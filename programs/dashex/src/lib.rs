use anchor_lang::prelude::*;

declare_id!("9XLPuTNvnnnmj6BM1ASrCZ92TvjuhggTmm4dmkm97Zcv");

pub mod constants;
pub mod error;
pub mod instructions;
pub mod math;
pub mod state;

use instructions::*;

#[program]
pub mod dashex {
    use super::*;

    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        instructions::initialize_protocol::handler(ctx)
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_index: u16,
        oracle: Pubkey,
        base_asset_reserve: u64,
        quote_asset_reserve: u64,
        peg_multiplier: u64,
        initial_margin_ratio: u32,
        maintenance_margin_ratio: u32,
    ) -> Result<()> {
        instructions::create_market::handler(
            ctx,
            market_index,
            oracle,
            base_asset_reserve,
            quote_asset_reserve,
            peg_multiplier,
            initial_margin_ratio,
            maintenance_margin_ratio,
        )
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
        instructions::deposit_collateral::handler(ctx, amount)
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
        instructions::withdraw_collateral::handler(ctx, amount)
    }

    pub fn open_position(ctx: Context<TradePosition>, base_delta: i64, limit_price: u64) -> Result<()> {
        instructions::open_handler(ctx, base_delta, limit_price)
    }

    pub fn close_position(ctx: Context<TradePosition>, base_amount: u64, limit_price: u64) -> Result<()> {
        instructions::close_handler(ctx, base_amount, limit_price)
    }
}

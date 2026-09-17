use anchor_lang::prelude::*;

use crate::{
    constants::{MIN_RESERVE, RATIO_PRECISION},
    error::DashExError,
    state::{GlobalState, Market, MarketStatus},
};

#[derive(Accounts)]
#[instruction(market_index: u16)]
pub struct CreateMarket<'info> {
    #[account(mut, has_one = admin @ DashExError::Unauthorized)]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init,
        payer = admin,
        space = 8 + Market::INIT_SPACE,
        seeds = [b"market".as_ref(), market_index.to_le_bytes().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateMarket>,
    market_index: u16,
    oracle: Pubkey,
    base_asset_reserve: u64,
    quote_asset_reserve: u64,
    peg_multiplier: u64,
    initial_margin_ratio: u32,
    maintenance_margin_ratio: u32,
) -> Result<()> {
    require!(base_asset_reserve >= MIN_RESERVE, DashExError::InvalidMarketConfiguration);
    require!(quote_asset_reserve >= MIN_RESERVE, DashExError::InvalidMarketConfiguration);
    require!(peg_multiplier > 0, DashExError::InvalidMarketConfiguration);
    require!(initial_margin_ratio > 0, DashExError::InvalidMarketConfiguration);
    require!(maintenance_margin_ratio <= initial_margin_ratio, DashExError::InvalidMarketConfiguration);
    require!(initial_margin_ratio as u128 <= RATIO_PRECISION, DashExError::InvalidMarketConfiguration);

    let product = (base_asset_reserve as u128)
        .checked_mul(quote_asset_reserve as u128)
        .ok_or(error!(DashExError::MathOverflow))?;
    let sqrt_k = integer_sqrt(product);
    require!((sqrt_k as u128) * (sqrt_k as u128) == product, DashExError::InvalidMarketConfiguration);

    let market = &mut ctx.accounts.market;
    market.version = 0;
    market.market_index = market_index;
    market.status = MarketStatus::Active;
    market.oracle = oracle;
    market.base_asset_reserve = base_asset_reserve;
    market.quote_asset_reserve = quote_asset_reserve;
    market.sqrt_k = sqrt_k;
    market.peg_multiplier = peg_multiplier;
    market.initial_margin_ratio = initial_margin_ratio;
    market.maintenance_margin_ratio = maintenance_margin_ratio;
    market.open_interest_long = 0;
    market.open_interest_short = 0;
    market.bump = ctx.bumps.market;
    ctx.accounts.global_state.market_count = ctx.accounts.global_state.market_count.checked_add(1).ok_or(error!(DashExError::MathOverflow))?;
    Ok(())
}

fn integer_sqrt(value: u128) -> u64 {
    let mut low = 0_u128;
    let mut high = u64::MAX as u128;
    while low < high {
        let mid = (low + high + 1) / 2;
        if mid <= value / mid {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    low as u64
}

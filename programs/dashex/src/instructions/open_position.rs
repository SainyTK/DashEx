use anchor_lang::prelude::*;

use crate::{
    error::DashExError,
    math::{execute_trade, margin_requirement, mark_price, unrealized_pnl},
    state::{Market, MarketStatus, UserAccount},
};

#[derive(Accounts)]
pub struct TradePosition<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner @ DashExError::InvalidUserAccountOwner
    )]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

pub fn open_handler(ctx: Context<TradePosition>, base_delta: i64, limit_price: u64) -> Result<()> {
    require!(ctx.accounts.market.status == MarketStatus::Active, DashExError::MarketUnavailable);
    execute_and_check(&mut ctx.accounts.market, &mut ctx.accounts.user_account, base_delta, limit_price, true)
}

pub fn close_handler(ctx: Context<TradePosition>, base_amount: u64, limit_price: u64) -> Result<()> {
    require!(base_amount > 0, DashExError::InvalidAmount);
    require!(ctx.accounts.market.status != MarketStatus::Paused, DashExError::MarketUnavailable);
    let position = &ctx.accounts.user_account.positions[0];
    require!(position.is_open, DashExError::CloseAmountExceedsPosition);
    require!(base_amount <= position.base_asset_amount.unsigned_abs(), DashExError::CloseAmountExceedsPosition);
    let base_delta = if position.base_asset_amount > 0 {
        -(base_amount as i64)
    } else {
        base_amount as i64
    };
    execute_and_check(&mut ctx.accounts.market, &mut ctx.accounts.user_account, base_delta, limit_price, false)
}

fn execute_and_check(
    market: &mut Account<Market>,
    user: &mut Account<UserAccount>,
    base_delta: i64,
    limit_price: u64,
    require_initial_margin: bool,
) -> Result<()> {
    let trade = execute_trade(
        market.base_asset_reserve,
        market.quote_asset_reserve,
        market.sqrt_k,
        market.peg_multiplier,
        base_delta,
    )?;
    if base_delta > 0 {
        require!(trade.execution_price <= limit_price, DashExError::LimitPriceExceeded);
    } else {
        require!(trade.execution_price >= limit_price, DashExError::LimitPriceExceeded);
    }

    let (old_base, new_base, new_quote, remains_open) = {
        let position = &mut user.positions[0];
        require!(!position.is_open || position.market_index == market.market_index, DashExError::InvalidAmount);
        let old_base = position.base_asset_amount;
        let new_base = old_base.checked_add(base_delta).ok_or(error!(DashExError::MathOverflow))?;
        let new_quote = position.quote_asset_amount.checked_add(trade.user_quote_delta).ok_or(error!(DashExError::MathOverflow))?;
        let remains_open = new_base != 0;
        position.market_index = market.market_index;
        position.base_asset_amount = new_base;
        position.quote_asset_amount = if remains_open { new_quote } else { 0 };
        position.is_open = remains_open;
        (old_base, new_base, new_quote, remains_open)
    };

    market.base_asset_reserve = trade.new_base_reserve;
    market.quote_asset_reserve = trade.new_quote_reserve;
    update_open_interest(market, old_base, new_base)?;

    if !remains_open {
        user.collateral = user.collateral.checked_add(new_quote).ok_or(error!(DashExError::MathOverflow))?;
    }

    if require_initial_margin {
        let mark = mark_price(market.base_asset_reserve, market.quote_asset_reserve, market.peg_multiplier)?;
        let equity = user.collateral.checked_add(unrealized_pnl(new_base, if remains_open { new_quote } else { 0 }, mark)?).ok_or(error!(DashExError::MathOverflow))?;
        let requirement = margin_requirement(new_base, mark, market.initial_margin_ratio)?;
        require!(equity >= requirement, DashExError::InsufficientFreeCollateral);
    }
    Ok(())
}

fn update_open_interest(market: &mut Market, old_base: i64, new_base: i64) -> Result<()> {
    let old_abs = old_base.unsigned_abs();
    let new_abs = new_base.unsigned_abs();
    if old_base > 0 {
        market.open_interest_long = market.open_interest_long.checked_sub(old_abs).ok_or(error!(DashExError::MathOverflow))?;
    } else if old_base < 0 {
        market.open_interest_short = market.open_interest_short.checked_sub(old_abs).ok_or(error!(DashExError::MathOverflow))?;
    }
    if new_base > 0 {
        market.open_interest_long = market.open_interest_long.checked_add(new_abs).ok_or(error!(DashExError::MathOverflow))?;
    } else if new_base < 0 {
        market.open_interest_short = market.open_interest_short.checked_add(new_abs).ok_or(error!(DashExError::MathOverflow))?;
    }
    Ok(())
}

use anchor_lang::prelude::*;

use crate::{
    constants::{BASE_PRECISION, MIN_RESERVE, RATIO_PRECISION},
    error::DashExError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AmmTrade {
    pub new_base_reserve: u64,
    pub new_quote_reserve: u64,
    pub user_quote_delta: i64,
    pub execution_price: u64,
}

pub fn ceil_div(numerator: u128, denominator: u128) -> Result<u128> {
    require!(denominator != 0, DashExError::MathOverflow);
    numerator
        .checked_add(denominator - 1)
        .ok_or(error!(DashExError::MathOverflow))?
        .checked_div(denominator)
        .ok_or(error!(DashExError::MathOverflow))
}

pub fn mark_price(base_reserve: u64, quote_reserve: u64, peg_multiplier: u64) -> Result<u64> {
    require!(base_reserve >= MIN_RESERVE, DashExError::InvalidReserve);
    let price = (quote_reserve as u128)
        .checked_mul(peg_multiplier as u128)
        .ok_or(error!(DashExError::MathOverflow))?
        .checked_div(base_reserve as u128)
        .ok_or(error!(DashExError::MathOverflow))?;
    u64::try_from(price).map_err(|_| error!(DashExError::MathOverflow))
}

pub fn execute_trade(
    base_reserve: u64,
    quote_reserve: u64,
    sqrt_k: u64,
    peg_multiplier: u64,
    base_delta: i64,
) -> Result<AmmTrade> {
    require!(base_delta != 0, DashExError::InvalidAmount);
    let new_base = (base_reserve as i128)
        .checked_sub(base_delta as i128)
        .ok_or(error!(DashExError::MathOverflow))?;
    require!(new_base >= MIN_RESERVE as i128, DashExError::InvalidReserve);
    let new_base_reserve = u64::try_from(new_base).map_err(|_| error!(DashExError::MathOverflow))?;
    let k = (sqrt_k as u128)
        .checked_mul(sqrt_k as u128)
        .ok_or(error!(DashExError::MathOverflow))?;
    let new_quote_reserve = u64::try_from(ceil_div(k, new_base_reserve as u128)?)
        .map_err(|_| error!(DashExError::MathOverflow))?;
    let reserve_delta = new_quote_reserve.abs_diff(quote_reserve) as u128;
    let quote_delta = ceil_div(
        reserve_delta
            .checked_mul(peg_multiplier as u128)
            .ok_or(error!(DashExError::MathOverflow))?,
        BASE_PRECISION,
    )?;
    let quote_delta = i64::try_from(quote_delta).map_err(|_| error!(DashExError::MathOverflow))?;
    let user_quote_delta = if base_delta > 0 { -quote_delta } else { quote_delta };
    let base_abs = (base_delta as i128).unsigned_abs();
    let execution_price = u64::try_from(ceil_div((quote_delta as u128) * BASE_PRECISION, base_abs)?)
        .map_err(|_| error!(DashExError::MathOverflow))?;

    Ok(AmmTrade {
        new_base_reserve,
        new_quote_reserve,
        user_quote_delta,
        execution_price,
    })
}

pub fn unrealized_pnl(base_asset_amount: i64, quote_asset_amount: i64, mark_price: u64) -> Result<i64> {
    let mark_value = (base_asset_amount as i128)
        .checked_mul(mark_price as i128)
        .ok_or(error!(DashExError::MathOverflow))?
        .checked_div(BASE_PRECISION as i128)
        .ok_or(error!(DashExError::MathOverflow))?;
    let pnl = (quote_asset_amount as i128)
        .checked_add(mark_value)
        .ok_or(error!(DashExError::MathOverflow))?;
    i64::try_from(pnl).map_err(|_| error!(DashExError::MathOverflow))
}

pub fn margin_requirement(base_asset_amount: i64, mark_price: u64, margin_ratio: u32) -> Result<i64> {
    let notional = ceil_div(
        (base_asset_amount as i128).unsigned_abs() * mark_price as u128,
        BASE_PRECISION,
    )?;
    let requirement = ceil_div(notional * margin_ratio as u128, RATIO_PRECISION)?;
    i64::try_from(requirement).map_err(|_| error!(DashExError::MathOverflow))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESERVE: u64 = 1_000_000_000_000;
    const PEG: u64 = 100_000_000;

    #[test]
    fn long_moves_mark_up_and_debits_quote() {
        let before = mark_price(RESERVE, RESERVE, PEG).unwrap();
        let trade = execute_trade(RESERVE, RESERVE, RESERVE, PEG, 1_000_000_000).unwrap();
        let after = mark_price(trade.new_base_reserve, trade.new_quote_reserve, PEG).unwrap();
        assert!(after > before);
        assert!(trade.user_quote_delta < 0);
    }

    #[test]
    fn short_moves_mark_down_and_credits_quote() {
        let before = mark_price(RESERVE, RESERVE, PEG).unwrap();
        let trade = execute_trade(RESERVE, RESERVE, RESERVE, PEG, -1_000_000_000).unwrap();
        let after = mark_price(trade.new_base_reserve, trade.new_quote_reserve, PEG).unwrap();
        assert!(after < before);
        assert!(trade.user_quote_delta > 0);
    }

    #[test]
    fn pnl_has_correct_sign_for_both_directions() {
        assert_eq!(unrealized_pnl(1_000_000_000, -100_000_000, 110_000_000).unwrap(), 10_000_000);
        assert_eq!(unrealized_pnl(-1_000_000_000, 100_000_000, 110_000_000).unwrap(), -10_000_000);
    }

    #[test]
    fn margin_rounds_up() {
        assert_eq!(margin_requirement(1, 1, 1).unwrap(), 1);
    }
}

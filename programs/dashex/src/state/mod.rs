use anchor_lang::prelude::*;

use crate::constants::MAX_POSITIONS;

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub version: u16,
    pub admin: Pubkey,
    pub collateral_mint: Pubkey,
    pub collateral_vault: Pubkey,
    pub insurance_vault: Pubkey,
    pub market_count: u16,
    pub paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Market {
    pub version: u16,
    pub market_index: u16,
    pub status: MarketStatus,
    pub oracle: Pubkey,
    pub base_asset_reserve: u64,
    pub quote_asset_reserve: u64,
    pub sqrt_k: u64,
    pub peg_multiplier: u64,
    pub initial_margin_ratio: u32,
    pub maintenance_margin_ratio: u32,
    pub open_interest_long: u64,
    pub open_interest_short: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub version: u16,
    pub owner: Pubkey,
    pub collateral: i64,
    pub positions: [Position; MAX_POSITIONS],
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, InitSpace)]
pub struct Position {
    pub market_index: u16,
    pub base_asset_amount: i64,
    pub quote_asset_amount: i64,
    pub last_cumulative_funding_rate: i64,
    pub is_open: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace, PartialEq, Eq)]
pub enum MarketStatus {
    Active,
    ReduceOnly,
    Paused,
    ClosedMarket,
}

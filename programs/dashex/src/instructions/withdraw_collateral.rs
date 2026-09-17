use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{
    error::DashExError,
    math::{margin_requirement, mark_price, unrealized_pnl},
    state::{GlobalState, Market, UserAccount},
};

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(seeds = [b"global"], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,
    pub market: Account<'info, Market>,
    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner @ DashExError::InvalidUserAccountOwner
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, address = global_state.collateral_vault)]
    pub collateral_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_collateral.owner == owner.key(), constraint = user_collateral.mint == global_state.collateral_mint)]
    pub user_collateral: Account<'info, TokenAccount>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<WithdrawCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, DashExError::InvalidAmount);
    let amount_i64 = i64::try_from(amount).map_err(|_| error!(DashExError::MathOverflow))?;
    let user = &mut ctx.accounts.user_account;
    user.collateral = user.collateral.checked_sub(amount_i64).ok_or(error!(DashExError::MathOverflow))?;

    let position = user.positions[0];
    if position.is_open {
        require!(position.market_index == ctx.accounts.market.market_index, DashExError::InvalidAmount);
        let mark = mark_price(
            ctx.accounts.market.base_asset_reserve,
            ctx.accounts.market.quote_asset_reserve,
            ctx.accounts.market.peg_multiplier,
        )?;
        let equity = user.collateral.checked_add(unrealized_pnl(position.base_asset_amount, position.quote_asset_amount, mark)?).ok_or(error!(DashExError::MathOverflow))?;
        let requirement = margin_requirement(position.base_asset_amount, mark, ctx.accounts.market.initial_margin_ratio)?;
        require!(equity >= requirement, DashExError::InsufficientFreeCollateral);
    } else {
        require!(user.collateral >= 0, DashExError::InsufficientFreeCollateral);
    }

    let signer_seeds: &[&[u8]] = &[b"global", &[ctx.accounts.global_state.bump]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.collateral_vault.to_account_info(),
                to: ctx.accounts.user_collateral.to_account_info(),
                authority: ctx.accounts.global_state.to_account_info(),
            },
            &[signer_seeds],
        ),
        amount,
    )
}

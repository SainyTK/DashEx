use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::{error::DashExError, state::{GlobalState, UserAccount}};

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + UserAccount::INIT_SPACE,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, constraint = user_collateral.owner == owner.key(), constraint = user_collateral.mint == global_state.collateral_mint)]
    pub user_collateral: Account<'info, TokenAccount>,
    #[account(mut, address = global_state.collateral_vault, constraint = collateral_vault.mint == global_state.collateral_mint)]
    pub collateral_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<DepositCollateral>, amount: u64) -> Result<()> {
    require!(amount > 0, DashExError::InvalidAmount);
    let user = &mut ctx.accounts.user_account;
    require!(user.owner == Pubkey::default() || user.owner == ctx.accounts.owner.key(), DashExError::InvalidUserAccountOwner);
    if user.owner == Pubkey::default() {
        user.version = 0;
        user.owner = ctx.accounts.owner.key();
        user.bump = ctx.bumps.user_account;
    }
    user.collateral = user.collateral.checked_add(i64::try_from(amount).map_err(|_| error!(DashExError::MathOverflow))?).ok_or(error!(DashExError::MathOverflow))?;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            Transfer {
                from: ctx.accounts.user_collateral.to_account_info(),
                to: ctx.accounts.collateral_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )
}

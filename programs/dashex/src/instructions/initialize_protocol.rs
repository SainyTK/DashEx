use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::GlobalState;

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    pub collateral_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        token::mint = collateral_mint,
        token::authority = global_state,
        seeds = [b"collateral-vault"],
        bump
    )]
    pub collateral_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = admin,
        token::mint = collateral_mint,
        token::authority = global_state,
        seeds = [b"insurance-vault"],
        bump
    )]
    pub insurance_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeProtocol>) -> Result<()> {
    let global = &mut ctx.accounts.global_state;
    global.version = 0;
    global.admin = ctx.accounts.admin.key();
    global.collateral_mint = ctx.accounts.collateral_mint.key();
    global.collateral_vault = ctx.accounts.collateral_vault.key();
    global.insurance_vault = ctx.accounts.insurance_vault.key();
    global.market_count = 0;
    global.paused = false;
    global.bump = ctx.bumps.global_state;
    Ok(())
}

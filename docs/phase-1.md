# Phase 1 status

The initial Anchor workspace contains the six Phase 1 instructions:

- `initialize_protocol`
- `create_market`
- `deposit_collateral`
- `withdraw_collateral`
- `open_position`
- `close_position`

The program ID is `9XLPuTNvnnnmj6BM1ASrCZ92TvjuhggTmm4dmkm97Zcv`. It is a local build ID, not yet deployed to devnet.

## Implemented

- PDA-owned collateral and insurance token vaults
- Admin-only market creation
- One isolated user position
- Constant-product AMM reserve updates and limit-price checks
- Internal quote-entry accounting and realized PnL on full close
- Initial-margin checks on exposure increases and withdrawals
- Rust unit tests for AMM direction, quote signs, PnL, and margin rounding

## Not implemented

- Devnet deployment and devnet USDC setup
- Transaction-level tests for deposit, trade, and withdrawal
- Oracle validation, funding, and liquidation

## Commands

Start a new terminal after installing the Solana CLI, then run:

```sh
anchor build
cargo test --package dashex
```

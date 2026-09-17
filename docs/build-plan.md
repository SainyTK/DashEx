# Build plan

## Delivery rule

Do not start stock-market-hours features, PreStocks, extra markets, or market-maker routing until milestones M1 through M8 work together on devnet.

## Milestones

| Milestone | Deliverable |
| --- | --- |
| M0 | Anchor program deploys. |
| M1 | Wallet deposits devnet USDC. |
| M2 | Wallet opens long and short NVDA-PERP positions. |
| M3 | Pyth determines the index price. |
| M4 | PnL, margin, and leverage work. |
| M5 | Funding works. |
| M6 | Liquidation works. |
| M7 | Keeper updates funding and liquidates accounts. |
| M8 | Trading UI supports the full flow. |
| M9 | Market-hours risk controls work. |
| M10 | PreStocks-backed market works. |
| M11 | More markets work. |
| M12 | Market-maker and JIT liquidity routing works. |

## Phases

### Phase 0: protocol specification

Freeze the MVP before coding. Write and review the pricing, PnL, margin, funding, and liquidation equations. Define account fields and rounding rules.

### Phase 1: minimal perp engine

Implement `initialize_protocol`, `create_market`, `deposit_collateral`, `withdraw_collateral`, `open_position`, and `close_position`. Use a constant-product vAMM. Demonstrate a deposit, a trade, and PnL after a price move.

### Phase 2: oracle and risk engine

Read Pyth inside the program. Reject stale, mismatched, or low-confidence feeds. Implement position value, unrealized PnL, account equity, initial margin, maintenance margin, free collateral, and health.

### Phase 3: funding

Track index and mark price TWAPs. Compute a funding premium and update a cumulative funding index. Settle funding lazily when an account changes state. Add a keeper-callable `update_funding` instruction.

### Phase 4: liquidation and keeper

Implement `liquidate(user, market)` when equity is below maintenance margin. Pay a liquidation reward. Build a TypeScript process that reads protocol accounts, updates funding, finds unhealthy accounts, and submits liquidation transactions.

### Phase 5: trading UI

Build a familiar perpetuals interface. It needs long and short controls, collateral flow, a price chart, positions, and risk data. Show index price, mark price, funding, leverage, liquidation price, PnL, and health.

### Phase 6: stock-specific risk

When the underlying market closes or the oracle becomes stale, reduce maximum leverage, increase margin requirements, cap oracle divergence, and tighten funding limits.

### Phase 7: PreStocks

Add one private-asset perpetual only after NVDA-PERP is stable. Introduce an oracle abstraction that returns normalized price, confidence, and timestamp values.

### Phase 8: better liquidity

Only after the MVP, route between just-in-time market-maker quotes and the vAMM backstop.

## Validation

Write math tests before or alongside every instruction. Cover AMM reserve updates, price impact, PnL, margin, funding settlement, health boundaries, and liquidation outcomes.

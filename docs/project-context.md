# Project context

## Source

Project brief: <https://chatgpt.com/share/6aac0299-6eb4-83ec-baf3-c35cc65d04ae?ogimg=plain>

## Product

DashEx is a decentralized perpetual futures exchange for equity-like assets on Solana. It lets users take leveraged long or short exposure without holding the underlying stock.

The MVP uses devnet USDC as collateral and launches a single isolated-margin market, `NVDA-PERP`. Pyth provides the public-equity index price. DashEx's internal virtual AMM supplies execution and a mark price.

## Hackathon goal

Prove one credible end-to-end market on devnet. A judge should be able to connect a wallet, deposit fake USDC, trade NVDA-PERP, see Pyth-derived index pricing affect PnL and risk, accrue funding, and liquidate an unhealthy account.

Do not build a broad Drift replacement, an order book, many markets, or external market-maker routing before that flow works.

## Core design

- One collateral asset: devnet USDC.
- One market first: `NVDA-PERP`.
- Isolated margin.
- Constant-product virtual AMM for execution.
- Pyth as the normalized index-price source for public equities.
- Cumulative funding indices with lazy settlement when an account interacts.
- Permissionless liquidation when account equity falls below maintenance margin.
- A TypeScript keeper updates funding and sends liquidation transactions.

## Differentiator

Equities have trading hours while DashEx trades continuously. After the core MVP, the risk engine should change leverage, margin requirements, divergence limits, and funding limits when the underlying market is closed or the oracle is stale.

A later private-equity market, such as `OPENAI-PERP`, can consume a PreStocks price source. Markets must depend on a normalized oracle interface instead of Pyth-specific business logic.

## Non-goals for the MVP

- Multiple live markets.
- A central limit order book.
- External market makers or just-in-time liquidity.
- Mainnet deployment.
- Tight coupling between the trading engine and one oracle provider.

## Success criteria

1. The Anchor program deploys on devnet.
2. A wallet deposits and withdraws dUSDC within margin constraints.
3. A user can open and close both long and short NVDA-PERP positions through the vAMM.
4. Oracle validation guards risk-sensitive instructions.
5. The UI displays index, mark, PnL, margin, leverage, funding, health, and liquidation price.
6. Funding changes account equity correctly.
7. The keeper identifies and liquidates an unhealthy account.

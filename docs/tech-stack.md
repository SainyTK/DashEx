# Technical stack

## On-chain program

- Solana devnet
- Rust
- Anchor
- SPL Token for devnet USDC custody
- Pyth price feeds for the `NVDA/USD` index price

The program owns protocol state, market state, collateral accounting, position changes, funding indices, margin checks, and liquidation rules.

## Client and services

- TypeScript
- `@solana/web3.js`
- Anchor TypeScript client
- Solana Wallet Adapter
- A TypeScript keeper process for funding updates and liquidations

## Web application

- Next.js
- TypeScript
- Tailwind CSS
- Solana Wallet Adapter
- Anchor TypeScript client

## Repository layout

```text
dashex/
├── programs/
│   └── dashex/src/
│       ├── instructions/
│       ├── state/
│       ├── math/
│       ├── oracle/
│       └── lib.rs
├── sdk/typescript/
├── keeper/
├── app/
├── tests/
│   ├── trading.ts
│   ├── funding.ts
│   ├── margin.ts
│   └── liquidation.ts
└── docs/
```

## Engineering rules

- Keep financial calculations in independently unit-tested math modules.
- Keep Anchor instructions thin. They validate accounts, load state, call math functions, and persist results.
- Validate oracle feed identity, publish time, and confidence before every risk-sensitive action.
- Design market oracle inputs behind a normalized interface so Pyth and PreStocks can share the perp engine.
- Prefer fixed-point integer arithmetic on chain. Define rounding behavior and test boundary cases.

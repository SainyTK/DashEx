# DashEx

DashEx is a Solana perpetual futures exchange for equities. The first product is `NVDA-PERP` on devnet, collateralized in devnet USDC. It uses a constant-product virtual AMM for execution and Pyth for the index price.

The hackathon target is a complete trading loop: connect a wallet, deposit dUSDC, open or close an NVDA perpetual position, view PnL and risk, apply funding, and liquidate unhealthy accounts.

Read the local project record before implementation:

- [Project context](docs/project-context.md)
- [Technical stack](docs/tech-stack.md)
- [Build plan](docs/build-plan.md)
- [Protocol specification](docs/protocol-spec.md)
- [Phase 1 status](docs/phase-1.md)

## Initial layout

```text
programs/dashex/src/{instructions,state,math,oracle}
sdk/typescript
keeper
app
tests
docs
```

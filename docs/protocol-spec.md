# Protocol specification

This document freezes the Phase 0 rules for the first devnet market. Any change to a rule below needs a versioned migration plan and new tests.

## Scope

DashEx v0 has one isolated-margin market, `NVDA-PERP`, and devnet USDC collateral. The program uses a virtual constant-product AMM. There are no maker rebates, trading fees, or partial liquidations in v0.

## Units, precision, and arithmetic

| Value | Representation | Scale |
| --- | --- | --- |
| Collateral and quote amounts | unsigned or signed integer USDC atoms | `QUOTE_PRECISION = 1_000_000` |
| Base position amount | signed integer base atoms | `BASE_PRECISION = 1_000_000_000` |
| Price and peg multiplier | unsigned integer quote atoms per whole base asset | `PRICE_PRECISION = 1_000_000` |
| vAMM reserves | unsigned integer virtual reserve atoms | `AMM_RESERVE_PRECISION = 1_000_000_000` |
| Ratios and funding rates | unsigned or signed integer | `RATIO_PRECISION = 1_000_000` |
| Timestamps | signed Unix seconds | seconds |

All on-chain intermediate calculations use checked `u128` or `i128` arithmetic. A conversion back to `u64` or `i64` must fail on overflow. The program rejects a zero denominator and reserve updates that would leave a base or quote reserve below `MIN_RESERVE = 1_000_000`.

For a non-negative rational value `n / d`, use `floor(n / d)` when crediting a user or reducing a requirement, and `ceil(n / d)` when debiting a user or increasing a requirement. Signed calculations truncate toward zero only after their sign is separated. The program must document each exception in the instruction or math function that needs it.

## Accounts

```text
GlobalState
├── version: u16
├── admin: Pubkey
├── collateral_mint: Pubkey
├── collateral_vault: Pubkey
├── insurance_vault: Pubkey
├── market_count: u16
├── paused: bool
└── bump: u8

Market
├── version: u16
├── market_index: u16
├── status: MarketStatus
├── oracle: Pubkey
├── oracle_exponent: i8
├── base_asset_reserve: u64
├── quote_asset_reserve: u64
├── sqrt_k: u64
├── peg_multiplier: u64
├── cumulative_funding_rate: i64
├── last_funding_ts: i64
├── mark_twap: u64
├── index_twap: u64
├── twap_ts: i64
├── initial_margin_ratio: u32
├── maintenance_margin_ratio: u32
├── max_oracle_divergence_ratio: u32
├── max_oracle_age: i64
├── max_confidence_ratio: u32
├── max_funding_rate: u32
├── funding_period: i64
├── open_interest_long: u64
├── open_interest_short: u64
├── net_funding: i64
├── bad_debt: u64
└── bump: u8

UserAccount
├── version: u16
├── owner: Pubkey
├── collateral: i64
├── positions: [Position; 1]
└── bump: u8

Position
├── market_index: u16
├── base_asset_amount: i64
├── quote_asset_amount: i64
├── last_cumulative_funding_rate: i64
└── is_open: bool
```

`collateral` and `quote_asset_amount` are signed because realized losses can exceed a previous balance before liquidation closes the position. Users can only withdraw a positive collateral balance. `sqrt_k` lets the program verify `base_asset_reserve * quote_asset_reserve == sqrt_k²` after each reserve update.

`MarketStatus` has four states. `Active` permits all allowed actions. `ReduceOnly` permits only trades that reduce absolute base exposure and permits withdrawals that pass margin. `Paused` permits no deposits, withdrawals, or trades. `ClosedMarket` is reserved for later market-hours rules and has the same v0 behavior as `ReduceOnly`.

## Oracle normalization and validation

The oracle adapter returns `{ price, confidence, publish_time }` in `PRICE_PRECISION` quote atoms per base asset. Pyth-specific parsing stays inside the adapter.

Before an instruction that changes risk, the program checks all of the following:

1. The supplied oracle account equals `Market.oracle`.
2. The normalized price is positive.
3. `current_unix_timestamp - publish_time <= max_oracle_age`.
4. `ceil(confidence * RATIO_PRECISION / price) <= max_confidence_ratio`.
5. `abs(mark_price - index_price) / index_price <= max_oracle_divergence_ratio` for an action that increases exposure.

Withdrawals and liquidations validate feed identity, positive price, age, and confidence. They do not apply the divergence check because they reduce protocol risk. For v0 market creation, use `max_oracle_age = 60`, `max_confidence_ratio = 20_000` (2%), and `max_oracle_divergence_ratio = 100_000` (10%).

## vAMM execution and mark price

The AMM invariant is:

```text
base_asset_reserve * quote_asset_reserve = sqrt_k²
```

The mark price, in `PRICE_PRECISION`, is:

```text
mark_price = floor(quote_asset_reserve * peg_multiplier / base_asset_reserve)
```

A trade requests a signed `base_delta` in base atoms. A positive delta opens or increases a long. A negative delta opens or increases a short. Convert it to reserve atoms at the same 1e9 scale, then calculate:

```text
new_base_reserve = old_base_reserve - base_delta
new_quote_reserve = ceil(sqrt_k² / new_base_reserve)
user_quote_delta = -sign(base_delta) * ceil(abs(new_quote_reserve - old_quote_reserve) * peg_multiplier / AMM_RESERVE_PRECISION)
```

The program rejects a `new_base_reserve` below `MIN_RESERVE` and any trade whose absolute base delta is zero. It updates the user position as follows:

```text
new_base_asset_amount = old_base_asset_amount + base_delta
new_quote_asset_amount = old_quote_asset_amount + user_quote_delta
```

The reserve calculation is the source of execution price. `user_quote_delta` is negative for a long because the user pays quote, and positive for a short because the user receives quote. A trade that crosses through zero closes the old position first and opens the remainder at the same AMM execution path. It retains one net `base_asset_amount` and `quote_asset_amount` value.

v0 has no trading fee. The trade limit price check uses the absolute quote cost calculated above: longs reject when the effective execution price exceeds the user limit, shorts reject when it is below the user limit.

## PnL, equity, margin, and health

For each position, calculate mark value and unrealized PnL as:

```text
mark_value = trunc_toward_zero(base_asset_amount * mark_price / BASE_PRECISION)
unrealized_pnl = quote_asset_amount + mark_value
```

The formula works for both directions. A long has negative entry quote and a short has positive entry quote. Before computing equity, settle funding into `collateral` for every position touched by the instruction. Then:

```text
account_equity = collateral + sum(unrealized_pnl)
position_notional = ceil(abs(base_asset_amount) * mark_price / BASE_PRECISION)
initial_margin_required = sum(ceil(position_notional * initial_margin_ratio / RATIO_PRECISION))
maintenance_margin_required = sum(ceil(position_notional * maintenance_margin_ratio / RATIO_PRECISION))
free_collateral = account_equity - initial_margin_required
```

For v0 market creation, set `initial_margin_ratio = 200_000` (20%) and `maintenance_margin_ratio = 100_000` (10%). Creation rejects a maintenance ratio greater than its initial ratio or a zero initial ratio.

A deposit always succeeds when the SPL Token transfer succeeds. A withdrawal, new position, position increase, or trade that changes a position's direction succeeds only when `free_collateral >= 0` after the change. A decrease or close may leave free collateral negative, but cannot make a previously healthy account worse under the same validated mark price.

An account is liquidatable when `account_equity < maintenance_margin_required`. For UI output, health is `floor(account_equity * RATIO_PRECISION / maintenance_margin_required)` when maintenance margin is positive. It is `RATIO_PRECISION` for an account with no positions and non-negative equity, and `0` for an account with no positions and negative equity.

## Funding

Funding runs at most once per `funding_period = 3_600` seconds. The keeper may call it after that period has elapsed. `mark_twap` and `index_twap` use a time-weighted cumulative-price update with a one-hour window. The first update seeds both values and does not change funding.

For later updates, the program rejects an `index_twap` of zero.

```text
premium = trunc_toward_zero((mark_twap - index_twap) * RATIO_PRECISION / index_twap)
raw_funding_rate = trunc_toward_zero(premium / 24)
funding_rate = clamp(raw_funding_rate, -max_funding_rate, max_funding_rate)
cumulative_funding_rate += funding_rate
```

`max_funding_rate = 12_500` (1.25% per update). A positive funding rate means longs pay. On every position change, withdrawal, and liquidation, settle:

```text
funding_payment = trunc_toward_zero(base_asset_amount * (cumulative_funding_rate - last_cumulative_funding_rate) / BASE_PRECISION)
collateral -= funding_payment
last_cumulative_funding_rate = cumulative_funding_rate
```

`funding_payment` is quote atoms because the cumulative index is quote atoms per whole base asset. Funding settles against the market's virtual counterparty. The program records the net amount in `Market.net_funding`; a positive net amount moves to the insurance vault ledger and a negative amount is paid from that ledger. v0 rejects a funding update that would make the insurance-vault ledger negative.

## Liquidation

Anyone can liquidate a user when the validated account equity is below maintenance margin. The instruction settles funding, rechecks eligibility, and closes the entire position through the vAMM using the normal reserve calculation. It then computes a penalty from the pre-trade mark notional:

```text
liquidation_penalty = ceil(pre_trade_notional * 50_000 / RATIO_PRECISION)
liquidator_reward_target = ceil(pre_trade_notional * 15_000 / RATIO_PRECISION)
liquidator_reward = min(liquidator_reward_target, max(account_equity_after_close, 0))
insurance_contribution = min(liquidation_penalty - liquidator_reward, max(account_equity_after_close - liquidator_reward, 0))
```

The instruction debits the reward and insurance contribution from the user account, credits the liquidator's internal collateral account with the reward, and credits the insurance-vault ledger with the remainder. It does not transfer more than the positive equity left after closing. Any remaining negative equity becomes `Market.bad_debt` and is covered by the insurance-vault ledger before the transaction completes. The instruction rejects if insurance cannot cover it.

## Phase 0 acceptance tests

The math test suite must cover reserve invariant preservation, long and short quote signs, rounding at one atom, long and short PnL, margin boundaries, stale and high-confidence oracle rejection, positive and negative funding, funding caps, liquidation eligibility at equality, reward caps, and bad-debt rejection. Phase 1 starts only after these cases pass in pure Rust tests.

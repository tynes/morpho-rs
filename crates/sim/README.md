# morpho-rs-sim

A Rust simulation library for [Morpho Blue](https://morpho.org/) lending markets and [MetaMorpho](https://docs.morpho.org/metamorpho/overview) vaults. This crate enables offline APY calculations, position health tracking, vault deposit/withdrawal simulations, and yield optimization strategies without requiring on-chain transactions.

## Features

- **Market Simulation**: Supply, borrow, withdraw, and repay operations with accurate interest accrual
- **APY Calculations**: Calculate supply/borrow APYs using the Adaptive Curve IRM
- **Vault Operations**: Simulate MetaMorpho vault deposits, withdrawals, and reallocations
- **Position Tracking**: Monitor health factors, LTV, liquidation prices, and capacity limits
- **Yield Optimization**: Find optimal market allocations and best vaults for deposits
- **Public Allocator**: Simulate public reallocation with flow limits

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
morpho-rs-sim = "0.1"
```

## Quick Start

### Market APY Calculation

```rust
use morpho_rs_sim::{Market, MarketId, WAD};
use alloy_primitives::{FixedBytes, U256};

// Create a market with 80% utilization
let market = Market::new(
    FixedBytes::ZERO,                           // market ID
    U256::from(1_000_000) * WAD,                // total supply: 1M
    U256::from(800_000) * WAD,                  // total borrow: 800K
    U256::from(1_000_000) * WAD,                // supply shares
    U256::from(800_000) * WAD,                  // borrow shares
    1704067200,                                 // last update timestamp
    U256::from(100_000_000_000_000_000u64),     // 10% protocol fee
    Some(U256::from(1_268_391_679u64)),         // rate at target (~4% APY)
);

let timestamp = 1704153600; // current timestamp

// Get current APYs
let supply_apy = market.get_supply_apy(timestamp).unwrap();
let borrow_apy = market.get_borrow_apy(timestamp).unwrap();

println!("Supply APY: {:.2}%", supply_apy * 100.0);
println!("Borrow APY: {:.2}%", borrow_apy * 100.0);
```

### Simulating a Supply Operation

```rust
use morpho_rs_sim::{supply_apy_impact, Market, WAD};

let deposit_amount = U256::from(100_000) * WAD; // 100K tokens

let impact = supply_apy_impact(&market, deposit_amount, timestamp).unwrap();

println!("APY before: {:.2}%", impact.apy_before * 100.0);
println!("APY after: {:.2}%", impact.apy_after * 100.0);
println!("APY change: {:.4}%", impact.apy_delta * 100.0);
println!("Shares received: {}", impact.shares_received);
```

### Vault Simulation

```rust
use morpho_rs_sim::{
    Vault, VaultSimulation, VaultMarketConfig, vault_deposit_apy_impact, WAD
};
use std::collections::HashMap;

// Build vault with market allocations
let mut allocations = HashMap::new();
allocations.insert(
    market_id,
    VaultMarketConfig {
        market_id,
        cap: U256::from(2_000_000) * WAD,       // 2M cap
        supply_assets: U256::from(500_000) * WAD, // 500K supplied
        enabled: true,
        public_allocator_config: None,
    },
);

let vault = Vault {
    address: vault_address,
    asset_decimals: 18,
    fee: U256::from(100_000_000_000_000_000u64), // 10% performance fee
    total_assets: U256::from(500_000) * WAD,
    total_supply: U256::from(500_000) * WAD,
    last_total_assets: U256::from(500_000) * WAD,
    supply_queue: vec![market_id],
    withdraw_queue: vec![market_id],
    allocations,
    owner: owner_address,
    public_allocator_config: None,
};

let mut markets = HashMap::new();
markets.insert(market_id, market);

let simulation = VaultSimulation::new(vault, markets);

// Calculate deposit impact
let deposit = U256::from(50_000) * WAD;
let impact = vault_deposit_apy_impact(&simulation, deposit, timestamp).unwrap();

println!("Net APY before: {:.2}%", impact.apy_before * 100.0);
println!("Net APY after: {:.2}%", impact.apy_after * 100.0);
```

### Position Health Tracking

```rust
use morpho_rs_sim::{Position, Market, ORACLE_PRICE_SCALE, WAD};

// Create market with oracle price
let market = Market::new_with_oracle(
    market_id,
    total_supply,
    total_borrow,
    supply_shares,
    borrow_shares,
    last_update,
    fee,
    rate_at_target,
    Some(ORACLE_PRICE_SCALE),                    // 1:1 price
    U256::from(800_000_000_000_000_000u64),      // 80% LLTV
);

let position = Position::new(
    user_address,
    market_id,
    U256::from(1000) * WAD,  // supply shares
    U256::from(500) * WAD,   // borrow shares
    U256::from(1000) * WAD,  // collateral
);

// Check position health
let is_healthy = position.is_healthy(&market).unwrap();
let health_factor = position.health_factor(&market).unwrap();
let ltv = position.ltv(&market).unwrap();
let liquidation_price = position.liquidation_price(&market);
let max_borrowable = position.max_borrowable_assets(&market).unwrap();

println!("Healthy: {}", is_healthy);
println!("Health Factor: {:.2}", morpho_rs_sim::math::rate_to_f64(health_factor));
```

## Core Concepts

### WAD-Scaled Arithmetic

All monetary values use fixed-point arithmetic with 18 decimal places (WAD = 10^18):

```rust
use morpho_rs_sim::WAD;

let one_token = WAD;                      // 1.0
let half_token = WAD / U256::from(2);     // 0.5
let ten_percent = WAD / U256::from(10);   // 0.1 (10%)
```

### Share-Based Accounting

Morpho Blue uses shares to track positions, preventing manipulation attacks:

```rust
// Convert between assets and shares
let shares = market.to_supply_shares(assets, RoundingDirection::Down);
let assets = market.to_supply_assets(shares, RoundingDirection::Down);
```

### Interest Accrual

Interest compounds continuously using a Taylor series approximation:

```rust
// Accrue interest up to a timestamp
let updated_market = market.accrue_interest(new_timestamp)?;

// Interest is added to both supply and borrow totals
// Protocol fee is minted as additional supply shares
```

### Adaptive Curve IRM

The Interest Rate Model adjusts the rate at target utilization based on market conditions:

- **Target Utilization**: 90%
- **Curve Steepness**: 4x (rates increase 4x faster above target)
- **Adjustment Speed**: 50% per year (rate at target adapts over time)

```rust
use morpho_rs_sim::irm::{get_borrow_rate, TARGET_UTILIZATION};

let result = get_borrow_rate(utilization, rate_at_target, elapsed_seconds);

println!("Avg borrow rate: {}", result.avg_borrow_rate);
println!("End borrow rate: {}", result.end_borrow_rate);
println!("New rate at target: {}", result.end_rate_at_target);
```

## API Reference

### Market Module

| Function | Description |
|----------|-------------|
| `Market::new()` | Create a market without oracle |
| `Market::new_with_oracle()` | Create a market with price and LLTV |
| `market.supply()` | Simulate supply, returns new market and shares |
| `market.withdraw()` | Simulate withdrawal |
| `market.borrow()` | Simulate borrow |
| `market.repay()` | Simulate repayment |
| `market.accrue_interest()` | Update market state with accrued interest |
| `market.get_supply_apy()` | Calculate current supply APY |
| `market.get_borrow_apy()` | Calculate current borrow APY |
| `market.utilization()` | Get current utilization rate |
| `market.liquidity()` | Get available liquidity |
| `supply_apy_impact()` | Calculate APY impact of a supply |
| `borrow_apy_impact()` | Calculate APY impact of a borrow |
| `rank_markets_by_supply_apy()` | Rank markets by supply APY |
| `find_best_market_for_supply()` | Find optimal market for a deposit |

### Vault Module

| Function | Description |
|----------|-------------|
| `VaultSimulation::new()` | Create vault simulation with markets |
| `simulation.simulate_deposit()` | Simulate vault deposit |
| `simulation.simulate_withdraw()` | Simulate vault withdrawal |
| `simulation.simulate_reallocate()` | Simulate reallocation between markets |
| `simulation.simulate_public_reallocate()` | Simulate public allocator reallocation |
| `simulation.get_net_apy()` | Calculate net APY (after fees) |
| `simulation.get_apy()` | Calculate gross APY (before fees) |
| `vault_deposit_apy_impact()` | Calculate APY impact of deposit |
| `vault_withdraw_apy_impact()` | Calculate APY impact of withdrawal |
| `amount_for_vault_apy_impact()` | Find deposit amount for target APY change |
| `rank_vaults_by_apy()` | Rank vaults by net APY |
| `find_best_vault_for_deposit()` | Find optimal vault for a deposit |
| `find_optimal_market_allocation()` | Optimize allocation across markets |

### Position Module

| Function | Description |
|----------|-------------|
| `Position::new()` | Create a position |
| `Position::empty()` | Create an empty position |
| `position.supply_assets()` | Get supply value in assets |
| `position.borrow_assets()` | Get borrow value in assets |
| `position.health_factor()` | Calculate health factor |
| `position.ltv()` | Calculate loan-to-value ratio |
| `position.is_healthy()` | Check if position is healthy |
| `position.is_liquidatable()` | Check if position can be liquidated |
| `position.liquidation_price()` | Get price at which liquidation occurs |
| `position.max_borrowable_assets()` | Get additional borrowable amount |
| `position.withdrawable_collateral()` | Get withdrawable collateral amount |
| `position.get_capacities()` | Get all operation capacity limits |

### IRM Module

| Function | Description |
|----------|-------------|
| `get_borrow_rate()` | Calculate borrow rate with IRM adaptation |
| `get_utilization_at_borrow_rate()` | Inverse: utilization for a given rate |
| `get_supply_for_borrow_rate()` | Supply/withdraw needed for target rate |
| `w_exp()` | WAD-scaled exponential function |

### Math Module

| Function | Description |
|----------|-------------|
| `w_mul_down()` / `w_mul_up()` | WAD multiplication with rounding |
| `w_div_down()` / `w_div_up()` | WAD division with rounding |
| `shares_to_assets()` | Convert shares to assets |
| `assets_to_shares()` | Convert assets to shares |
| `rate_to_apy()` | Convert per-second rate to APY |
| `w_taylor_compounded()` | Taylor series for continuous compounding |

## Error Handling

All fallible operations return `Result<T, SimError>`:

```rust
use morpho_rs_sim::SimError;

match market.borrow(amount, timestamp) {
    Ok((new_market, shares)) => { /* success */ }
    Err(SimError::InsufficientMarketLiquidity { market_id }) => {
        println!("Not enough liquidity in market {:?}", market_id);
    }
    Err(e) => { /* handle other errors */ }
}
```

Common errors:
- `SimError::InsufficientMarketLiquidity` - Not enough liquidity for withdrawal/borrow
- `SimError::InsufficientCollateral` - Borrow would make position unhealthy
- `SimError::AllCapsReached` - Vault deposit exceeds all market caps
- `SimError::InvalidInterestAccrual` - Timestamp is before last update

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `WAD` | 10^18 | Fixed-point scaling factor |
| `ORACLE_PRICE_SCALE` | 10^36 | Oracle price scaling |
| `TARGET_UTILIZATION` | 0.9 WAD | IRM target utilization (90%) |
| `CURVE_STEEPNESS` | 4 WAD | IRM curve steepness |
| `LIQUIDATION_CURSOR` | 0.3 WAD | Liquidation incentive parameter |
| `MAX_LIQUIDATION_INCENTIVE_FACTOR` | 1.15 WAD | Maximum liquidation bonus (15%) |

## Testing

Run the test suite:

```bash
cargo test -p morpho-rs-sim
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Related

- [Morpho Blue Documentation](https://docs.morpho.org/)
- [MetaMorpho Documentation](https://docs.morpho.org/metamorpho/overview)
- [Morpho Blue Contracts](https://github.com/morpho-org/morpho-blue)

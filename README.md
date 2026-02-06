# morpho-rs

A Rust CLI and library for interacting with [Morpho](https://morpho.org) vaults across 25 chains, with offline simulation support.

## Features

- Query V1 (MetaMorpho) and V2 vaults
- View vault details, allocations, and rewards
- Deposit and withdraw from vaults
- Query user positions across chains
- Simulate APY impact, health factors, and yield optimization offline
- Multi-chain support (25 networks)
- Table or JSON output
- Use as a library (`morpho-rs-api`, `morpho-rs-sim`) or CLI

## Architecture

```
┌─────────────┐
│  morpho-rs  │  CLI (clap)
│    cli      │
└──────┬──────┘
       │
┌──────▼──────┐
│  morpho-rs  │  GraphQL client, MorphoClient
│    api      │
└──┬───────┬──┘
   │       │
┌──▼──┐ ┌──▼──────────┐
│ con │ │  morpho-rs   │  Offline simulation
│tracts│ │    sim       │  (feature-gated)
└─────┘ └─────────────┘
```

- **cli** depends on **api**
- **api** depends on **contracts** (on-chain transactions) and optionally **sim** (behind `sim` feature flag)
- **contracts** and **sim** are independent leaf crates

## Installation

```bash
cargo install --path crates/cli
```

## Usage

### List Vaults

```bash
# List V1 vaults
morpho vaultv1 list

# List V2 vaults on a specific chain
morpho vaultv2 list --chain base

# Filter by curator
morpho vaultv1 list --curator 0x...
```

### Vault Info

```bash
morpho vaultv1 info <VAULT_ADDRESS>
morpho vaultv2 info <VAULT_ADDRESS> --chain ethereum
```

### User Positions

```bash
morpho positions <USER_ADDRESS>
morpho positions <USER_ADDRESS> --chain base
```

### Deposit & Withdraw

```bash
# Deposit 100 tokens into a vault
morpho vaultv1 deposit <VAULT_ADDRESS> 100 --rpc-url <RPC_URL> --private-key <KEY>

# Withdraw 50 tokens
morpho vaultv2 withdraw <VAULT_ADDRESS> 50 --rpc-url <RPC_URL> --private-key <KEY>
```

Environment variables `ETH_RPC_URL` and `PRIVATE_KEY` can be used instead of flags.

### Output Format

```bash
# JSON output
morpho --format json vaultv1 list
```

## Library Usage

### API-Only Client

```rust,no_run
use morpho_rs_api::{MorphoClient, NamedChain};

#[tokio::main]
async fn main() -> Result<(), morpho_rs_api::ApiError> {
    let client = MorphoClient::new();
    let vaults = client.get_vaults_by_chain(NamedChain::Mainnet).await?;
    println!("Found {} vaults", vaults.len());
    Ok(())
}
```

### Transaction Client (Deposit / Withdraw)

```rust,no_run
use morpho_rs_api::{MorphoClient, MorphoClientConfig};
use alloy::primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), morpho_rs_api::ApiError> {
    let config = MorphoClientConfig::new()
        .with_rpc_url("https://eth.llamarpc.com")
        .with_private_key("0x...");
    let client = MorphoClient::with_config(config)?;

    let vault: Address = "0x...".parse().unwrap();
    let amount = U256::from(1_000_000);
    client.vault_v1()?.approve(vault, amount).await?;
    client.vault_v1()?.deposit(vault, amount).await?;
    Ok(())
}
```

### Simulation (APY Calculation)

```rust
use morpho_rs_sim::{Market, WAD};
use alloy_primitives::{FixedBytes, U256};

let market = Market::new(
    FixedBytes::ZERO,
    U256::from(1_000_000) * WAD,            // total supply
    U256::from(800_000) * WAD,              // total borrow
    U256::from(1_000_000) * WAD,            // supply shares
    U256::from(800_000) * WAD,              // borrow shares
    1704067200,                             // last update
    U256::from(100_000_000_000_000_000u64), // 10% fee
    Some(U256::from(1_268_391_679u64)),     // rate at target
);

let supply_apy = market.get_supply_apy(1704153600).unwrap();
let borrow_apy = market.get_borrow_apy(1704153600).unwrap();
assert!(borrow_apy > supply_apy);
```

## Supported Chains

Ethereum, Base, Polygon, Arbitrum, Optimism, World, Fraxtal, Scroll, Ink, Unichain, Sonic, Mode, Corn, Katana, Etherlink, Lisk, Hyperliquid, Sei, Linea, Monad, Stable, Cronos, Celo, Abstract, Sepolia.

## Project Structure

| Crate | Path | Description |
|-------|------|-------------|
| `morpho-rs-cli` | `crates/cli` | Command-line interface (clap) |
| `morpho-rs-api` | `crates/api` | GraphQL API client, `MorphoClient`, vault/position queries |
| `morpho-rs-contracts` | `crates/contracts` | ERC-4626 / ERC-20 bindings, transaction clients |
| `morpho-rs-sim` | `crates/sim` | Offline simulation: APY, IRM, vault/market modeling |

## Development

This project uses [`just`](https://github.com/casey/just) as a command runner.

```bash
just build          # cargo build
just test           # cargo test (unit + integration)
just test-e2e       # Fork tests (requires ETH_RPC_URL)
just test-all       # All tests including E2E
just fmt            # cargo fmt
just clippy         # cargo clippy
just check          # cargo check
```

### Lint Policy

The workspace enforces strict clippy lints via `Cargo.toml`:

- **Pedantic** lints enabled globally (with targeted overrides)
- **Panic safety**: `unwrap_used`, `expect_used`, `panic` are **denied**
- **No debug artifacts**: `dbg_macro`, `todo`, `unimplemented` are **denied**
- **No raw I/O in libraries**: `print_stdout`, `print_stderr` are **denied**

## License

MIT

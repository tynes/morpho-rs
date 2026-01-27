# morpho-rs-api

GraphQL API client for querying [Morpho](https://morpho.org) vaults with unified transaction support.

## Installation

```bash
cargo add morpho-rs-api
```

## Features

- **VaultV1Client / VaultV2Client** - Dedicated clients for V1 and V2 vault queries
- **MorphoApiClient** - Combined API client for both vault versions
- **MorphoClient** - Unified client combining API queries and on-chain transactions
- **25 supported chains** - Ethereum, Base, Arbitrum, Optimism, and more
- **User position queries** - Track positions, PnL, and ROE across chains
- **Flexible filtering** - Query vaults by chain, curator, APY, and more
- **alloy-chains integration** - Uses `NamedChain` from alloy-chains for chain types

## Usage

### Query-Only Client

```rust
use morpho_rs_api::{MorphoClient, NamedChain};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an API-only client (no transaction support)
    let client = MorphoClient::new();

    // Query vaults on a specific chain (returns Vec<Box<dyn Vault>>)
    let vaults = client.api().get_vaults_by_chain(NamedChain::Mainnet).await?;
    for vault in vaults {
        // Vault is a trait - access via methods
        println!("{}: {} (APY: {:.2}%)", vault.symbol(), vault.name(), vault.net_apy() * 100.0);
    }

    Ok(())
}
```

### Full Client with Transaction Support

```rust
use morpho_rs_api::{MorphoClient, MorphoClientConfig, NamedChain};
use alloy_primitives::{Address, U256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MorphoClientConfig::new()
        .with_rpc_url("https://eth.llamarpc.com")
        .with_private_key("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");

    let client = MorphoClient::with_config(config)?;

    // API queries still work
    let vaults = client.api().get_vaults_by_chain(NamedChain::Mainnet).await?;

    // Plus transaction support
    let vault: Address = "0x...".parse()?;
    let amount = U256::from(1000000);

    // V1 vault operations
    let balance = client.vault_v1()?.balance(vault).await?;
    client.vault_v1()?.deposit(vault, amount).await?;

    Ok(())
}
```

### Querying Vaults with Filters

```rust
use morpho_rs_api::{MorphoClient, VaultFiltersV1, NamedChain};

let client = MorphoClient::new();

// Build filters
let filters = VaultFiltersV1::new()
    .chain(NamedChain::Mainnet)
    .listed(true)
    .min_apy(0.05); // 5% minimum APY

let vaults = client.api().v1.get_vaults(Some(filters)).await?;
```

### Depositing and Withdrawing

```rust
use morpho_rs_api::{MorphoClient, MorphoClientConfig};
use alloy_primitives::{Address, U256};

let config = MorphoClientConfig::new()
    .with_rpc_url("https://eth.llamarpc.com")
    .with_private_key("0x...");

let client = MorphoClient::with_config(config)?;
let vault: Address = "0x...".parse()?;
let amount = U256::from(1000000);

// Deposit to V1 vault
client.vault_v1()?.deposit(vault, amount).await?;

// Withdraw from V2 vault
client.vault_v2()?.withdraw(vault, amount).await?;
```

### Querying User Positions

```rust
use morpho_rs_api::{MorphoClient, NamedChain};
use alloy_primitives::Address;

let client = MorphoClient::new();
let user: Address = "0x...".parse()?;

// Query positions on all chains
let positions = client.api().get_user_vault_positions(user, None).await?;
println!("V1 positions: {}", positions.vault_positions.len());
println!("V2 positions: {}", positions.vault_v2_positions.len());

// Query positions on specific chain
let positions = client.api().get_user_vault_positions(user, Some(NamedChain::Base)).await?;

// Get complete account overview
let overview = client.api().get_user_account_overview(user, NamedChain::Mainnet).await?;
println!("Total assets USD: {:?}", overview.state.total_assets_usd);
```

## Supported Chains

| Chain | ID | Aliases |
|-------|-----|---------|
| Ethereum | 1 | `ethereum`, `eth`, `mainnet` |
| Base | 8453 | `base` |
| Arbitrum | 42161 | `arbitrum`, `arb` |
| Optimism | 10 | `optimism`, `op` |
| Polygon | 137 | `polygon`, `matic` |
| Linea | 59144 | `linea` |
| Scroll | 534352 | `scroll` |
| Mode | 34443 | `mode` |
| Sonic | 146 | `sonic` |
| World Chain | 480 | `worldchain` |
| Fraxtal | 252 | `fraxtal` |
| Ink | 57073 | `ink` |
| Unichain | 130 | `unichain` |
| Corn | 21000000 | `corn` |
| Katana | 747474 | `katana` |
| Etherlink | 42793 | `etherlink` |
| Lisk | 1135 | `lisk` |
| Hyperliquid | 999 | `hyperliquid` |
| Sei | 1329 | `sei` |
| Monad | 143 | `monad` |
| Stable | 988 | `stable` |
| Cronos | 25 | `cronos` |
| Celo | 42220 | `celo` |
| Abstract | 2741 | `abstract` |
| Sepolia | 11155111 | `sepolia` (testnet) |

## Public API

### Client Types

- `MorphoClient` - Unified client with API + optional transaction support
- `MorphoClientConfig` - Configuration builder for MorphoClient
- `MorphoApiClient` - Combined V1 + V2 API client
- `VaultV1Client` - V1 vault query client
- `VaultV2Client` - V2 vault query client
- `VaultV1Operations` / `VaultV2Operations` - Transaction wrappers

### Data Types

- `Vault` - Trait for common vault operations (implemented by VaultV1 and VaultV2)
- `VaultV1` / `VaultV2` - Version-specific vault types implementing `Vault` trait
- `VaultStateV1` - V1 vault state with APY, fees, allocations
- `NamedChain` - Supported blockchain networks (from alloy-chains)
- `Asset` - Token information
- `UserVaultPositions` - User's vault positions
- `UserAccountOverview` - Complete user account state

### Filter Types

- `VaultFiltersV1` - Filter builder for V1 vault queries
- `VaultFiltersV2` - Filter builder for V2 vault queries

### Error Handling

```rust
pub enum ApiError {
    Request(reqwest::Error),
    GraphQL(String),
    Parse(String),
    VaultNotFound { address: Address, chain_id: i64 },
    InvalidAddress(String),
    InvalidChainId(i64),
    Contract(ContractError),
    TransactionNotConfigured,
}
```

## License

MIT

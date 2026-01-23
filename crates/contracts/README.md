# morpho-rs-contracts

Contract bindings and transaction clients for [Morpho](https://morpho.org) V1 (MetaMorpho) and V2 vaults.

## Installation

```bash
cargo add morpho-rs-contracts
```

## Features

- **VaultV1TransactionClient** - Execute transactions against MetaMorpho (V1) vaults
- **VaultV2TransactionClient** - Execute transactions against V2 vaults
- **ERC20/ERC4626 bindings** - Solidity interface bindings via `alloy::sol!`
- **HttpProvider** - Type alias for RPC connections using alloy

## Usage

### Creating a Transaction Client

```rust
use morpho_rs_contracts::{VaultV1TransactionClient, VaultV2TransactionClient};

// V1 (MetaMorpho) vaults
let v1_client = VaultV1TransactionClient::new(
    "https://eth.llamarpc.com",
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)?;

// V2 vaults
let v2_client = VaultV2TransactionClient::new(
    "https://eth.llamarpc.com",
    "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
)?;
```

### Querying Balance and Allowance

```rust
use alloy_primitives::{Address, U256};

let vault: Address = "0x...".parse()?;
let token: Address = "0x...".parse()?;
let owner = client.signer_address();

// Get underlying asset address
let asset = client.get_asset(vault).await?;

// Get token balance
let balance = client.get_balance(token, owner).await?;

// Get approval allowance
let allowance = client.get_allowance(token, owner, vault).await?;

// Get token decimals
let decimals = client.get_decimals(token).await?;
```

### Executing Deposit and Withdraw

```rust
use alloy_primitives::{Address, U256};

let vault: Address = "0x...".parse()?;
let amount = U256::from(1000000); // Amount in smallest units
let receiver = client.signer_address();

// Approve the vault to spend tokens (if needed)
let approval_receipt = client.approve_if_needed(asset, vault, amount).await?;

// Deposit assets into the vault
let deposit_receipt = client.deposit(vault, amount, receiver).await?;
println!("Deposit tx: {:?}", deposit_receipt.transaction_hash);

// Withdraw assets from the vault
let withdraw_receipt = client.withdraw(vault, amount, receiver, receiver).await?;
println!("Withdraw tx: {:?}", withdraw_receipt.transaction_hash);
```

## Public API

### Types

- `VaultV1TransactionClient` - Transaction client for V1 vaults
- `VaultV2TransactionClient` - Transaction client for V2 vaults
- `HttpProvider` - HTTP provider type alias
- `ContractError` - Error type for contract operations
- `Result<T>` - Result type alias

### VaultV1TransactionClient / VaultV2TransactionClient Methods

| Method | Description |
|--------|-------------|
| `new(rpc_url, private_key)` | Create a new transaction client |
| `signer_address()` | Get the signer's address |
| `get_asset(vault)` | Get the underlying asset address |
| `get_decimals(token)` | Get token decimals |
| `get_balance(token, owner)` | Get token balance |
| `get_allowance(token, owner, spender)` | Get approval allowance |
| `approve_if_needed(token, spender, amount)` | Approve if current allowance insufficient |
| `deposit(vault, amount, receiver)` | Deposit assets into vault |
| `withdraw(vault, amount, receiver, owner)` | Withdraw assets from vault |

### Error Types

```rust
pub enum ContractError {
    RpcConnection(String),
    TransactionFailed(String),
    InsufficientBalance { have: U256, need: U256 },
    InvalidPrivateKey,
}
```

## License

MIT

# morpho-rs

A Rust CLI and library for interacting with [Morpho](https://morpho.org) vaults.

## Features

- Query V1 (MetaMorpho) and V2 vaults
- View vault details, allocations, and rewards
- Deposit and withdraw from vaults
- Query user positions across chains
- Table or JSON output

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

## Supported Chains

Ethereum, Base, Arbitrum, Optimism, Polygon, Scroll, Sepolia, and [many more](crates/api/src/chains.rs).

## Project Structure

- `crates/cli` - Command-line interface
- `crates/api` - GraphQL API client for Morpho
- `crates/contracts` - Contract bindings and transaction clients

## License

MIT

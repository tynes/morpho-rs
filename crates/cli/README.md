# morpho-rs-cli

Command-line tool for interacting with [Morpho](https://morpho.org) vaults.

## Installation

```bash
cargo install morpho-rs-cli
```

Or build from source:

```bash
cargo install --path crates/cli
```

## Commands

### `vaultv1` - Query V1 (MetaMorpho) Vaults

```bash
# List vaults
morpho vaultv1 list
morpho vaultv1 list --chain base
morpho vaultv1 list --curator 0x... --whitelisted -n 50

# Get vault details
morpho vaultv1 info <VAULT_ADDRESS>
morpho vaultv1 info <VAULT_ADDRESS> --chain ethereum

# Deposit into vault
morpho vaultv1 deposit <VAULT_ADDRESS> <AMOUNT>

# Withdraw from vault
morpho vaultv1 withdraw <VAULT_ADDRESS> <AMOUNT>
```

### `vaultv2` - Query V2 Vaults

```bash
# List vaults
morpho vaultv2 list
morpho vaultv2 list --chain arbitrum -n 100

# Get vault details
morpho vaultv2 info <VAULT_ADDRESS> --chain base

# Deposit into vault
morpho vaultv2 deposit <VAULT_ADDRESS> <AMOUNT>

# Withdraw from vault
morpho vaultv2 withdraw <VAULT_ADDRESS> <AMOUNT>
```

### `positions` - Query User Positions

```bash
# Query positions across all chains
morpho positions <USER_ADDRESS>

# Query positions on specific chain
morpho positions <USER_ADDRESS> --chain ethereum
```

## Examples

### List Vaults

```bash
# List top 25 V1 vaults on Ethereum
morpho vaultv1 list

# List V2 vaults on Base
morpho vaultv2 list --chain base

# List whitelisted vaults by curator
morpho vaultv1 list --curator 0x... --whitelisted

# Output as JSON
morpho --format json vaultv1 list
```

### View Vault Info

```bash
# Get V1 vault details
morpho vaultv1 info 0x78Fc2c2eD1A4cDb5402365934aE5648aDAd094d0

# Get V2 vault on Base
morpho vaultv2 info 0x... --chain base
```

### Deposit and Withdraw

```bash
# Set environment variables
export ETH_RPC_URL="https://eth.llamarpc.com"
export PRIVATE_KEY="0x..."

# Deposit 100 tokens
morpho vaultv1 deposit 0x78Fc2c2eD1A4cDb5402365934aE5648aDAd094d0 100

# Deposit with explicit RPC and key
morpho vaultv2 deposit 0x... 50.5 --rpc-url https://base.llamarpc.com --private-key 0x...

# Withdraw 25 tokens
morpho vaultv1 withdraw 0x... 25
```

### Query User Positions

```bash
# All positions across all chains
morpho positions 0xYourAddress...

# Positions on specific chain
morpho positions 0xYourAddress... --chain optimism

# Output as JSON for scripting
morpho --format json positions 0xYourAddress...
```

## Environment Variables

| Variable | Description | Used By |
|----------|-------------|---------|
| `ETH_RPC_URL` | RPC endpoint URL | `deposit`, `withdraw` |
| `PRIVATE_KEY` | Private key for signing | `deposit`, `withdraw` |

Environment variables can be overridden with command-line flags:
- `--rpc-url <URL>` overrides `ETH_RPC_URL`
- `--private-key <KEY>` overrides `PRIVATE_KEY`

## Chain Aliases

The `--chain` flag accepts chain names or IDs:

| Chain | Aliases |
|-------|---------|
| Ethereum | `ethereum`, `eth`, `mainnet`, `1` |
| Base | `base`, `8453` |
| Arbitrum | `arbitrum`, `arb`, `42161` |
| Optimism | `optimism`, `op`, `10` |
| Polygon | `polygon`, `matic`, `137` |
| Linea | `linea`, `59144` |
| Scroll | `scroll`, `534352` |
| Mode | `mode`, `34443` |
| Sonic | `sonic`, `146` |
| World Chain | `worldchain`, `480` |
| Fraxtal | `fraxtal`, `252` |
| Ink | `ink`, `57073` |
| Unichain | `unichain`, `130` |
| Hemi | `hemi`, `43111` |
| Corn | `corn`, `21000000` |
| Plume | `plume`, `98866` |
| Camp | `camp`, `123420001114` |
| Katana | `katana`, `747474` |
| Etherlink | `etherlink`, `42793` |
| TAC | `tac`, `239` |
| Lisk | `lisk`, `1135` |
| Hyperliquid | `hyperliquid`, `999` |
| Sei | `sei`, `1329` |
| Zero-G | `zerog`, `0g`, `16661` |
| Monad | `monad`, `143` |
| Stable | `stable`, `988` |
| Cronos | `cronos`, `25` |
| Celo | `celo`, `42220` |
| Abstract | `abstract`, `2741` |
| Sepolia | `sepolia`, `11155111` (testnet) |

## Output Format

Use `--format` to control output:

```bash
# Table output (default)
morpho vaultv1 list

# JSON output
morpho --format json vaultv1 list
morpho --format json positions 0x...
```

## License

MIT

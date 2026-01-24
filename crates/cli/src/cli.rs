//! CLI argument definitions using clap.

use std::str::FromStr;

use alloy_chains::NamedChain;
use clap::{Parser, Subcommand, ValueEnum};

/// Morpho CLI - Query V1 and V2 vaults
#[derive(Parser, Debug)]
#[command(name = "morpho")]
#[command(about = "CLI tool for querying Morpho vaults", long_about = None)]
pub struct Cli {
    /// Output format
    #[arg(long, global = true, default_value = "table")]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query V1 (MetaMorpho) vaults
    #[command(name = "vaultv1")]
    VaultV1 {
        #[command(subcommand)]
        subcommand: VaultV1Subcommand,
    },
    /// Query V2 vaults
    #[command(name = "vaultv2")]
    VaultV2 {
        #[command(subcommand)]
        subcommand: VaultV2Subcommand,
    },
    /// Query user vault positions (V1 and V2)
    #[command(name = "positions")]
    Positions(PositionsArgs),
}

#[derive(Subcommand, Debug)]
pub enum VaultV1Subcommand {
    /// List V1 vaults
    List(ListArgs),
    /// Get detailed info for a specific V1 vault
    Info(InfoArgs),
    /// Deposit assets into a V1 vault
    Deposit(DepositArgs),
    /// Withdraw assets from a V1 vault
    Withdraw(WithdrawArgs),
}

#[derive(Subcommand, Debug)]
pub enum VaultV2Subcommand {
    /// List V2 vaults
    List(ListArgs),
    /// Get detailed info for a specific V2 vault
    Info(InfoArgs),
    /// Deposit assets into a V2 vault
    Deposit(DepositArgs),
    /// Withdraw assets from a V2 vault
    Withdraw(WithdrawArgs),
}

#[derive(Parser, Debug)]
pub struct ListArgs {
    /// Filter by chain (e.g., ethereum, base, polygon)
    #[arg(long)]
    pub chain: Option<ChainArg>,

    /// Filter by curator address
    #[arg(long)]
    pub curator: Option<String>,

    /// Only show whitelisted (listed) vaults
    #[arg(long)]
    pub whitelisted: bool,

    /// Limit the number of results
    #[arg(short = 'n', long, default_value = "25")]
    pub limit: usize,
}

#[derive(Parser, Debug)]
pub struct InfoArgs {
    /// Vault address
    pub address: String,

    /// Chain the vault is on (default: ethereum)
    #[arg(long, default_value = "ethereum")]
    pub chain: ChainArg,
}

#[derive(Parser, Debug)]
pub struct PositionsArgs {
    /// User wallet address to query positions for
    pub address: String,

    /// Chain to query (omit to query all chains)
    #[arg(long)]
    pub chain: Option<ChainArg>,
}

#[derive(Parser, Debug)]
pub struct DepositArgs {
    /// Vault contract address
    pub vault: String,

    /// Amount to deposit in human-readable units (e.g., "100.5")
    pub amount: String,

    /// Chain the vault is on (optional, not used for transaction routing)
    #[arg(long)]
    pub chain: Option<ChainArg>,

    /// Private key for signing transactions (can also use PRIVATE_KEY env var)
    #[arg(long, env = "PRIVATE_KEY")]
    pub private_key: String,

    /// RPC URL for the target chain (can also use ETH_RPC_URL env var)
    #[arg(long, env = "ETH_RPC_URL")]
    pub rpc_url: String,
}

#[derive(Parser, Debug)]
pub struct WithdrawArgs {
    /// Vault contract address
    pub vault: String,

    /// Amount to withdraw in human-readable units (e.g., "100.5")
    pub amount: String,

    /// Chain the vault is on (optional, not used for transaction routing)
    #[arg(long)]
    pub chain: Option<ChainArg>,

    /// Private key for signing transactions (can also use PRIVATE_KEY env var)
    #[arg(long, env = "PRIVATE_KEY")]
    pub private_key: String,

    /// RPC URL for the target chain (can also use ETH_RPC_URL env var)
    #[arg(long, env = "ETH_RPC_URL")]
    pub rpc_url: String,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
}

/// Wrapper for NamedChain that implements FromStr with aliases
#[derive(Clone, Copy, Debug)]
pub struct ChainArg(pub NamedChain);

impl FromStr for ChainArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chain = match s.to_lowercase().as_str() {
            // Ethereum aliases
            "ethereum" | "eth" | "mainnet" | "1" => NamedChain::Mainnet,
            // Base aliases
            "base" | "8453" => NamedChain::Base,
            // Polygon aliases
            "polygon" | "matic" | "137" => NamedChain::Polygon,
            // Arbitrum aliases
            "arbitrum" | "arb" | "42161" => NamedChain::Arbitrum,
            // Optimism aliases
            "optimism" | "op" | "10" => NamedChain::Optimism,
            // Other chains (using their network names)
            "worldchain" | "world" | "480" => NamedChain::World,
            "fraxtal" | "252" => NamedChain::Fraxtal,
            "scroll" | "534352" => NamedChain::Scroll,
            "ink" | "57073" => NamedChain::Ink,
            "unichain" | "130" => NamedChain::Unichain,
            "sonic" | "146" => NamedChain::Sonic,
            "mode" | "34443" => NamedChain::Mode,
            "corn" | "21000000" => NamedChain::Corn,
            "katana" | "747474" => NamedChain::Katana,
            "etherlink" | "42793" => NamedChain::Etherlink,
            "lisk" | "1135" => NamedChain::Lisk,
            "hyperliquid" | "999" => NamedChain::Hyperliquid,
            "sei" | "1329" => NamedChain::Sei,
            "linea" | "59144" => NamedChain::Linea,
            "monad" | "143" => NamedChain::Monad,
            "stable" | "988" => NamedChain::StableMainnet,
            "cronos" | "25" => NamedChain::Cronos,
            "celo" | "42220" => NamedChain::Celo,
            "abstract" | "2741" => NamedChain::Abstract,
            "sepolia" | "11155111" => NamedChain::Sepolia,
            _ => return Err(format!("Unknown chain: {}", s)),
        };
        Ok(ChainArg(chain))
    }
}

impl std::fmt::Display for ChainArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

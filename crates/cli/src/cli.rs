//! CLI argument definitions using clap.

use std::str::FromStr;

use api::Chain;
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

/// Wrapper for Chain that implements FromStr with aliases
#[derive(Clone, Copy, Debug)]
pub struct ChainArg(pub Chain);

impl FromStr for ChainArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chain = match s.to_lowercase().as_str() {
            // Ethereum aliases
            "ethereum" | "eth" | "mainnet" | "1" => Chain::EthMainnet,
            // Base aliases
            "base" | "8453" => Chain::BaseMainnet,
            // Polygon aliases
            "polygon" | "matic" | "137" => Chain::PolygonMainnet,
            // Arbitrum aliases
            "arbitrum" | "arb" | "42161" => Chain::ArbitrumMainnet,
            // Optimism aliases
            "optimism" | "op" | "10" => Chain::OptimismMainnet,
            // Other chains (using their network names)
            "worldchain" | "480" => Chain::WorldChainMainnet,
            "fraxtal" | "252" => Chain::FraxtalMainnet,
            "scroll" | "534352" => Chain::ScrollMainnet,
            "ink" | "57073" => Chain::InkMainnet,
            "unichain" | "130" => Chain::Unichain,
            "sonic" | "146" => Chain::SonicMainnet,
            "hemi" | "43111" => Chain::HemiMainnet,
            "mode" | "34443" => Chain::ModeMainnet,
            "corn" | "21000000" => Chain::CornMainnet,
            "plume" | "98866" => Chain::PlumeMainnet,
            "camp" | "123420001114" => Chain::CampMainnet,
            "katana" | "747474" => Chain::KatanaMainnet,
            "etherlink" | "42793" => Chain::EtherlinkMainnet,
            "tac" | "239" => Chain::TacMainnet,
            "lisk" | "1135" => Chain::LiskMainnet,
            "hyperliquid" | "999" => Chain::HyperliquidMainnet,
            "sei" | "1329" => Chain::SeiMainnet,
            "zerog" | "0g" | "16661" => Chain::ZeroGMainnet,
            "linea" | "59144" => Chain::LineaMainnet,
            "monad" | "143" => Chain::MonadMainnet,
            "stable" | "988" => Chain::StableMainnet,
            "cronos" | "25" => Chain::CronosMainnet,
            "celo" | "42220" => Chain::CeloMainnet,
            "abstract" | "2741" => Chain::AbstractMainnet,
            "sepolia" | "11155111" => Chain::Sepolia,
            _ => return Err(format!("Unknown chain: {}", s)),
        };
        Ok(ChainArg(chain))
    }
}

impl std::fmt::Display for ChainArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.network())
    }
}

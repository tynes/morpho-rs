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

    /// Morpho API URL (can also use MORPHO_API_URL env var)
    #[arg(long, global = true, env = "MORPHO_API_URL")]
    pub api_url: Option<String>,

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // ChainArg::FromStr tests - Ethereum aliases
    #[test]
    fn test_chain_arg_ethereum() {
        let chain: ChainArg = "ethereum".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    #[test]
    fn test_chain_arg_eth() {
        let chain: ChainArg = "eth".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    #[test]
    fn test_chain_arg_mainnet() {
        let chain: ChainArg = "mainnet".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    #[test]
    fn test_chain_arg_ethereum_by_id() {
        let chain: ChainArg = "1".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    // ChainArg::FromStr tests - Base aliases
    #[test]
    fn test_chain_arg_base() {
        let chain: ChainArg = "base".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Base);
    }

    #[test]
    fn test_chain_arg_base_by_id() {
        let chain: ChainArg = "8453".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Base);
    }

    // ChainArg::FromStr tests - Polygon aliases
    #[test]
    fn test_chain_arg_polygon() {
        let chain: ChainArg = "polygon".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Polygon);
    }

    #[test]
    fn test_chain_arg_matic() {
        let chain: ChainArg = "matic".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Polygon);
    }

    #[test]
    fn test_chain_arg_polygon_by_id() {
        let chain: ChainArg = "137".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Polygon);
    }

    // ChainArg::FromStr tests - Arbitrum aliases
    #[test]
    fn test_chain_arg_arbitrum() {
        let chain: ChainArg = "arbitrum".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Arbitrum);
    }

    #[test]
    fn test_chain_arg_arb() {
        let chain: ChainArg = "arb".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Arbitrum);
    }

    #[test]
    fn test_chain_arg_arbitrum_by_id() {
        let chain: ChainArg = "42161".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Arbitrum);
    }

    // ChainArg::FromStr tests - Optimism aliases
    #[test]
    fn test_chain_arg_optimism() {
        let chain: ChainArg = "optimism".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Optimism);
    }

    #[test]
    fn test_chain_arg_op() {
        let chain: ChainArg = "op".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Optimism);
    }

    #[test]
    fn test_chain_arg_optimism_by_id() {
        let chain: ChainArg = "10".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Optimism);
    }

    // ChainArg::FromStr tests - Other chains
    #[test]
    fn test_chain_arg_worldchain() {
        let chain: ChainArg = "worldchain".parse().unwrap();
        assert_eq!(chain.0, NamedChain::World);
    }

    #[test]
    fn test_chain_arg_fraxtal() {
        let chain: ChainArg = "fraxtal".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Fraxtal);
    }

    #[test]
    fn test_chain_arg_scroll() {
        let chain: ChainArg = "scroll".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Scroll);
    }

    #[test]
    fn test_chain_arg_ink() {
        let chain: ChainArg = "ink".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Ink);
    }

    #[test]
    fn test_chain_arg_unichain() {
        let chain: ChainArg = "unichain".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Unichain);
    }

    #[test]
    fn test_chain_arg_sonic() {
        let chain: ChainArg = "sonic".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Sonic);
    }

    #[test]
    fn test_chain_arg_mode() {
        let chain: ChainArg = "mode".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mode);
    }

    #[test]
    fn test_chain_arg_sepolia() {
        let chain: ChainArg = "sepolia".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Sepolia);
    }

    // ChainArg::FromStr tests - Invalid and case sensitivity
    #[test]
    fn test_chain_arg_invalid() {
        let result: Result<ChainArg, _> = "invalid_chain".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown chain"));
    }

    #[test]
    fn test_chain_arg_case_insensitive_upper() {
        let chain: ChainArg = "ETHEREUM".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    #[test]
    fn test_chain_arg_case_insensitive_mixed() {
        let chain: ChainArg = "EtHeReUm".parse().unwrap();
        assert_eq!(chain.0, NamedChain::Mainnet);
    }

    // ChainArg Display test
    #[test]
    fn test_chain_arg_display() {
        let chain = ChainArg(NamedChain::Mainnet);
        assert_eq!(format!("{}", chain), "mainnet");
    }

    // CLI argument parsing tests
    #[test]
    fn test_cli_vaultv1_list_defaults() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::List(args) } => {
                assert_eq!(args.limit, 25);
                assert!(args.chain.is_none());
                assert!(args.curator.is_none());
                assert!(!args.whitelisted);
            }
            _ => panic!("Expected VaultV1 List command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_list_with_chain() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list", "--chain", "base"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::List(args) } => {
                assert!(args.chain.is_some());
                assert_eq!(args.chain.unwrap().0, NamedChain::Base);
            }
            _ => panic!("Expected VaultV1 List command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_list_with_limit() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list", "-n", "50"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::List(args) } => {
                assert_eq!(args.limit, 50);
            }
            _ => panic!("Expected VaultV1 List command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_list_with_curator() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list", "--curator", "0x1234"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::List(args) } => {
                assert_eq!(args.curator, Some("0x1234".to_string()));
            }
            _ => panic!("Expected VaultV1 List command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_list_whitelisted() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list", "--whitelisted"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::List(args) } => {
                assert!(args.whitelisted);
            }
            _ => panic!("Expected VaultV1 List command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_info() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "info", "0x1234567890abcdef"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::Info(args) } => {
                assert_eq!(args.address, "0x1234567890abcdef");
                assert_eq!(args.chain.0, NamedChain::Mainnet); // default
            }
            _ => panic!("Expected VaultV1 Info command"),
        }
    }

    #[test]
    fn test_cli_vaultv1_info_with_chain() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "info", "0x1234", "--chain", "polygon"]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::Info(args) } => {
                assert_eq!(args.address, "0x1234");
                assert_eq!(args.chain.0, NamedChain::Polygon);
            }
            _ => panic!("Expected VaultV1 Info command"),
        }
    }

    #[test]
    fn test_cli_vaultv2_list() {
        let cli = Cli::parse_from(["morpho", "vaultv2", "list"]);
        match cli.command {
            Commands::VaultV2 { subcommand: VaultV2Subcommand::List(args) } => {
                assert_eq!(args.limit, 25);
            }
            _ => panic!("Expected VaultV2 List command"),
        }
    }

    #[test]
    fn test_cli_positions() {
        let cli = Cli::parse_from(["morpho", "positions", "0xuser1234"]);
        match cli.command {
            Commands::Positions(args) => {
                assert_eq!(args.address, "0xuser1234");
                assert!(args.chain.is_none());
            }
            _ => panic!("Expected Positions command"),
        }
    }

    #[test]
    fn test_cli_positions_with_chain() {
        let cli = Cli::parse_from(["morpho", "positions", "0xuser", "--chain", "arbitrum"]);
        match cli.command {
            Commands::Positions(args) => {
                assert_eq!(args.address, "0xuser");
                assert_eq!(args.chain.unwrap().0, NamedChain::Arbitrum);
            }
            _ => panic!("Expected Positions command"),
        }
    }

    #[test]
    fn test_cli_output_format_table() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list"]);
        assert!(matches!(cli.format, OutputFormat::Table));
    }

    #[test]
    fn test_cli_output_format_json() {
        let cli = Cli::parse_from(["morpho", "--format", "json", "vaultv1", "list"]);
        assert!(matches!(cli.format, OutputFormat::Json));
    }

    #[test]
    fn test_cli_deposit_args() {
        let cli = Cli::parse_from([
            "morpho", "vaultv1", "deposit",
            "0xvault", "100.5",
            "--private-key", "0xprivkey",
            "--rpc-url", "http://localhost:8545"
        ]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::Deposit(args) } => {
                assert_eq!(args.vault, "0xvault");
                assert_eq!(args.amount, "100.5");
                assert_eq!(args.private_key, "0xprivkey");
                assert_eq!(args.rpc_url, "http://localhost:8545");
            }
            _ => panic!("Expected VaultV1 Deposit command"),
        }
    }

    #[test]
    fn test_cli_withdraw_args() {
        let cli = Cli::parse_from([
            "morpho", "vaultv1", "withdraw",
            "0xvault", "50.0",
            "--private-key", "0xprivkey",
            "--rpc-url", "http://localhost:8545"
        ]);
        match cli.command {
            Commands::VaultV1 { subcommand: VaultV1Subcommand::Withdraw(args) } => {
                assert_eq!(args.vault, "0xvault");
                assert_eq!(args.amount, "50.0");
                assert_eq!(args.private_key, "0xprivkey");
                assert_eq!(args.rpc_url, "http://localhost:8545");
            }
            _ => panic!("Expected VaultV1 Withdraw command"),
        }
    }

    #[test]
    fn test_cli_invalid_command() {
        let result = Cli::try_parse_from(["morpho", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_missing_required_arg() {
        // positions requires address
        let result = Cli::try_parse_from(["morpho", "positions"]);
        assert!(result.is_err());
    }

    // API URL global argument tests
    #[test]
    fn test_cli_api_url_default_none() {
        let cli = Cli::parse_from(["morpho", "vaultv1", "list"]);
        assert!(cli.api_url.is_none());
    }

    #[test]
    fn test_cli_api_url_flag() {
        let cli = Cli::parse_from(["morpho", "--api-url", "http://custom-api.test", "vaultv1", "list"]);
        assert_eq!(cli.api_url, Some("http://custom-api.test".to_string()));
    }

    #[test]
    fn test_cli_api_url_with_other_globals() {
        let cli = Cli::parse_from([
            "morpho",
            "--api-url", "http://test.api",
            "--format", "json",
            "vaultv2", "list"
        ]);
        assert_eq!(cli.api_url, Some("http://test.api".to_string()));
        assert!(matches!(cli.format, OutputFormat::Json));
    }
}

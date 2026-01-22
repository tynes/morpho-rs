//! Morpho CLI - Query V1 and V2 vaults.

mod cli;
mod commands;
mod output;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands, VaultV1Subcommand, VaultV2Subcommand};
use commands::{run_v1_info, run_v1_list, run_v2_info, run_v2_list};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::VaultV1 { subcommand } => match subcommand {
            VaultV1Subcommand::List(args) => {
                run_v1_list(&args, cli.format).await?;
            }
            VaultV1Subcommand::Info(args) => {
                run_v1_info(&args, cli.format).await?;
            }
        },
        Commands::VaultV2 { subcommand } => match subcommand {
            VaultV2Subcommand::List(args) => {
                run_v2_list(&args, cli.format).await?;
            }
            VaultV2Subcommand::Info(args) => {
                run_v2_info(&args, cli.format).await?;
            }
        },
    }

    Ok(())
}

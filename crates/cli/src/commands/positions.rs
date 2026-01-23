//! User positions command implementation.

use alloy_primitives::Address;
use anyhow::Result;
use morpho_rs_api::{MorphoClient, UserVaultPositions};

use crate::cli::{OutputFormat, PositionsArgs};
use crate::output::format_user_positions;

pub async fn run_positions(args: &PositionsArgs, format: OutputFormat) -> Result<()> {
    let client = MorphoClient::new();

    let chain = args.chain.map(|c| c.0);

    let positions = match client.get_user_vault_positions(&args.address, chain).await {
        Ok(p) => p,
        Err(morpho_rs_api::ApiError::GraphQL(msg)) if msg.contains("No results") => {
            // User has no positions on this chain - return empty result
            let address = args.address.parse().unwrap_or(Address::ZERO);
            UserVaultPositions {
                address,
                vault_positions: vec![],
                vault_v2_positions: vec![],
            }
        }
        Err(e) => return Err(e.into()),
    };

    match format {
        OutputFormat::Table => {
            println!("{}", format_user_positions(&positions));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&positions)?;
            println!("{}", json);
        }
    }

    Ok(())
}

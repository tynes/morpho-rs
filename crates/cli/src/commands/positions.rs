//! User positions command implementation.

use alloy_primitives::Address;
use anyhow::Result;
use morpho_rs_api::{ClientConfig, MorphoClient, MorphoClientConfig, UserVaultPositions};

use crate::cli::{OutputFormat, PositionsArgs};
use crate::output::format_user_positions;

/// Create a MorphoClient with optional API URL.
fn create_client(api_url: Option<&str>) -> Result<MorphoClient> {
    if let Some(url) = api_url {
        let api_config = ClientConfig::new().with_api_url(url.parse()?);
        let config = MorphoClientConfig::new().with_api_config(api_config);
        Ok(MorphoClient::with_config(config)?)
    } else {
        Ok(MorphoClient::new())
    }
}

pub async fn run_positions(args: &PositionsArgs, format: OutputFormat, api_url: Option<&str>) -> Result<()> {
    let client = create_client(api_url)?;

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

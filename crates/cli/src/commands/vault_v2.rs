//! V2 vault command implementations.

use alloy_chains::NamedChain;
use anyhow::Result;
use morpho_rs_api::{ClientConfig, VaultV2, VaultV2Client};

use crate::cli::{InfoArgs, ListArgs, OutputFormat};
use crate::output::{format_v2_vault_detail, format_v2_vaults_table};

/// Create a ClientConfig with a custom page size and optional API URL.
fn client_config_with_page_size(page_size: i64, api_url: Option<&str>) -> Result<ClientConfig> {
    let config = ClientConfig::new().with_page_size(page_size);
    if let Some(url) = api_url {
        Ok(config.with_api_url(url.parse()?))
    } else {
        Ok(config)
    }
}

/// Create a default ClientConfig with optional API URL.
fn client_config(api_url: Option<&str>) -> Result<ClientConfig> {
    let config = ClientConfig::new();
    if let Some(url) = api_url {
        Ok(config.with_api_url(url.parse()?))
    } else {
        Ok(config)
    }
}

pub async fn run_v2_list(args: &ListArgs, format: OutputFormat, api_url: Option<&str>) -> Result<()> {
    // Use larger page size when client-side filtering is needed (e.g., curator filter)
    // to ensure we have enough results after filtering
    let page_size = if args.curator.is_some() {
        100.min(args.limit.max(50) as i64)
    } else {
        args.limit as i64
    };
    let config = client_config_with_page_size(page_size, api_url)?;
    let client = VaultV2Client::with_config(config);

    let vaults = if args.whitelisted {
        // Use whitelisted filter
        let chain = args.chain.map(|c| c.0);
        client.get_whitelisted_vaults(chain).await?
    } else if let Some(chain_arg) = args.chain {
        // Use chain filter
        client.get_vaults_by_chain(chain_arg.0).await?
    } else {
        // Get all vaults (no filters)
        client.get_vaults(None).await?
    };

    // Apply additional client-side filters
    let mut vaults: Vec<VaultV2> = vaults;

    // V2 doesn't have API-level curator filter, so filter client-side
    if let Some(curator_address) = &args.curator {
        let curator_lower = curator_address.to_lowercase();
        vaults.retain(|v| {
            v.curator
                .as_ref()
                .map(|c| format!("{}", c).to_lowercase() == curator_lower)
                .unwrap_or(false)
        });
    }

    // Limit results
    vaults.truncate(args.limit);

    // Output
    match format {
        OutputFormat::Table => {
            println!("{}", format_v2_vaults_table(&vaults));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&vaults)?;
            println!("{}", json);
        }
    }

    Ok(())
}

pub async fn run_v2_info(args: &InfoArgs, format: OutputFormat, api_url: Option<&str>) -> Result<()> {
    let config = client_config(api_url)?;
    let client = VaultV2Client::with_config(config);
    let chain: NamedChain = args.chain.0;

    let vault = client.get_vault(&args.address, chain).await?;

    match format {
        OutputFormat::Table => {
            println!("{}", format_v2_vault_detail(&vault));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&vault)?;
            println!("{}", json);
        }
    }

    Ok(())
}

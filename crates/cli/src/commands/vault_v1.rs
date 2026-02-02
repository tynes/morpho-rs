//! V1 vault command implementations.

use alloy_chains::NamedChain;
use anyhow::Result;
use morpho_rs_api::{ClientConfig, VaultV1, VaultV1Client};

use crate::cli::{InfoArgs, ListArgs, OutputFormat};
use crate::output::{format_v1_vault_detail, format_v1_vaults_table};

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

pub async fn run_v1_list(args: &ListArgs, format: OutputFormat, api_url: Option<&str>) -> Result<()> {
    // Use smaller page size to avoid query complexity issues
    let config = client_config_with_page_size(args.limit as i64, api_url)?;
    let client = VaultV1Client::with_config(config);

    let vaults = if args.curator.is_some() {
        // Use curator filter
        let curator = args.curator.as_ref().unwrap();
        let chain = args.chain.map(|c| c.0);
        client.get_vaults_by_curator(curator, chain).await?
    } else if args.whitelisted {
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

    // Apply additional filters if needed
    let mut vaults: Vec<VaultV1> = vaults;

    // If we used curator filter but also want whitelisted, filter client-side
    if args.curator.is_some() && args.whitelisted {
        vaults.retain(|v| v.whitelisted || v.listed);
    }

    // Limit results
    vaults.truncate(args.limit);

    // Output
    match format {
        OutputFormat::Table => {
            println!("{}", format_v1_vaults_table(&vaults));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&vaults)?;
            println!("{}", json);
        }
    }

    Ok(())
}

pub async fn run_v1_info(args: &InfoArgs, format: OutputFormat, api_url: Option<&str>) -> Result<()> {
    let config = client_config(api_url)?;
    let client = VaultV1Client::with_config(config);
    let chain: NamedChain = args.chain.0;

    let vault = client.get_vault(&args.address, chain).await?;

    match format {
        OutputFormat::Table => {
            println!("{}", format_v1_vault_detail(&vault));
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&vault)?;
            println!("{}", json);
        }
    }

    Ok(())
}

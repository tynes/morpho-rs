//! Detailed output formatting for single vault info.

use api::{VaultV1, VaultV2};
use colored::Colorize;

fn format_address(addr: &impl std::fmt::Display) -> String {
    format!("{}", addr)
}

fn format_apy(apy: f64) -> String {
    format!("{:.2}%", apy * 100.0)
}

fn format_usd(value: Option<f64>) -> String {
    match value {
        Some(v) if v >= 1_000_000.0 => format!("${:.2}M", v / 1_000_000.0),
        Some(v) if v >= 1_000.0 => format!("${:.2}K", v / 1_000.0),
        Some(v) => format!("${:.2}", v),
        None => "-".to_string(),
    }
}

fn format_fee(fee: f64) -> String {
    format!("{:.2}%", fee * 100.0)
}

pub fn format_v1_vault_detail(vault: &VaultV1) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&format!("{}\n", vault.name.bold()));
    output.push_str(&format!("{}\n\n", "=".repeat(60)));

    // Basic Info
    output.push_str(&format!("{}\n", "Basic Info".cyan().bold()));
    output.push_str(&format!("  Address:     {}\n", format_address(&vault.address)));
    output.push_str(&format!("  Symbol:      {}\n", vault.symbol));
    output.push_str(&format!("  Chain:       {}\n", vault.chain));
    output.push_str(&format!("  Listed:      {}\n", if vault.listed { "Yes" } else { "No" }));
    output.push_str(&format!("  Whitelisted: {}\n", if vault.whitelisted { "Yes" } else { "No" }));
    output.push_str(&format!("  Featured:    {}\n\n", if vault.featured { "Yes" } else { "No" }));

    // Asset Info
    output.push_str(&format!("{}\n", "Asset".cyan().bold()));
    output.push_str(&format!("  Symbol:   {}\n", vault.asset.symbol));
    output.push_str(&format!("  Address:  {}\n", format_address(&vault.asset.address)));
    output.push_str(&format!("  Decimals: {}\n\n", vault.asset.decimals));

    // State/Metrics
    if let Some(state) = &vault.state {
        output.push_str(&format!("{}\n", "State & Metrics".cyan().bold()));

        if let Some(curator) = &state.curator {
            output.push_str(&format!("  Curator:      {}\n", format_address(curator)));
        }
        if let Some(owner) = &state.owner {
            output.push_str(&format!("  Owner:        {}\n", format_address(owner)));
        }
        if let Some(guardian) = &state.guardian {
            output.push_str(&format!("  Guardian:     {}\n", format_address(guardian)));
        }

        output.push_str(&format!("  Fee:          {}\n", format_fee(state.fee)));
        output.push_str(&format!("  APY:          {}\n", format_apy(state.apy)));
        output.push_str(&format!("  Net APY:      {}\n", format_apy(state.net_apy)));
        output.push_str(&format!("  Total Assets: {}\n", format_usd(state.total_assets_usd)));
        output.push_str(&format!("  Timelock:     {} seconds\n\n", state.timelock));

        // Allocations
        if !state.allocation.is_empty() {
            output.push_str(&format!("{}\n", "Market Allocations".cyan().bold()));
            for alloc in &state.allocation {
                let collateral = alloc.collateral_asset_symbol.as_deref().unwrap_or("N/A");
                let loan = alloc.loan_asset_symbol.as_deref().unwrap_or("N/A");
                output.push_str(&format!(
                    "  {} / {} - {}\n",
                    loan,
                    collateral,
                    format_usd(alloc.supply_assets_usd)
                ));
            }
            output.push('\n');
        }
    }

    // Allocators
    if !vault.allocators.is_empty() {
        output.push_str(&format!("{}\n", "Allocators".cyan().bold()));
        for allocator in &vault.allocators {
            output.push_str(&format!("  {}\n", format_address(&allocator.address)));
        }
        output.push('\n');
    }

    // Warnings
    if !vault.warnings.is_empty() {
        output.push_str(&format!("{}\n", "Warnings".yellow().bold()));
        for warning in &vault.warnings {
            output.push_str(&format!(
                "  [{}] {}\n",
                warning.level.to_uppercase(),
                warning.warning_type
            ));
        }
    }

    output
}

pub fn format_v2_vault_detail(vault: &VaultV2) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", "=".repeat(60)));
    output.push_str(&format!("{}\n", vault.name.bold()));
    output.push_str(&format!("{}\n\n", "=".repeat(60)));

    // Basic Info
    output.push_str(&format!("{}\n", "Basic Info".cyan().bold()));
    output.push_str(&format!("  Address:     {}\n", format_address(&vault.address)));
    output.push_str(&format!("  Symbol:      {}\n", vault.symbol));
    output.push_str(&format!("  Chain:       {}\n", vault.chain));
    output.push_str(&format!("  Listed:      {}\n", if vault.listed { "Yes" } else { "No" }));
    output.push_str(&format!("  Whitelisted: {}\n\n", if vault.whitelisted { "Yes" } else { "No" }));

    // Asset Info
    output.push_str(&format!("{}\n", "Asset".cyan().bold()));
    output.push_str(&format!("  Symbol:   {}\n", vault.asset.symbol));
    output.push_str(&format!("  Address:  {}\n", format_address(&vault.asset.address)));
    output.push_str(&format!("  Decimals: {}\n\n", vault.asset.decimals));

    // State/Metrics
    output.push_str(&format!("{}\n", "State & Metrics".cyan().bold()));

    if let Some(curator) = &vault.curator {
        output.push_str(&format!("  Curator:         {}\n", format_address(curator)));
    }
    if let Some(owner) = &vault.owner {
        output.push_str(&format!("  Owner:           {}\n", format_address(owner)));
    }

    if let Some(perf_fee) = vault.performance_fee {
        output.push_str(&format!("  Performance Fee: {}\n", format_fee(perf_fee)));
    }
    if let Some(mgmt_fee) = vault.management_fee {
        output.push_str(&format!("  Management Fee:  {}\n", format_fee(mgmt_fee)));
    }

    if let Some(apy) = vault.apy {
        output.push_str(&format!("  APY:             {}\n", format_apy(apy)));
    }
    if let Some(net_apy) = vault.net_apy {
        output.push_str(&format!("  Net APY:         {}\n", format_apy(net_apy)));
    }
    if let Some(avg_apy) = vault.avg_apy {
        output.push_str(&format!("  Avg APY:         {}\n", format_apy(avg_apy)));
    }
    if let Some(avg_net_apy) = vault.avg_net_apy {
        output.push_str(&format!("  Avg Net APY:     {}\n", format_apy(avg_net_apy)));
    }

    output.push_str(&format!("  Total Assets:    {}\n", format_usd(vault.total_assets_usd)));
    output.push_str(&format!("  Liquidity:       {}\n\n", format_usd(vault.liquidity_usd)));

    // Adapters
    if !vault.adapters.is_empty() {
        output.push_str(&format!("{}\n", "Adapters".cyan().bold()));
        for adapter in &vault.adapters {
            output.push_str(&format!(
                "  {} ({}) - {}\n",
                adapter.adapter_type,
                format_address(&adapter.address)[..10].to_string() + "...",
                format_usd(adapter.assets_usd)
            ));
        }
        output.push('\n');
    }

    // Rewards
    if !vault.rewards.is_empty() {
        output.push_str(&format!("{}\n", "Rewards".cyan().bold()));
        for reward in &vault.rewards {
            let apr = reward
                .supply_apr
                .map(|a| format_apy(a))
                .unwrap_or_else(|| "-".to_string());
            output.push_str(&format!(
                "  {} - {} APR\n",
                reward.asset_symbol,
                apr
            ));
        }
        output.push('\n');
    }

    // Warnings
    if !vault.warnings.is_empty() {
        output.push_str(&format!("{}\n", "Warnings".yellow().bold()));
        for warning in &vault.warnings {
            output.push_str(&format!(
                "  [{}] {}\n",
                warning.level.to_uppercase(),
                warning.warning_type
            ));
        }
    }

    output
}

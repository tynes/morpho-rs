//! Output formatting for user vault positions.

use morpho_rs_api::UserVaultPositions;
use tabled::{
    settings::{object::Rows, Alignment, Modify, Style},
    Table, Tabled,
};

#[derive(Tabled)]
struct PositionRow {
    #[tabled(rename = "Type")]
    vault_type: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Vault Name")]
    name: String,
    #[tabled(rename = "Symbol")]
    symbol: String,
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Assets (USD)")]
    assets_usd: String,
    #[tabled(rename = "PnL (USD)")]
    pnl_usd: String,
}

fn truncate_address(addr: &str) -> String {
    if addr.len() > 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() > max_len {
        format!("{}...", &name[..max_len - 3])
    } else {
        name.to_string()
    }
}

fn format_usd(value: Option<f64>) -> String {
    match value {
        Some(v) if v >= 1_000_000.0 => format!("${:.2}M", v / 1_000_000.0),
        Some(v) if v >= 1_000.0 => format!("${:.2}K", v / 1_000.0),
        Some(v) if v < 0.0 && v > -1_000.0 => format!("-${:.2}", v.abs()),
        Some(v) if v <= -1_000.0 && v > -1_000_000.0 => format!("-${:.2}K", v.abs() / 1_000.0),
        Some(v) if v <= -1_000_000.0 => format!("-${:.2}M", v.abs() / 1_000_000.0),
        Some(v) => format!("${:.2}", v),
        None => "-".to_string(),
    }
}

pub fn format_user_positions(positions: &UserVaultPositions) -> String {
    let v1_count = positions.vault_positions.len();
    let v2_count = positions.vault_v2_positions.len();

    if v1_count == 0 && v2_count == 0 {
        return "No positions found.".to_string();
    }

    let mut rows: Vec<PositionRow> = Vec::with_capacity(v1_count + v2_count);

    // Add V1 positions
    for pos in &positions.vault_positions {
        let pnl_usd = pos.state.as_ref().and_then(|s| s.pnl_usd);

        rows.push(PositionRow {
            vault_type: "V1".to_string(),
            chain: pos.vault.chain.as_str().to_string(),
            name: truncate_name(&pos.vault.name, 25),
            symbol: pos.vault.symbol.clone(),
            address: truncate_address(&format!("{}", pos.vault.address)),
            assets_usd: format_usd(pos.assets_usd),
            pnl_usd: format_usd(pnl_usd),
        });
    }

    // Add V2 positions
    for pos in &positions.vault_v2_positions {
        rows.push(PositionRow {
            vault_type: "V2".to_string(),
            chain: pos.vault.chain.as_str().to_string(),
            name: truncate_name(&pos.vault.name, 25),
            symbol: pos.vault.symbol.clone(),
            address: truncate_address(&format!("{}", pos.vault.address)),
            assets_usd: format_usd(pos.assets_usd),
            pnl_usd: format_usd(pos.pnl_usd),
        });
    }

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(Modify::new(Rows::new(1..)).with(Alignment::left()));

    format!(
        "User: {}\n\n{}",
        truncate_address(&format!("{}", positions.address)),
        table
    )
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // truncate_address tests (same function as table.rs but tested here for coverage)
    #[test]
    fn test_truncate_address_long() {
        let addr = "0x1234567890abcdef1234567890abcdef12345678";
        assert_eq!(truncate_address(addr), "0x1234...5678");
    }

    #[test]
    fn test_truncate_address_short() {
        let addr = "0x1234";
        assert_eq!(truncate_address(addr), "0x1234");
    }

    // truncate_name tests
    #[test]
    fn test_truncate_name_under_limit() {
        assert_eq!(truncate_name("Short Name", 20), "Short Name");
    }

    #[test]
    fn test_truncate_name_over_limit() {
        assert_eq!(truncate_name("This is a very long name", 20), "This is a very lo...");
    }

    // format_usd tests for negative values
    #[test]
    fn test_format_usd_negative_small() {
        assert_eq!(format_usd(Some(-50.0)), "-$50.00");
    }

    #[test]
    fn test_format_usd_negative_thousands() {
        assert_eq!(format_usd(Some(-1500.0)), "-$1.50K");
    }

    #[test]
    fn test_format_usd_negative_millions() {
        assert_eq!(format_usd(Some(-2_500_000.0)), "-$2.50M");
    }

    // format_usd tests for positive values
    #[test]
    fn test_format_usd_none() {
        assert_eq!(format_usd(None), "-");
    }

    #[test]
    fn test_format_usd_small_value() {
        assert_eq!(format_usd(Some(123.45)), "$123.45");
    }

    #[test]
    fn test_format_usd_thousands() {
        assert_eq!(format_usd(Some(1500.0)), "$1.50K");
    }

    #[test]
    fn test_format_usd_millions() {
        assert_eq!(format_usd(Some(2_500_000.0)), "$2.50M");
    }

    #[test]
    fn test_format_usd_zero() {
        assert_eq!(format_usd(Some(0.0)), "$0.00");
    }
}

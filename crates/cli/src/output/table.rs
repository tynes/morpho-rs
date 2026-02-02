//! Table formatting for vault lists.

use morpho_rs_api::{VaultV1, VaultV2};
use tabled::{
    settings::{object::Rows, Alignment, Modify, Style},
    Table, Tabled,
};

#[derive(Tabled)]
struct VaultV1Row {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Curator")]
    curator: String,
    #[tabled(rename = "APY")]
    apy: String,
    #[tabled(rename = "TVL (USD)")]
    tvl_usd: String,
}

#[derive(Tabled)]
struct VaultV2Row {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Address")]
    address: String,
    #[tabled(rename = "Chain")]
    chain: String,
    #[tabled(rename = "Curator")]
    curator: String,
    #[tabled(rename = "APY")]
    apy: String,
    #[tabled(rename = "TVL (USD)")]
    tvl_usd: String,
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

pub fn format_v1_vaults_table(vaults: &[VaultV1]) -> String {
    if vaults.is_empty() {
        return "No vaults found.".to_string();
    }

    let rows: Vec<VaultV1Row> = vaults
        .iter()
        .map(|v| {
            let curator = v
                .state
                .as_ref()
                .and_then(|s| s.curator.as_ref())
                .map(|c| truncate_address(&format!("{}", c)))
                .unwrap_or_else(|| "-".to_string());

            let apy = v
                .state
                .as_ref()
                .map(|s| format_apy(s.net_apy))
                .unwrap_or_else(|| "-".to_string());

            let tvl_usd = v.state.as_ref().and_then(|s| s.total_assets_usd);

            VaultV1Row {
                name: truncate_name(&v.name, 30),
                address: truncate_address(&format!("{}", v.address)),
                chain: v.chain.as_str().to_string(),
                curator,
                apy,
                tvl_usd: format_usd(tvl_usd),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(Modify::new(Rows::new(1..)).with(Alignment::left()));

    table.to_string()
}

pub fn format_v2_vaults_table(vaults: &[VaultV2]) -> String {
    if vaults.is_empty() {
        return "No vaults found.".to_string();
    }

    let rows: Vec<VaultV2Row> = vaults
        .iter()
        .map(|v| {
            let curator = v
                .curator
                .as_ref()
                .map(|c| truncate_address(&format!("{}", c)))
                .unwrap_or_else(|| "-".to_string());

            let apy = v
                .net_apy
                .map(format_apy)
                .unwrap_or_else(|| "-".to_string());

            VaultV2Row {
                name: truncate_name(&v.name, 30),
                address: truncate_address(&format!("{}", v.address)),
                chain: v.chain.as_str().to_string(),
                curator,
                apy,
                tvl_usd: format_usd(v.total_assets_usd),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .with(Modify::new(Rows::new(1..)).with(Alignment::left()));

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // truncate_address tests
    #[test]
    fn test_truncate_address_long() {
        let addr = "0x1234567890abcdef1234567890abcdef12345678";
        assert_eq!(truncate_address(addr), "0x1234...5678");
    }

    #[test]
    fn test_truncate_address_exact_10() {
        let addr = "0x12345678";
        assert_eq!(truncate_address(addr), "0x12345678");
    }

    #[test]
    fn test_truncate_address_short() {
        let addr = "0x1234";
        assert_eq!(truncate_address(addr), "0x1234");
    }

    #[test]
    fn test_truncate_address_11_chars() {
        let addr = "0x123456789";
        assert_eq!(truncate_address(addr), "0x1234...6789");
    }

    // truncate_name tests
    #[test]
    fn test_truncate_name_under_limit() {
        assert_eq!(truncate_name("Short Name", 20), "Short Name");
    }

    #[test]
    fn test_truncate_name_at_limit() {
        assert_eq!(truncate_name("Exactly Twenty Chars", 20), "Exactly Twenty Chars");
    }

    #[test]
    fn test_truncate_name_over_limit() {
        assert_eq!(truncate_name("This is a very long name", 20), "This is a very lo...");
    }

    #[test]
    fn test_truncate_name_custom_limit() {
        assert_eq!(truncate_name("Hello World", 8), "Hello...");
    }

    // format_apy tests
    #[test]
    fn test_format_apy_zero() {
        assert_eq!(format_apy(0.0), "0.00%");
    }

    #[test]
    fn test_format_apy_five_percent() {
        assert_eq!(format_apy(0.05), "5.00%");
    }

    #[test]
    fn test_format_apy_small_fraction() {
        assert_eq!(format_apy(0.0012), "0.12%");
    }

    #[test]
    fn test_format_apy_large_value() {
        assert_eq!(format_apy(1.5), "150.00%");
    }

    #[test]
    fn test_format_apy_precise_decimal() {
        // 0.12345 * 100 = 12.345, rounds to 12.35
        assert_eq!(format_apy(0.12345), "12.35%");
    }

    // format_usd tests
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
    fn test_format_usd_exact_thousand() {
        assert_eq!(format_usd(Some(1000.0)), "$1.00K");
    }

    #[test]
    fn test_format_usd_millions() {
        assert_eq!(format_usd(Some(2_500_000.0)), "$2.50M");
    }

    #[test]
    fn test_format_usd_exact_million() {
        assert_eq!(format_usd(Some(1_000_000.0)), "$1.00M");
    }

    #[test]
    fn test_format_usd_zero() {
        assert_eq!(format_usd(Some(0.0)), "$0.00");
    }
}

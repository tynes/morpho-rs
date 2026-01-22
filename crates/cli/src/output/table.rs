//! Table formatting for vault lists.

use api::{VaultV1, VaultV2};
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
                chain: v.chain.network().to_string(),
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
                .map(|a| format_apy(a))
                .unwrap_or_else(|| "-".to_string());

            VaultV2Row {
                name: truncate_name(&v.name, 30),
                address: truncate_address(&format!("{}", v.address)),
                chain: v.chain.network().to_string(),
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

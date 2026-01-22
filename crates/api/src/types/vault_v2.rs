//! V2 vault types.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;
use super::chain::Chain;
use super::scalars::{parse_address, parse_bigint};

/// Represents a Morpho V2 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV2 {
    /// The vault's contract address.
    pub address: Address,
    /// The vault's name.
    pub name: String,
    /// The vault's symbol.
    pub symbol: String,
    /// The blockchain the vault is deployed on.
    pub chain: Chain,
    /// Whether the vault is listed on the Morpho UI.
    pub listed: bool,
    /// Whether the vault is whitelisted.
    pub whitelisted: bool,
    /// The vault's underlying asset.
    pub asset: Asset,
    /// The curator's address.
    pub curator: Option<Address>,
    /// The owner's address.
    pub owner: Option<Address>,
    /// Total assets in the vault (in asset's smallest unit).
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: Option<f64>,
    /// Total supply of vault shares.
    pub total_supply: U256,
    /// Current share price.
    pub share_price: Option<f64>,
    /// Performance fee (as a fraction).
    pub performance_fee: Option<f64>,
    /// Management fee (as a fraction).
    pub management_fee: Option<f64>,
    /// Average APY.
    pub avg_apy: Option<f64>,
    /// Average net APY after fees.
    pub avg_net_apy: Option<f64>,
    /// Current APY.
    pub apy: Option<f64>,
    /// Current net APY.
    pub net_apy: Option<f64>,
    /// Liquidity available.
    pub liquidity: U256,
    /// Liquidity in USD.
    pub liquidity_usd: Option<f64>,
    /// Vault adapters.
    pub adapters: Vec<VaultAdapter>,
    /// Vault rewards.
    pub rewards: Vec<VaultReward>,
    /// Vault warnings.
    pub warnings: Vec<VaultV2Warning>,
}

/// Vault adapter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAdapter {
    /// Adapter ID.
    pub id: String,
    /// Adapter contract address.
    pub address: Address,
    /// Adapter type (e.g., MetaMorpho, MorphoMarketV1).
    pub adapter_type: String,
    /// Assets held by this adapter.
    pub assets: U256,
    /// Assets held in USD.
    pub assets_usd: Option<f64>,
}

/// Vault reward configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultReward {
    /// Reward asset address.
    pub asset_address: Address,
    /// Reward asset symbol.
    pub asset_symbol: String,
    /// Supply APR from rewards.
    pub supply_apr: Option<f64>,
    /// Yearly supply tokens.
    pub yearly_supply_tokens: Option<f64>,
}

/// V2 vault warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV2Warning {
    /// Warning type.
    pub warning_type: String,
    /// Warning level.
    pub level: String,
}

impl VaultV2 {
    /// Create a VaultV2 from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        address: &str,
        name: String,
        symbol: String,
        chain_id: i64,
        listed: bool,
        whitelisted: bool,
        asset: Asset,
        curator: Option<&str>,
        owner: Option<&str>,
        total_assets: &str,
        total_assets_usd: Option<f64>,
        total_supply: &str,
        share_price: Option<f64>,
        performance_fee: Option<f64>,
        management_fee: Option<f64>,
        avg_apy: Option<f64>,
        avg_net_apy: Option<f64>,
        apy: Option<f64>,
        net_apy: Option<f64>,
        liquidity: &str,
        liquidity_usd: Option<f64>,
        adapters: Vec<VaultAdapter>,
        rewards: Vec<VaultReward>,
        warnings: Vec<VaultV2Warning>,
    ) -> Option<Self> {
        Some(VaultV2 {
            address: parse_address(address)?,
            name,
            symbol,
            chain: Chain::from_id(chain_id)?,
            listed,
            whitelisted,
            asset,
            curator: curator.and_then(parse_address),
            owner: owner.and_then(parse_address),
            total_assets: parse_bigint(total_assets)?,
            total_assets_usd,
            total_supply: parse_bigint(total_supply)?,
            share_price,
            performance_fee,
            management_fee,
            avg_apy,
            avg_net_apy,
            apy,
            net_apy,
            liquidity: parse_bigint(liquidity).unwrap_or(U256::ZERO),
            liquidity_usd,
            adapters,
            rewards,
            warnings,
        })
    }
}

impl VaultAdapter {
    /// Create a VaultAdapter from GraphQL response data.
    pub fn from_gql(
        id: String,
        address: &str,
        adapter_type: String,
        assets: &str,
        assets_usd: Option<f64>,
    ) -> Option<Self> {
        Some(VaultAdapter {
            id,
            address: parse_address(address)?,
            adapter_type,
            assets: parse_bigint(assets).unwrap_or(U256::ZERO),
            assets_usd,
        })
    }
}

impl VaultReward {
    /// Create a VaultReward from GraphQL response data.
    pub fn from_gql(
        asset_address: &str,
        asset_symbol: String,
        supply_apr: Option<f64>,
        yearly_supply_tokens: Option<f64>,
    ) -> Option<Self> {
        Some(VaultReward {
            asset_address: parse_address(asset_address)?,
            asset_symbol,
            supply_apr,
            yearly_supply_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_adapter_from_gql() {
        let adapter = VaultAdapter::from_gql(
            "adapter-1".to_string(),
            "0x1234567890123456789012345678901234567890",
            "MetaMorpho".to_string(),
            "1000000000000000000",
            Some(1000.0),
        )
        .unwrap();
        assert_eq!(adapter.id, "adapter-1");
        assert_eq!(adapter.adapter_type, "MetaMorpho");
        assert_eq!(adapter.assets, U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_vault_reward_from_gql() {
        let reward = VaultReward::from_gql(
            "0x1234567890123456789012345678901234567890",
            "MORPHO".to_string(),
            Some(0.05),
            Some(1000.0),
        )
        .unwrap();
        assert_eq!(reward.asset_symbol, "MORPHO");
        assert_eq!(reward.supply_apr, Some(0.05));
    }
}

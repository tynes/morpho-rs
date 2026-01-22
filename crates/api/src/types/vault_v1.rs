//! V1 (MetaMorpho) vault types.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;
use super::chain::Chain;
use super::scalars::{parse_address, parse_bigint};

/// Represents a Morpho V1 (MetaMorpho) vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV1 {
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
    /// Whether the vault is featured.
    pub featured: bool,
    /// Whether the vault is whitelisted.
    pub whitelisted: bool,
    /// The vault's underlying asset.
    pub asset: Asset,
    /// Current vault state.
    pub state: Option<VaultStateV1>,
    /// Vault allocators.
    pub allocators: Vec<VaultAllocator>,
    /// Vault warnings.
    pub warnings: Vec<VaultWarning>,
}

/// Current state of a V1 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultStateV1 {
    /// The curator's address.
    pub curator: Option<Address>,
    /// The owner's address.
    pub owner: Option<Address>,
    /// The guardian's address.
    pub guardian: Option<Address>,
    /// Total assets in the vault (in asset's smallest unit).
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: Option<f64>,
    /// Total supply of vault shares.
    pub total_supply: U256,
    /// Performance fee (as a fraction, e.g., 0.1 = 10%).
    pub fee: f64,
    /// Timelock duration in seconds.
    pub timelock: u64,
    /// Current APY (as a fraction).
    pub apy: f64,
    /// Net APY after fees (as a fraction).
    pub net_apy: f64,
    /// Current share price.
    pub share_price: U256,
    /// Allocation across markets.
    pub allocation: Vec<VaultAllocation>,
}

/// Vault allocation to a specific market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAllocation {
    /// Market unique key.
    pub market_key: String,
    /// Loan asset symbol.
    pub loan_asset_symbol: Option<String>,
    /// Loan asset address.
    pub loan_asset_address: Option<Address>,
    /// Collateral asset symbol.
    pub collateral_asset_symbol: Option<String>,
    /// Collateral asset address.
    pub collateral_asset_address: Option<Address>,
    /// Supply assets allocated.
    pub supply_assets: U256,
    /// Supply assets in USD.
    pub supply_assets_usd: Option<f64>,
    /// Supply cap.
    pub supply_cap: U256,
}

/// Vault allocator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAllocator {
    /// The allocator's address.
    pub address: Address,
}

/// Vault warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultWarning {
    /// Warning type.
    pub warning_type: String,
    /// Warning level.
    pub level: String,
}

impl VaultV1 {
    /// Create a VaultV1 from GraphQL response data.
    pub fn from_gql(
        address: &str,
        name: String,
        symbol: String,
        chain_id: i64,
        listed: bool,
        featured: bool,
        whitelisted: bool,
        asset: Asset,
        state: Option<VaultStateV1>,
        allocators: Vec<VaultAllocator>,
        warnings: Vec<VaultWarning>,
    ) -> Option<Self> {
        Some(VaultV1 {
            address: parse_address(address)?,
            name,
            symbol,
            chain: Chain::from_id(chain_id)?,
            listed,
            featured,
            whitelisted,
            asset,
            state,
            allocators,
            warnings,
        })
    }
}

impl VaultStateV1 {
    /// Create a VaultStateV1 from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        curator: Option<&str>,
        owner: Option<&str>,
        guardian: Option<&str>,
        total_assets: &str,
        total_assets_usd: Option<f64>,
        total_supply: &str,
        fee: f64,
        timelock: &str,
        apy: f64,
        net_apy: f64,
        share_price: &str,
        allocation: Vec<VaultAllocation>,
    ) -> Option<Self> {
        Some(VaultStateV1 {
            curator: curator.and_then(parse_address),
            owner: owner.and_then(parse_address),
            guardian: guardian.and_then(parse_address),
            total_assets: parse_bigint(total_assets)?,
            total_assets_usd,
            total_supply: parse_bigint(total_supply)?,
            fee,
            timelock: parse_bigint(timelock)?.to::<u64>(),
            apy,
            net_apy,
            share_price: parse_bigint(share_price)?,
            allocation,
        })
    }
}

impl VaultAllocation {
    /// Create a VaultAllocation from GraphQL response data.
    pub fn from_gql(
        market_key: String,
        loan_asset_symbol: Option<String>,
        loan_asset_address: Option<&str>,
        collateral_asset_symbol: Option<String>,
        collateral_asset_address: Option<&str>,
        supply_assets: &str,
        supply_assets_usd: Option<f64>,
        supply_cap: &str,
    ) -> Option<Self> {
        Some(VaultAllocation {
            market_key,
            loan_asset_symbol,
            loan_asset_address: loan_asset_address.and_then(parse_address),
            collateral_asset_symbol,
            collateral_asset_address: collateral_asset_address.and_then(parse_address),
            supply_assets: parse_bigint(supply_assets)?,
            supply_assets_usd,
            supply_cap: parse_bigint(supply_cap)?,
        })
    }
}

impl VaultAllocator {
    /// Create a VaultAllocator from a GraphQL address string.
    pub fn from_gql(address: &str) -> Option<Self> {
        Some(VaultAllocator {
            address: parse_address(address)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_allocator_from_gql() {
        let allocator =
            VaultAllocator::from_gql("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(
            allocator.address,
            parse_address("0x1234567890123456789012345678901234567890").unwrap()
        );
    }
}

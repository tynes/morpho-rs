//! Unified vault abstraction for both V1 and V2 vaults.

use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;
use super::chain::Chain;
use super::vault_v1::VaultV1;
use super::vault_v2::VaultV2;

/// Vault version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VaultVersion {
    /// MetaMorpho (V1) vault.
    V1,
    /// V2 vault.
    V2,
}

impl std::fmt::Display for VaultVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultVersion::V1 => write!(f, "V1"),
            VaultVersion::V2 => write!(f, "V2"),
        }
    }
}

/// Unified vault representation that works for both V1 and V2 vaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    /// Vault version (V1 or V2).
    pub version: VaultVersion,
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
    /// The vault's underlying asset.
    pub asset: Asset,
    /// The curator's address (if available).
    pub curator: Option<Address>,
    /// Total assets in the vault (in asset's smallest unit).
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: Option<f64>,
    /// Total supply of vault shares.
    pub total_supply: U256,
    /// Net APY after fees.
    pub net_apy: f64,
}

impl Vault {
    /// Create a unified Vault from a V1 vault.
    pub fn from_v1(vault: &VaultV1) -> Self {
        let (total_assets, total_assets_usd, total_supply, net_apy, curator) =
            if let Some(state) = &vault.state {
                (
                    state.total_assets,
                    state.total_assets_usd,
                    state.total_supply,
                    state.net_apy,
                    state.curator,
                )
            } else {
                (U256::ZERO, None, U256::ZERO, 0.0, None)
            };

        Vault {
            version: VaultVersion::V1,
            address: vault.address,
            name: vault.name.clone(),
            symbol: vault.symbol.clone(),
            chain: vault.chain,
            listed: vault.listed,
            asset: vault.asset.clone(),
            curator,
            total_assets,
            total_assets_usd,
            total_supply,
            net_apy,
        }
    }

    /// Create a unified Vault from a V2 vault.
    pub fn from_v2(vault: &VaultV2) -> Self {
        Vault {
            version: VaultVersion::V2,
            address: vault.address,
            name: vault.name.clone(),
            symbol: vault.symbol.clone(),
            chain: vault.chain,
            listed: vault.listed,
            asset: vault.asset.clone(),
            curator: vault.curator,
            total_assets: vault.total_assets,
            total_assets_usd: vault.total_assets_usd,
            total_supply: vault.total_supply,
            net_apy: vault.avg_net_apy.unwrap_or(0.0),
        }
    }
}

impl From<VaultV1> for Vault {
    fn from(vault: VaultV1) -> Self {
        Vault::from_v1(&vault)
    }
}

impl From<VaultV2> for Vault {
    fn from(vault: VaultV2) -> Self {
        Vault::from_v2(&vault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_version_display() {
        assert_eq!(VaultVersion::V1.to_string(), "V1");
        assert_eq!(VaultVersion::V2.to_string(), "V2");
    }
}

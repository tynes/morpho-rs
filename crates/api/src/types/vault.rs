//! Unified vault abstraction for both V1 and V2 vaults.

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;

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

/// Trait for common vault operations across V1 and V2.
pub trait Vault: Send + Sync {
    /// Returns the vault's contract address.
    fn address(&self) -> Address;
    /// Returns the vault's name.
    fn name(&self) -> &str;
    /// Returns the vault's symbol.
    fn symbol(&self) -> &str;
    /// Returns the blockchain the vault is deployed on.
    fn chain(&self) -> NamedChain;
    /// Returns the vault version (V1 or V2).
    fn version(&self) -> VaultVersion;
    /// Returns whether the vault is listed on the Morpho UI.
    fn listed(&self) -> bool;
    /// Returns whether the vault is whitelisted.
    fn whitelisted(&self) -> bool;
    /// Returns the vault's underlying asset.
    fn asset(&self) -> &Asset;
    /// Returns the curator's address (if available).
    fn curator(&self) -> Option<Address>;
    /// Returns the total assets in the vault (in asset's smallest unit).
    fn total_assets(&self) -> U256;
    /// Returns the total assets in USD.
    fn total_assets_usd(&self) -> Option<f64>;
    /// Returns the total supply of vault shares.
    fn total_supply(&self) -> U256;
    /// Returns the net APY after fees.
    fn net_apy(&self) -> f64;
    /// Returns the available liquidity.
    fn liquidity(&self) -> U256;
    /// Returns whether the vault has any critical warnings.
    fn has_critical_warnings(&self) -> bool;

    /// Clones the vault into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Vault>;
}

impl Clone for Box<dyn Vault> {
    fn clone(&self) -> Self {
        self.clone_box()
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

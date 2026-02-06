//! Asset types for Morpho vaults.

use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

use super::scalars::parse_address;

/// Represents an ERC-20 asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// The asset's contract address.
    pub address: Address,
    /// The asset's symbol (e.g., "USDC").
    pub symbol: String,
    /// The asset's name (e.g., "USD Coin").
    pub name: Option<String>,
    /// The asset's decimals.
    pub decimals: u8,
    /// Current price in USD.
    pub price_usd: Option<f64>,
}

impl Asset {
    /// Convert GraphQL response fields into an [`Asset`].
    ///
    /// Parses the hex `address` string into an [`Address`]. Returns `None` if the
    /// address is not a valid 20-byte hex string. The `decimals` parameter is
    /// truncated from `f64` to `u8` to match the GraphQL schema's numeric type.
    pub fn from_gql(
        address: &str,
        symbol: String,
        name: Option<String>,
        decimals: f64,
        price_usd: Option<f64>,
    ) -> Option<Self> {
        Some(Asset {
            address: parse_address(address)?,
            symbol,
            name,
            decimals: decimals as u8,
            price_usd,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_from_gql() {
        let asset = Asset::from_gql(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC".to_string(),
            Some("USD Coin".to_string()),
            6.0,
            Some(1.0),
        );
        assert!(asset.is_some());
        let asset = asset.unwrap();
        assert_eq!(asset.symbol, "USDC");
        assert_eq!(asset.decimals, 6);
    }
}

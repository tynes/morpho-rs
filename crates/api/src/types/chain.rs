//! Chain types and helpers for Morpho-supported networks.
//!
//! This module provides helpers for working with `alloy_chains::NamedChain` in the context
//! of Morpho's supported networks. The full `NamedChain` type is re-exported from `alloy_chains`.

use alloy_chains::NamedChain;

/// All chains supported by Morpho.
///
/// This constant lists all the blockchain networks that Morpho vaults are deployed on.
pub const SUPPORTED_CHAINS: &[NamedChain] = &[
    NamedChain::Mainnet,
    NamedChain::Base,
    NamedChain::Polygon,
    NamedChain::Arbitrum,
    NamedChain::Optimism,
    NamedChain::World,
    NamedChain::Fraxtal,
    NamedChain::Scroll,
    NamedChain::Ink,
    NamedChain::Unichain,
    NamedChain::Sonic,
    NamedChain::Mode,
    NamedChain::Corn,
    NamedChain::Katana,
    NamedChain::Etherlink,
    NamedChain::Lisk,
    NamedChain::Hyperliquid,
    NamedChain::Sei,
    NamedChain::Linea,
    NamedChain::Monad,
    NamedChain::StableMainnet,
    NamedChain::Cronos,
    NamedChain::Celo,
    NamedChain::Abstract,
    NamedChain::Sepolia,
];

/// Try to create a NamedChain from a chain ID.
///
/// This is a convenience wrapper around `NamedChain::try_from`.
pub fn chain_from_id(id: i64) -> Option<NamedChain> {
    NamedChain::try_from(id as u64).ok()
}

/// Serde helper module for serializing/deserializing NamedChain as i64 chain ID.
///
/// The Morpho GraphQL API uses i64 for chain IDs, so this module provides
/// serialization helpers to convert between NamedChain and i64.
///
/// # Example
///
/// ```ignore
/// use alloy_chains::NamedChain;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Vault {
///     #[serde(with = "chain_serde")]
///     chain: NamedChain,
/// }
/// ```
pub mod chain_serde {
    use alloy_chains::NamedChain;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(chain: &NamedChain, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let id: u64 = (*chain).into();
        serializer.serialize_i64(id as i64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NamedChain, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = i64::deserialize(deserializer)?;
        NamedChain::try_from(id as u64).map_err(|_| {
            serde::de::Error::custom(format!("Unknown chain ID: {}", id))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_from_id() {
        assert_eq!(chain_from_id(1), Some(NamedChain::Mainnet));
        assert_eq!(chain_from_id(8453), Some(NamedChain::Base));
        // Use a very large invalid ID that's unlikely to be a real chain
        assert_eq!(chain_from_id(9999999999999), None);
    }

    #[test]
    fn test_supported_chains_have_valid_ids() {
        for chain in SUPPORTED_CHAINS {
            let id: u64 = (*chain).into();
            let recovered = chain_from_id(id as i64);
            assert_eq!(recovered, Some(*chain));
        }
    }

    #[test]
    fn test_chain_serde_roundtrip() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            #[serde(with = "chain_serde")]
            chain: NamedChain,
        }

        let original = TestStruct {
            chain: NamedChain::Base,
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("8453"));

        let recovered: TestStruct = serde_json::from_str(&json).unwrap();
        assert_eq!(original, recovered);
    }
}

//! Morpho Vaults Rust API Library
//!
//! This crate provides a Rust client for querying Morpho V1 (MetaMorpho) and V2 vaults
//! via their GraphQL API.
//!
//! # Example
//!
//! ```no_run
//! use api::{VaultClient, VaultV1Client, VaultV2Client, Chain, VaultFiltersV1};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), api::ApiError> {
//!     // Use separate clients for V1 and V2
//!     let v1_client = VaultV1Client::new();
//!     let v2_client = VaultV2Client::new();
//!
//!     // Get whitelisted V1 vaults on Ethereum
//!     let v1_vaults = v1_client.get_whitelisted_vaults(Some(Chain::EthMainnet)).await?;
//!
//!     // Get V2 vaults on Base
//!     let v2_vaults = v2_client.get_vaults_by_chain(Chain::BaseMainnet).await?;
//!
//!     // Or use the combined client for unified queries
//!     let client = VaultClient::new();
//!     let all_vaults = client.get_whitelisted_vaults(Some(Chain::EthMainnet)).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod filters;
pub mod queries;
pub mod types;

// Re-export main types at crate root
pub use client::{ClientConfig, VaultClient, VaultV1Client, VaultV2Client, DEFAULT_API_URL};
pub use error::{ApiError, Result};
pub use filters::{VaultFiltersV1, VaultFiltersV2};
pub use contracts::{VaultV1TransactionClient, VaultV2TransactionClient};
pub use types::{
    Asset, Chain, MarketInfo, UserAccountOverview, UserMarketPosition, UserState,
    UserVaultPositions, UserVaultV1Position, UserVaultV2Position, Vault, VaultAdapter,
    VaultAllocation, VaultAllocator, VaultInfo, VaultPositionState, VaultReward, VaultStateV1,
    VaultV1, VaultV2, VaultV2Warning, VaultVersion, VaultWarning,
};

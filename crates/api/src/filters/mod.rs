//! Filter builders for vault queries.
//!
//! This module provides builder-pattern filter types for querying Morpho V1 and V2 vaults.
//!
//! # Architecture
//!
//! - **[`VaultFiltersV1`]** / **[`VaultFiltersV2`]** — server-side filters sent to the GraphQL API
//! - **[`VaultQueryOptionsV1`]** / **[`VaultQueryOptionsV2`]** — combine filters with ordering,
//!   pagination, and (for V2) client-side filtering
//!
//! V1 supports server-side asset and curator filtering. V2 does not — those filters are applied
//! client-side via [`VaultQueryOptionsV2`].
//!
//! # Common Query Patterns
//!
//! ## Top USDC vaults by APY on Ethereum (V1)
//!
//! ```no_run
//! # use morpho_rs_api::*;
//! # async fn example() -> Result<()> {
//! let client = MorphoClient::new();
//! let options = VaultQueryOptionsV1::new()
//!     .filters(VaultFiltersV1::new()
//!         .chain(NamedChain::Mainnet)
//!         .asset_symbols(["USDC"])
//!         .listed(true))
//!     .order_by(VaultOrderByV1::NetApy)
//!     .order_direction(OrderDirection::Desc)
//!     .limit(10);
//! let vaults = client.api().v1.get_vaults_with_options(options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Top V2 vaults by a specific curator
//!
//! ```no_run
//! # use morpho_rs_api::*;
//! # async fn example() -> Result<()> {
//! let client = MorphoClient::new();
//! let options = VaultQueryOptionsV2::new()
//!     .filters(VaultFiltersV2::new()
//!         .chain(NamedChain::Mainnet)
//!         .min_total_assets_usd(100_000.0))
//!     .curator_addresses(["0x1234567890123456789012345678901234567890"])
//!     .order_by(VaultOrderByV2::NetApy)
//!     .order_direction(OrderDirection::Desc)
//!     .limit(5);
//! let vaults = client.api().v2.get_vaults_with_options(options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## High-APY vaults across multiple chains
//!
//! ```no_run
//! # use morpho_rs_api::*;
//! # async fn example() -> Result<()> {
//! let client = MorphoClient::new();
//! let options = VaultQueryOptionsV1::new()
//!     .filters(VaultFiltersV1::new()
//!         .chains([NamedChain::Mainnet, NamedChain::Base, NamedChain::Arbitrum])
//!         .min_apy(0.05)
//!         .listed(true));
//! let vaults = client.api().v1.get_vaults_with_options(options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Shortcut helpers
//!
//! ```no_run
//! # use morpho_rs_api::*;
//! # async fn example() -> Result<()> {
//! let client = MorphoClient::new();
//!
//! // Top 10 V1 vaults by APY with custom filters
//! let filters = VaultFiltersV1::new().chain(NamedChain::Mainnet);
//! let vaults = client.api().v1.get_top_vaults_by_apy(10, Some(filters)).await?;
//!
//! // All whitelisted V2 vaults on Base
//! let vaults = client.api().v2.get_whitelisted_vaults(Some(NamedChain::Base)).await?;
//!
//! // V2 vaults by asset (client-side filtering)
//! let vaults = client.api().v2.get_vaults_by_asset("WETH", Some(NamedChain::Mainnet)).await?;
//! # Ok(())
//! # }
//! ```

pub mod query_options;
pub mod v1_filters;
pub mod v2_filters;

pub use query_options::{VaultQueryOptionsV1, VaultQueryOptionsV2};
pub use v1_filters::VaultFiltersV1;
pub use v2_filters::VaultFiltersV2;

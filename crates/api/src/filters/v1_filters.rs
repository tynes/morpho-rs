//! Filter builder for V1 vault queries.

use alloy_chains::NamedChain;

use crate::queries::v1::get_vaults_v1::VaultFilters;

/// Builder for V1 vault query filters.
///
/// Filters are applied server-side by the Morpho GraphQL API. Use the builder
/// methods to chain multiple filter criteria.
///
/// # Examples
///
/// ```
/// use morpho_rs_api::{VaultFiltersV1, NamedChain};
///
/// // Filter for listed Ethereum vaults with at least 5% APY
/// let filters = VaultFiltersV1::new()
///     .chain(NamedChain::Mainnet)
///     .listed(true)
///     .min_apy(0.05);
///
/// // Filter by multiple chains and asset
/// let filters = VaultFiltersV1::new()
///     .chains([NamedChain::Mainnet, NamedChain::Base])
///     .asset_symbols(["USDC", "WETH"]);
///
/// // Filter by curator
/// let filters = VaultFiltersV1::new()
///     .curators(["0x1234567890123456789012345678901234567890"]);
/// ```
#[derive(Debug, Clone, Default)]
pub struct VaultFiltersV1 {
    /// Filter by chain IDs.
    pub chain_ids: Option<Vec<i64>>,
    /// Filter by vault addresses.
    pub addresses: Option<Vec<String>>,
    /// Filter by listed status.
    pub listed: Option<bool>,
    /// Filter by featured status.
    pub featured: Option<bool>,
    /// Filter by curator addresses.
    pub curator_addresses: Option<Vec<String>>,
    /// Filter by owner addresses.
    pub owner_addresses: Option<Vec<String>>,
    /// Filter by asset addresses.
    pub asset_addresses: Option<Vec<String>>,
    /// Filter by asset symbols.
    pub asset_symbols: Option<Vec<String>>,
    /// Filter by minimum APY.
    pub apy_gte: Option<f64>,
    /// Filter by maximum APY.
    pub apy_lte: Option<f64>,
    /// Search query.
    pub search: Option<String>,
}

impl VaultFiltersV1 {
    /// Create a new empty filter builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by chains.
    pub fn chains<I>(mut self, chains: I) -> Self
    where
        I: IntoIterator<Item = NamedChain>,
    {
        self.chain_ids = Some(chains.into_iter().map(|c| u64::from(c) as i64).collect());
        self
    }

    /// Filter by a single chain.
    pub fn chain(mut self, chain: NamedChain) -> Self {
        self.chain_ids = Some(vec![u64::from(chain) as i64]);
        self
    }

    /// Filter by vault addresses.
    pub fn addresses<I, S>(mut self, addresses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.addresses = Some(addresses.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by listed status.
    pub fn listed(mut self, listed: bool) -> Self {
        self.listed = Some(listed);
        self
    }

    /// Filter by featured status.
    pub fn featured(mut self, featured: bool) -> Self {
        self.featured = Some(featured);
        self
    }

    /// Filter by curator addresses.
    pub fn curators<I, S>(mut self, curators: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.curator_addresses = Some(curators.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by owner addresses.
    pub fn owners<I, S>(mut self, owners: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.owner_addresses = Some(owners.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by asset addresses.
    pub fn asset_addresses<I, S>(mut self, addresses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.asset_addresses = Some(addresses.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by asset symbols.
    pub fn asset_symbols<I, S>(mut self, symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.asset_symbols = Some(symbols.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by minimum APY.
    pub fn min_apy(mut self, apy: f64) -> Self {
        self.apy_gte = Some(apy);
        self
    }

    /// Filter by maximum APY.
    pub fn max_apy(mut self, apy: f64) -> Self {
        self.apy_lte = Some(apy);
        self
    }

    /// Filter by search query.
    pub fn search<S: Into<String>>(mut self, query: S) -> Self {
        self.search = Some(query.into());
        self
    }

    /// Convert to GraphQL filter input type.
    pub fn to_gql(&self) -> VaultFilters {
        VaultFilters {
            chain_id_in: self.chain_ids.clone().map(|ids| ids.into_iter().collect()),
            address_in: self.addresses.clone(),
            listed: self.listed,
            featured: self.featured,
            curator_address_in: self.curator_addresses.clone(),
            owner_address_in: self.owner_addresses.clone(),
            asset_address_in: self.asset_addresses.clone(),
            asset_symbol_in: self.asset_symbols.clone(),
            apy_gte: self.apy_gte,
            apy_lte: self.apy_lte,
            search: self.search.clone(),
            // Set other fields to None
            id_in: None,
            address_not_in: None,
            creator_address_in: None,
            factory_address_in: None,
            symbol_in: None,
            asset_id_in: None,
            asset_tags_in: None,
            market_unique_key_in: None,
            country_code: None,
            curator_in: None,
            fee_gte: None,
            fee_lte: None,
            net_apy_gte: None,
            net_apy_lte: None,
            total_assets_gte: None,
            total_assets_lte: None,
            total_assets_usd_gte: None,
            total_assets_usd_lte: None,
            total_supply_gte: None,
            total_supply_lte: None,
            public_allocator_fee_lte: None,
            public_allocator_fee_usd_lte: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filters = VaultFiltersV1::new()
            .chain(NamedChain::Mainnet)
            .listed(true)
            .min_apy(0.05);

        assert_eq!(filters.chain_ids, Some(vec![1]));
        assert_eq!(filters.listed, Some(true));
        assert_eq!(filters.apy_gte, Some(0.05));
    }

    #[test]
    fn test_filter_multiple_chains() {
        let filters = VaultFiltersV1::new().chains([NamedChain::Mainnet, NamedChain::Base]);

        assert_eq!(filters.chain_ids, Some(vec![1, 8453]));
    }

    #[test]
    fn test_filter_v1_to_gql_all_fields() {
        let filters = VaultFiltersV1::new()
            .chain(NamedChain::Mainnet)
            .addresses(["0x1234567890123456789012345678901234567890"])
            .listed(true)
            .featured(false)
            .curators(["0xCurator"])
            .owners(["0xOwner"])
            .asset_addresses(["0xAsset"])
            .asset_symbols(["USDC", "WETH"])
            .min_apy(0.01)
            .max_apy(0.50)
            .search("steakhouse");

        let gql = filters.to_gql();

        assert_eq!(gql.chain_id_in, Some(vec![1]));
        assert_eq!(
            gql.address_in,
            Some(vec!["0x1234567890123456789012345678901234567890".to_string()])
        );
        assert_eq!(gql.listed, Some(true));
        assert_eq!(gql.featured, Some(false));
        assert_eq!(
            gql.curator_address_in,
            Some(vec!["0xCurator".to_string()])
        );
        assert_eq!(gql.owner_address_in, Some(vec!["0xOwner".to_string()]));
        assert_eq!(gql.asset_address_in, Some(vec!["0xAsset".to_string()]));
        assert_eq!(
            gql.asset_symbol_in,
            Some(vec!["USDC".to_string(), "WETH".to_string()])
        );
        assert_eq!(gql.apy_gte, Some(0.01));
        assert_eq!(gql.apy_lte, Some(0.50));
        assert_eq!(gql.search, Some("steakhouse".to_string()));
    }

    #[test]
    fn test_filter_v1_apy_range() {
        let filters = VaultFiltersV1::new().min_apy(0.05).max_apy(0.15);

        let gql = filters.to_gql();
        assert_eq!(gql.apy_gte, Some(0.05));
        assert_eq!(gql.apy_lte, Some(0.15));
    }

    #[test]
    fn test_filter_v1_chain_conversion() {
        let filters_mainnet = VaultFiltersV1::new().chain(NamedChain::Mainnet);
        assert_eq!(filters_mainnet.chain_ids, Some(vec![1]));

        let filters_base = VaultFiltersV1::new().chain(NamedChain::Base);
        assert_eq!(filters_base.chain_ids, Some(vec![8453]));

        let filters_arbitrum = VaultFiltersV1::new().chain(NamedChain::Arbitrum);
        assert_eq!(filters_arbitrum.chain_ids, Some(vec![42161]));
    }

    #[test]
    fn test_filter_v1_default() {
        let filters = VaultFiltersV1::default();
        assert!(filters.chain_ids.is_none());
        assert!(filters.addresses.is_none());
        assert!(filters.listed.is_none());
        assert!(filters.featured.is_none());
        assert!(filters.curator_addresses.is_none());
        assert!(filters.owner_addresses.is_none());
        assert!(filters.asset_addresses.is_none());
        assert!(filters.asset_symbols.is_none());
        assert!(filters.apy_gte.is_none());
        assert!(filters.apy_lte.is_none());
        assert!(filters.search.is_none());
    }

    #[test]
    fn test_filter_v1_to_gql_empty() {
        let filters = VaultFiltersV1::new();
        let gql = filters.to_gql();

        assert!(gql.chain_id_in.is_none());
        assert!(gql.address_in.is_none());
        assert!(gql.listed.is_none());
        assert!(gql.featured.is_none());
    }
}

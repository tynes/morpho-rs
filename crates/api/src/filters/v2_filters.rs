//! Filter builder for V2 vault queries.

use alloy_chains::NamedChain;

use crate::queries::v2::get_vaults_v2::VaultV2sFilters;

/// Builder for V2 vault query filters.
///
/// Filters are applied server-side by the Morpho GraphQL API. Use the builder
/// methods to chain multiple filter criteria.
///
/// Note: The V2 API does not support server-side asset or curator filtering.
/// For those, use [`VaultQueryOptionsV2::asset_symbols`](crate::VaultQueryOptionsV2::asset_symbols),
/// [`VaultQueryOptionsV2::asset_addresses`](crate::VaultQueryOptionsV2::asset_addresses),
/// or [`VaultQueryOptionsV2::curator_addresses`](crate::VaultQueryOptionsV2::curator_addresses)
/// which apply client-side filtering.
///
/// # Examples
///
/// ```
/// use morpho_rs_api::{VaultFiltersV2, NamedChain};
///
/// // Filter for listed Ethereum vaults with at least $1M TVL
/// let filters = VaultFiltersV2::new()
///     .chain(NamedChain::Mainnet)
///     .listed(true)
///     .min_total_assets_usd(1_000_000.0);
///
/// // Filter by APY range and liquidity
/// let filters = VaultFiltersV2::new()
///     .min_apy(0.03)
///     .max_apy(0.15)
///     .min_liquidity_usd(100_000.0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct VaultFiltersV2 {
    /// Filter by chain IDs.
    pub chain_ids: Option<Vec<i64>>,
    /// Filter by vault addresses.
    pub addresses: Option<Vec<String>>,
    /// Filter by listed status.
    pub listed: Option<bool>,
    /// Filter by minimum total assets in USD.
    pub total_assets_usd_gte: Option<f64>,
    /// Filter by maximum total assets in USD.
    pub total_assets_usd_lte: Option<f64>,
    /// Filter by minimum liquidity in USD.
    pub liquidity_usd_gte: Option<f64>,
    /// Filter by maximum liquidity in USD.
    pub liquidity_usd_lte: Option<f64>,
    /// Filter by minimum APY.
    pub apy_gte: Option<f64>,
    /// Filter by maximum APY.
    pub apy_lte: Option<f64>,
}

impl VaultFiltersV2 {
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

    /// Filter by minimum total assets in USD.
    pub fn min_total_assets_usd(mut self, usd: f64) -> Self {
        self.total_assets_usd_gte = Some(usd);
        self
    }

    /// Filter by maximum total assets in USD.
    pub fn max_total_assets_usd(mut self, usd: f64) -> Self {
        self.total_assets_usd_lte = Some(usd);
        self
    }

    /// Filter by minimum liquidity in USD.
    pub fn min_liquidity_usd(mut self, usd: f64) -> Self {
        self.liquidity_usd_gte = Some(usd);
        self
    }

    /// Filter by maximum liquidity in USD.
    pub fn max_liquidity_usd(mut self, usd: f64) -> Self {
        self.liquidity_usd_lte = Some(usd);
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

    /// Convert to GraphQL filter input type.
    pub fn to_gql(&self) -> VaultV2sFilters {
        VaultV2sFilters {
            chain_id_in: self.chain_ids.clone().map(|ids| ids.into_iter().collect()),
            address_in: self.addresses.clone(),
            listed: self.listed,
            total_assets_usd_gte: self.total_assets_usd_gte,
            total_assets_usd_lte: self.total_assets_usd_lte,
            liquidity_usd_gte: self.liquidity_usd_gte,
            liquidity_usd_lte: self.liquidity_usd_lte,
            apy_gte: self.apy_gte,
            apy_lte: self.apy_lte,
            // Set other fields to None
            total_assets_gte: None,
            total_assets_lte: None,
            total_supply_gte: None,
            total_supply_lte: None,
            liquidity_gte: None,
            liquidity_lte: None,
            net_apy_gte: None,
            net_apy_lte: None,
            real_assets_gte: None,
            real_assets_lte: None,
            real_assets_usd_gte: None,
            real_assets_usd_lte: None,
            idle_assets_gte: None,
            idle_assets_lte: None,
            idle_assets_usd_gte: None,
            idle_assets_usd_lte: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_builder() {
        let filters = VaultFiltersV2::new()
            .chain(NamedChain::Mainnet)
            .listed(true)
            .min_total_assets_usd(1_000_000.0);

        assert_eq!(filters.chain_ids, Some(vec![1]));
        assert_eq!(filters.listed, Some(true));
        assert_eq!(filters.total_assets_usd_gte, Some(1_000_000.0));
    }

    #[test]
    fn test_filter_multiple_chains() {
        let filters = VaultFiltersV2::new().chains([NamedChain::Mainnet, NamedChain::Base]);

        assert_eq!(filters.chain_ids, Some(vec![1, 8453]));
    }

    #[test]
    fn test_filter_v2_to_gql_all_fields() {
        let filters = VaultFiltersV2::new()
            .chain(NamedChain::Mainnet)
            .addresses(["0x1234567890123456789012345678901234567890"])
            .listed(true)
            .min_total_assets_usd(100_000.0)
            .max_total_assets_usd(10_000_000.0)
            .min_liquidity_usd(50_000.0)
            .max_liquidity_usd(5_000_000.0)
            .min_apy(0.01)
            .max_apy(0.50);

        let gql = filters.to_gql();

        assert_eq!(gql.chain_id_in, Some(vec![1]));
        assert_eq!(
            gql.address_in,
            Some(vec!["0x1234567890123456789012345678901234567890".to_string()])
        );
        assert_eq!(gql.listed, Some(true));
        assert_eq!(gql.total_assets_usd_gte, Some(100_000.0));
        assert_eq!(gql.total_assets_usd_lte, Some(10_000_000.0));
        assert_eq!(gql.liquidity_usd_gte, Some(50_000.0));
        assert_eq!(gql.liquidity_usd_lte, Some(5_000_000.0));
        assert_eq!(gql.apy_gte, Some(0.01));
        assert_eq!(gql.apy_lte, Some(0.50));
    }

    #[test]
    fn test_filter_v2_usd_range_filters() {
        let filters = VaultFiltersV2::new()
            .min_total_assets_usd(1_000_000.0)
            .max_total_assets_usd(50_000_000.0)
            .min_liquidity_usd(100_000.0)
            .max_liquidity_usd(10_000_000.0);

        let gql = filters.to_gql();
        assert_eq!(gql.total_assets_usd_gte, Some(1_000_000.0));
        assert_eq!(gql.total_assets_usd_lte, Some(50_000_000.0));
        assert_eq!(gql.liquidity_usd_gte, Some(100_000.0));
        assert_eq!(gql.liquidity_usd_lte, Some(10_000_000.0));
    }

    #[test]
    fn test_filter_v2_apy_range() {
        let filters = VaultFiltersV2::new().min_apy(0.03).max_apy(0.12);

        let gql = filters.to_gql();
        assert_eq!(gql.apy_gte, Some(0.03));
        assert_eq!(gql.apy_lte, Some(0.12));
    }

    #[test]
    fn test_filter_v2_chain_conversion() {
        let filters_mainnet = VaultFiltersV2::new().chain(NamedChain::Mainnet);
        assert_eq!(filters_mainnet.chain_ids, Some(vec![1]));

        let filters_base = VaultFiltersV2::new().chain(NamedChain::Base);
        assert_eq!(filters_base.chain_ids, Some(vec![8453]));
    }

    #[test]
    fn test_filter_v2_default() {
        let filters = VaultFiltersV2::default();
        assert!(filters.chain_ids.is_none());
        assert!(filters.addresses.is_none());
        assert!(filters.listed.is_none());
        assert!(filters.total_assets_usd_gte.is_none());
        assert!(filters.total_assets_usd_lte.is_none());
        assert!(filters.liquidity_usd_gte.is_none());
        assert!(filters.liquidity_usd_lte.is_none());
        assert!(filters.apy_gte.is_none());
        assert!(filters.apy_lte.is_none());
    }

    #[test]
    fn test_filter_v2_to_gql_empty() {
        let filters = VaultFiltersV2::new();
        let gql = filters.to_gql();

        assert!(gql.chain_id_in.is_none());
        assert!(gql.address_in.is_none());
        assert!(gql.listed.is_none());
    }
}

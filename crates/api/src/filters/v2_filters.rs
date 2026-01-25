//! Filter builder for V2 vault queries.

use alloy_chains::NamedChain;

use crate::queries::v2::get_vaults_v2::VaultV2sFilters;
use crate::queries::v2_simulation::get_vaults_v2_for_simulation::VaultV2sFilters as SimVaultV2Filters;

/// Builder for V2 vault query filters.
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
            chain_id_in: self.chain_ids.clone().map(|ids| ids.into_iter().map(|id| id as i64).collect()),
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

    /// Convert to GraphQL filter input type for simulation queries.
    pub fn to_gql_sim(&self) -> SimVaultV2Filters {
        SimVaultV2Filters {
            chain_id_in: self.chain_ids.clone().map(|ids| ids.into_iter().map(|id| id as i64).collect()),
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
}

//! Query options for vault queries, combining filters, ordering, and pagination.

use crate::filters::{VaultFiltersV1, VaultFiltersV2};
use crate::types::ordering::{OrderDirection, VaultOrderByV1, VaultOrderByV2};

/// Options for V1 vault queries.
///
/// Combines filters, ordering, and pagination in a builder pattern.
///
/// # Example
///
/// ```
/// use morpho_rs_api::{VaultQueryOptionsV1, VaultFiltersV1, VaultOrderByV1, OrderDirection, NamedChain};
///
/// let options = VaultQueryOptionsV1::new()
///     .filters(VaultFiltersV1::new()
///         .chain(NamedChain::Mainnet)
///         .asset_symbols(["USDC"]))
///     .order_by(VaultOrderByV1::NetApy)
///     .order_direction(OrderDirection::Desc)
///     .limit(10);
/// ```
#[derive(Debug, Clone, Default)]
pub struct VaultQueryOptionsV1 {
    /// Filters to apply to the query.
    pub filters: Option<VaultFiltersV1>,
    /// Field to order results by.
    pub order_by: Option<VaultOrderByV1>,
    /// Direction to order results.
    pub order_direction: Option<OrderDirection>,
    /// Maximum number of results to return.
    pub limit: Option<i64>,
}

impl VaultQueryOptionsV1 {
    /// Create a new empty query options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the filters for the query.
    pub fn filters(mut self, filters: VaultFiltersV1) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the field to order by.
    pub fn order_by(mut self, order_by: VaultOrderByV1) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Set the order direction.
    pub fn order_direction(mut self, direction: OrderDirection) -> Self {
        self.order_direction = Some(direction);
        self
    }

    /// Set the maximum number of results.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Create options for fetching top vaults by APY.
    pub fn top_by_apy(limit: i64) -> Self {
        Self::new()
            .order_by(VaultOrderByV1::NetApy)
            .order_direction(OrderDirection::Desc)
            .limit(limit)
    }

    /// Create options for fetching top vaults by total assets.
    pub fn top_by_tvl(limit: i64) -> Self {
        Self::new()
            .order_by(VaultOrderByV1::TotalAssetsUsd)
            .order_direction(OrderDirection::Desc)
            .limit(limit)
    }
}

/// Options for V2 vault queries.
///
/// Combines filters, ordering, and pagination in a builder pattern.
/// Also supports client-side asset filtering since the V2 API doesn't support it server-side.
///
/// # Example
///
/// ```
/// use morpho_rs_api::{VaultQueryOptionsV2, VaultFiltersV2, VaultOrderByV2, OrderDirection, NamedChain};
///
/// let options = VaultQueryOptionsV2::new()
///     .filters(VaultFiltersV2::new()
///         .chain(NamedChain::Mainnet))
///     .order_by(VaultOrderByV2::NetApy)
///     .order_direction(OrderDirection::Desc)
///     .asset_symbols(["USDC"])  // Client-side filtering
///     .limit(10);
/// ```
#[derive(Debug, Clone, Default)]
pub struct VaultQueryOptionsV2 {
    /// Filters to apply to the query.
    pub filters: Option<VaultFiltersV2>,
    /// Field to order results by.
    pub order_by: Option<VaultOrderByV2>,
    /// Direction to order results.
    pub order_direction: Option<OrderDirection>,
    /// Maximum number of results to return.
    pub limit: Option<i64>,
    /// Asset addresses to filter by (client-side, API doesn't support this).
    pub asset_addresses: Option<Vec<String>>,
    /// Asset symbols to filter by (client-side, API doesn't support this).
    pub asset_symbols: Option<Vec<String>>,
}

impl VaultQueryOptionsV2 {
    /// Create a new empty query options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the filters for the query.
    pub fn filters(mut self, filters: VaultFiltersV2) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the field to order by.
    pub fn order_by(mut self, order_by: VaultOrderByV2) -> Self {
        self.order_by = Some(order_by);
        self
    }

    /// Set the order direction.
    pub fn order_direction(mut self, direction: OrderDirection) -> Self {
        self.order_direction = Some(direction);
        self
    }

    /// Set the maximum number of results.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Filter by asset addresses (client-side).
    ///
    /// Note: The Morpho V2 API doesn't support asset filtering server-side,
    /// so this filtering is done client-side after fetching results.
    pub fn asset_addresses<I, S>(mut self, addresses: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.asset_addresses = Some(addresses.into_iter().map(Into::into).collect());
        self
    }

    /// Filter by asset symbols (client-side).
    ///
    /// Note: The Morpho V2 API doesn't support asset filtering server-side,
    /// so this filtering is done client-side after fetching results.
    pub fn asset_symbols<I, S>(mut self, symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.asset_symbols = Some(symbols.into_iter().map(Into::into).collect());
        self
    }

    /// Create options for fetching top vaults by APY.
    pub fn top_by_apy(limit: i64) -> Self {
        Self::new()
            .order_by(VaultOrderByV2::NetApy)
            .order_direction(OrderDirection::Desc)
            .limit(limit)
    }

    /// Create options for fetching top vaults by total assets.
    pub fn top_by_tvl(limit: i64) -> Self {
        Self::new()
            .order_by(VaultOrderByV2::TotalAssetsUsd)
            .order_direction(OrderDirection::Desc)
            .limit(limit)
    }

    /// Create options for fetching top vaults by liquidity.
    pub fn top_by_liquidity(limit: i64) -> Self {
        Self::new()
            .order_by(VaultOrderByV2::LiquidityUsd)
            .order_direction(OrderDirection::Desc)
            .limit(limit)
    }

    /// Check if any client-side asset filtering is configured.
    pub fn has_asset_filter(&self) -> bool {
        self.asset_addresses.is_some() || self.asset_symbols.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_chains::NamedChain;

    // V1 Options Tests

    #[test]
    fn test_v1_options_default() {
        let options = VaultQueryOptionsV1::default();
        assert!(options.filters.is_none());
        assert!(options.order_by.is_none());
        assert!(options.order_direction.is_none());
        assert!(options.limit.is_none());
    }

    #[test]
    fn test_v1_options_builder() {
        let options = VaultQueryOptionsV1::new()
            .filters(VaultFiltersV1::new().chain(NamedChain::Mainnet))
            .order_by(VaultOrderByV1::NetApy)
            .order_direction(OrderDirection::Desc)
            .limit(25);

        assert!(options.filters.is_some());
        assert_eq!(options.order_by, Some(VaultOrderByV1::NetApy));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
        assert_eq!(options.limit, Some(25));
    }

    #[test]
    fn test_v1_options_top_by_apy() {
        let options = VaultQueryOptionsV1::top_by_apy(10);

        assert_eq!(options.limit, Some(10));
        assert_eq!(options.order_by, Some(VaultOrderByV1::NetApy));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
        assert!(options.filters.is_none());
    }

    #[test]
    fn test_v1_options_top_by_tvl() {
        let options = VaultQueryOptionsV1::top_by_tvl(5);

        assert_eq!(options.limit, Some(5));
        assert_eq!(options.order_by, Some(VaultOrderByV1::TotalAssetsUsd));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
    }

    // V2 Options Tests

    #[test]
    fn test_v2_options_default() {
        let options = VaultQueryOptionsV2::default();
        assert!(options.filters.is_none());
        assert!(options.order_by.is_none());
        assert!(options.order_direction.is_none());
        assert!(options.limit.is_none());
        assert!(options.asset_addresses.is_none());
        assert!(options.asset_symbols.is_none());
    }

    #[test]
    fn test_v2_options_builder() {
        let options = VaultQueryOptionsV2::new()
            .filters(VaultFiltersV2::new().chain(NamedChain::Mainnet))
            .order_by(VaultOrderByV2::NetApy)
            .order_direction(OrderDirection::Desc)
            .asset_symbols(["USDC", "WETH"])
            .limit(25);

        assert!(options.filters.is_some());
        assert_eq!(options.order_by, Some(VaultOrderByV2::NetApy));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
        assert_eq!(options.limit, Some(25));
        assert_eq!(
            options.asset_symbols,
            Some(vec!["USDC".to_string(), "WETH".to_string()])
        );
    }

    #[test]
    fn test_v2_options_has_asset_filter() {
        let no_filter = VaultQueryOptionsV2::new();
        assert!(!no_filter.has_asset_filter());

        let with_symbols = VaultQueryOptionsV2::new().asset_symbols(["USDC"]);
        assert!(with_symbols.has_asset_filter());

        let with_addresses = VaultQueryOptionsV2::new()
            .asset_addresses(["0x1234567890123456789012345678901234567890"]);
        assert!(with_addresses.has_asset_filter());

        let with_both = VaultQueryOptionsV2::new()
            .asset_symbols(["USDC"])
            .asset_addresses(["0x1234567890123456789012345678901234567890"]);
        assert!(with_both.has_asset_filter());
    }

    #[test]
    fn test_v2_options_top_by_apy() {
        let options = VaultQueryOptionsV2::top_by_apy(10);

        assert_eq!(options.limit, Some(10));
        assert_eq!(options.order_by, Some(VaultOrderByV2::NetApy));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
    }

    #[test]
    fn test_v2_options_top_by_tvl() {
        let options = VaultQueryOptionsV2::top_by_tvl(5);

        assert_eq!(options.limit, Some(5));
        assert_eq!(options.order_by, Some(VaultOrderByV2::TotalAssetsUsd));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
    }

    #[test]
    fn test_v2_options_top_by_liquidity() {
        let options = VaultQueryOptionsV2::top_by_liquidity(15);

        assert_eq!(options.limit, Some(15));
        assert_eq!(options.order_by, Some(VaultOrderByV2::LiquidityUsd));
        assert_eq!(options.order_direction, Some(OrderDirection::Desc));
    }

    #[test]
    fn test_v2_options_asset_addresses() {
        let options = VaultQueryOptionsV2::new()
            .asset_addresses([
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            ]);

        assert_eq!(
            options.asset_addresses,
            Some(vec![
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            ])
        );
    }
}

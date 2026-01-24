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
/// use morpho_rs_api::{VaultQueryOptionsV1, VaultFiltersV1, VaultOrderByV1, OrderDirection, Chain};
///
/// let options = VaultQueryOptionsV1::new()
///     .filters(VaultFiltersV1::new()
///         .chain(Chain::EthMainnet)
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
/// use morpho_rs_api::{VaultQueryOptionsV2, VaultFiltersV2, VaultOrderByV2, OrderDirection, Chain};
///
/// let options = VaultQueryOptionsV2::new()
///     .filters(VaultFiltersV2::new()
///         .chain(Chain::EthMainnet))
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

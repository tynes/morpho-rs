//! Ordering types for vault queries.

use crate::queries::simulation::get_vaults_for_simulation::{
    OrderDirection as OrderDirectionSim, VaultOrderBy as VaultOrderBySimGql,
};
use crate::queries::v1::get_vaults_v1::{
    OrderDirection as OrderDirectionV1, VaultOrderBy as VaultOrderByV1Gql,
};
use crate::queries::v2::get_vaults_v2::{
    OrderDirection as OrderDirectionV2, VaultV2OrderBy as VaultV2OrderByGql,
};

/// Order direction for queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderDirection {
    /// Ascending order.
    Asc,
    /// Descending order (default).
    #[default]
    Desc,
}

impl OrderDirection {
    /// Convert to V1 GraphQL order direction.
    pub(crate) fn to_gql_v1(self) -> OrderDirectionV1 {
        match self {
            OrderDirection::Asc => OrderDirectionV1::Asc,
            OrderDirection::Desc => OrderDirectionV1::Desc,
        }
    }

    /// Convert to V2 GraphQL order direction.
    pub(crate) fn to_gql_v2(self) -> OrderDirectionV2 {
        match self {
            OrderDirection::Asc => OrderDirectionV2::Asc,
            OrderDirection::Desc => OrderDirectionV2::Desc,
        }
    }

    /// Convert to simulation GraphQL order direction.
    pub(crate) fn to_gql_sim(self) -> OrderDirectionSim {
        match self {
            OrderDirection::Asc => OrderDirectionSim::Asc,
            OrderDirection::Desc => OrderDirectionSim::Desc,
        }
    }
}

/// Order by options for V1 vaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VaultOrderByV1 {
    /// Order by vault address.
    Address,
    /// Order by total assets.
    TotalAssets,
    /// Order by total assets in USD.
    TotalAssetsUsd,
    /// Order by total supply.
    TotalSupply,
    /// Order by fee.
    Fee,
    /// Order by APY (gross).
    Apy,
    /// Order by net APY (after fees).
    #[default]
    NetApy,
    /// Order by vault name.
    Name,
    /// Order by curator.
    Curator,
    /// Order by average APY.
    AvgApy,
    /// Order by average net APY.
    AvgNetApy,
    /// Order by daily APY.
    DailyApy,
    /// Order by daily net APY.
    DailyNetApy,
    /// Order by Credora risk score.
    CredoraRiskScore,
}

impl VaultOrderByV1 {
    /// Convert to GraphQL order by type.
    pub(crate) fn to_gql(self) -> VaultOrderByV1Gql {
        match self {
            VaultOrderByV1::Address => VaultOrderByV1Gql::Address,
            VaultOrderByV1::TotalAssets => VaultOrderByV1Gql::TotalAssets,
            VaultOrderByV1::TotalAssetsUsd => VaultOrderByV1Gql::TotalAssetsUsd,
            VaultOrderByV1::TotalSupply => VaultOrderByV1Gql::TotalSupply,
            VaultOrderByV1::Fee => VaultOrderByV1Gql::Fee,
            VaultOrderByV1::Apy => VaultOrderByV1Gql::Apy,
            VaultOrderByV1::NetApy => VaultOrderByV1Gql::NetApy,
            VaultOrderByV1::Name => VaultOrderByV1Gql::Name,
            VaultOrderByV1::Curator => VaultOrderByV1Gql::Curator,
            VaultOrderByV1::AvgApy => VaultOrderByV1Gql::AvgApy,
            VaultOrderByV1::AvgNetApy => VaultOrderByV1Gql::AvgNetApy,
            VaultOrderByV1::DailyApy => VaultOrderByV1Gql::DailyApy,
            VaultOrderByV1::DailyNetApy => VaultOrderByV1Gql::DailyNetApy,
            VaultOrderByV1::CredoraRiskScore => VaultOrderByV1Gql::CredoraRiskScore,
        }
    }

    /// Convert to simulation GraphQL order by type.
    pub(crate) fn to_gql_sim(self) -> VaultOrderBySimGql {
        match self {
            VaultOrderByV1::Address => VaultOrderBySimGql::Address,
            VaultOrderByV1::TotalAssets => VaultOrderBySimGql::TotalAssets,
            VaultOrderByV1::TotalAssetsUsd => VaultOrderBySimGql::TotalAssetsUsd,
            VaultOrderByV1::TotalSupply => VaultOrderBySimGql::TotalSupply,
            VaultOrderByV1::Fee => VaultOrderBySimGql::Fee,
            VaultOrderByV1::Apy => VaultOrderBySimGql::Apy,
            VaultOrderByV1::NetApy => VaultOrderBySimGql::NetApy,
            VaultOrderByV1::Name => VaultOrderBySimGql::Name,
            VaultOrderByV1::Curator => VaultOrderBySimGql::Curator,
            VaultOrderByV1::AvgApy => VaultOrderBySimGql::AvgApy,
            VaultOrderByV1::AvgNetApy => VaultOrderBySimGql::AvgNetApy,
            VaultOrderByV1::DailyApy => VaultOrderBySimGql::DailyApy,
            VaultOrderByV1::DailyNetApy => VaultOrderBySimGql::DailyNetApy,
            VaultOrderByV1::CredoraRiskScore => VaultOrderBySimGql::CredoraRiskScore,
        }
    }
}

/// Order by options for V2 vaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VaultOrderByV2 {
    /// Order by vault address.
    Address,
    /// Order by total assets.
    TotalAssets,
    /// Order by total assets in USD.
    TotalAssetsUsd,
    /// Order by total supply.
    TotalSupply,
    /// Order by liquidity.
    Liquidity,
    /// Order by liquidity in USD.
    LiquidityUsd,
    /// Order by APY (gross).
    Apy,
    /// Order by net APY (after fees).
    #[default]
    NetApy,
    /// Order by real assets.
    RealAssets,
    /// Order by real assets in USD.
    RealAssetsUsd,
    /// Order by idle assets.
    IdleAssets,
    /// Order by idle assets in USD.
    IdleAssetsUsd,
}

impl VaultOrderByV2 {
    /// Convert to GraphQL order by type.
    pub(crate) fn to_gql(self) -> VaultV2OrderByGql {
        match self {
            VaultOrderByV2::Address => VaultV2OrderByGql::Address,
            VaultOrderByV2::TotalAssets => VaultV2OrderByGql::TotalAssets,
            VaultOrderByV2::TotalAssetsUsd => VaultV2OrderByGql::TotalAssetsUsd,
            VaultOrderByV2::TotalSupply => VaultV2OrderByGql::TotalSupply,
            VaultOrderByV2::Liquidity => VaultV2OrderByGql::Liquidity,
            VaultOrderByV2::LiquidityUsd => VaultV2OrderByGql::LiquidityUsd,
            VaultOrderByV2::Apy => VaultV2OrderByGql::Apy,
            VaultOrderByV2::NetApy => VaultV2OrderByGql::NetApy,
            VaultOrderByV2::RealAssets => VaultV2OrderByGql::RealAssets,
            VaultOrderByV2::RealAssetsUsd => VaultV2OrderByGql::RealAssetsUsd,
            VaultOrderByV2::IdleAssets => VaultV2OrderByGql::IdleAssets,
            VaultOrderByV2::IdleAssetsUsd => VaultV2OrderByGql::IdleAssetsUsd,
        }
    }
}

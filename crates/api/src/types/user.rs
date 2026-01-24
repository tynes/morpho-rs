//! User types for Morpho API.

use alloy_chains::NamedChain;
use alloy_primitives::{Address, U256};
use serde::{Deserialize, Serialize};

use super::chain::{chain_from_id, chain_serde};
use super::scalars::{parse_address, parse_bigint};

/// Basic vault info for positions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultInfo {
    /// The vault's contract address.
    pub address: Address,
    /// The vault's name.
    pub name: String,
    /// The vault's symbol.
    pub symbol: String,
    /// The blockchain the vault is deployed on.
    #[serde(with = "chain_serde")]
    pub chain: NamedChain,
}

/// State of a vault position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultPositionState {
    /// Number of shares held.
    pub shares: U256,
    /// Assets value.
    pub assets: Option<U256>,
    /// Assets value in USD.
    pub assets_usd: Option<f64>,
    /// Profit and loss.
    pub pnl: Option<U256>,
    /// Profit and loss in USD.
    pub pnl_usd: Option<f64>,
    /// Return on equity.
    pub roe: Option<f64>,
    /// Return on equity in USD terms.
    pub roe_usd: Option<f64>,
}

/// A user's position in a V1 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserVaultV1Position {
    /// Position ID.
    pub id: String,
    /// Number of shares held.
    pub shares: U256,
    /// Assets value.
    pub assets: U256,
    /// Assets value in USD.
    pub assets_usd: Option<f64>,
    /// The vault.
    pub vault: VaultInfo,
    /// Position state.
    pub state: Option<VaultPositionState>,
}

/// A user's position in a V2 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserVaultV2Position {
    /// Position ID.
    pub id: String,
    /// Number of shares held.
    pub shares: U256,
    /// Assets value.
    pub assets: U256,
    /// Assets value in USD.
    pub assets_usd: Option<f64>,
    /// Profit and loss.
    pub pnl: Option<U256>,
    /// Profit and loss in USD.
    pub pnl_usd: Option<f64>,
    /// Return on equity.
    pub roe: Option<f64>,
    /// Return on equity in USD terms.
    pub roe_usd: Option<f64>,
    /// The vault.
    pub vault: VaultInfo,
}

/// Basic market info for positions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketInfo {
    /// Market unique key.
    pub unique_key: String,
    /// Loan asset symbol.
    pub loan_asset_symbol: Option<String>,
    /// Loan asset address.
    pub loan_asset_address: Option<Address>,
    /// Collateral asset symbol.
    pub collateral_asset_symbol: Option<String>,
    /// Collateral asset address.
    pub collateral_asset_address: Option<Address>,
}

/// A user's position in a market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMarketPosition {
    /// Position ID.
    pub id: String,
    /// Supply shares.
    pub supply_shares: U256,
    /// Supply assets.
    pub supply_assets: U256,
    /// Supply assets in USD.
    pub supply_assets_usd: Option<f64>,
    /// Borrow shares.
    pub borrow_shares: U256,
    /// Borrow assets.
    pub borrow_assets: U256,
    /// Borrow assets in USD.
    pub borrow_assets_usd: Option<f64>,
    /// Collateral amount.
    pub collateral: U256,
    /// Collateral value in USD.
    pub collateral_usd: Option<f64>,
    /// Health factor.
    pub health_factor: Option<f64>,
    /// The market.
    pub market: MarketInfo,
}

/// User's aggregated state across all positions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserState {
    /// Total PnL from V1 vaults in USD.
    pub vaults_pnl_usd: f64,
    /// Total ROE from V1 vaults in USD.
    pub vaults_roe_usd: f64,
    /// Total assets in V1 vaults in USD.
    pub vaults_assets_usd: f64,
    /// Total PnL from V2 vaults in USD.
    pub vault_v2s_pnl_usd: f64,
    /// Total ROE from V2 vaults in USD.
    pub vault_v2s_roe_usd: f64,
    /// Total assets in V2 vaults in USD.
    pub vault_v2s_assets_usd: f64,
    /// Total PnL from markets in USD.
    pub markets_pnl_usd: f64,
    /// Total ROE from markets in USD.
    pub markets_roe_usd: f64,
    /// Total supply PnL from markets in USD.
    pub markets_supply_pnl_usd: f64,
    /// Total supply ROE from markets in USD.
    pub markets_supply_roe_usd: f64,
    /// Total borrow PnL from markets in USD.
    pub markets_borrow_pnl_usd: f64,
    /// Total borrow ROE from markets in USD.
    pub markets_borrow_roe_usd: f64,
    /// Total collateral PnL from markets in USD.
    pub markets_collateral_pnl_usd: f64,
    /// Total collateral ROE from markets in USD.
    pub markets_collateral_roe_usd: f64,
    /// Total margin PnL from markets in USD.
    pub markets_margin_pnl_usd: f64,
    /// Total margin ROE from markets in USD.
    pub markets_margin_roe_usd: f64,
    /// Total collateral in markets in USD.
    pub markets_collateral_usd: f64,
    /// Total supply assets in markets in USD.
    pub markets_supply_assets_usd: f64,
    /// Total borrow assets in markets in USD.
    pub markets_borrow_assets_usd: f64,
    /// Total margin in markets in USD.
    pub markets_margin_usd: f64,
}

/// All vault positions for a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserVaultPositions {
    /// User's address.
    pub address: Address,
    /// V1 vault positions.
    pub vault_positions: Vec<UserVaultV1Position>,
    /// V2 vault positions.
    pub vault_v2_positions: Vec<UserVaultV2Position>,
}

/// Complete account overview for a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAccountOverview {
    /// User's address.
    pub address: Address,
    /// Aggregated state.
    pub state: UserState,
    /// V1 vault positions.
    pub vault_positions: Vec<UserVaultV1Position>,
    /// V2 vault positions.
    pub vault_v2_positions: Vec<UserVaultV2Position>,
    /// Market positions.
    pub market_positions: Vec<UserMarketPosition>,
}

impl VaultInfo {
    /// Create a VaultInfo from GraphQL response data.
    pub fn from_gql(address: &str, name: String, symbol: String, chain_id: i64) -> Option<Self> {
        Some(VaultInfo {
            address: parse_address(address)?,
            name,
            symbol,
            chain: chain_from_id(chain_id)?,
        })
    }
}

impl VaultPositionState {
    /// Create a VaultPositionState from GraphQL response data.
    pub fn from_gql(
        shares: &str,
        assets: Option<&str>,
        assets_usd: Option<f64>,
        pnl: Option<&str>,
        pnl_usd: Option<f64>,
        roe: Option<f64>,
        roe_usd: Option<f64>,
    ) -> Option<Self> {
        Some(VaultPositionState {
            shares: parse_bigint(shares)?,
            assets: assets.and_then(parse_bigint),
            assets_usd,
            pnl: pnl.and_then(parse_bigint),
            pnl_usd,
            roe,
            roe_usd,
        })
    }
}

impl UserVaultV1Position {
    /// Create a UserVaultV1Position from GraphQL response data.
    pub fn from_gql(
        id: String,
        shares: &str,
        assets: &str,
        assets_usd: Option<f64>,
        vault: VaultInfo,
        state: Option<VaultPositionState>,
    ) -> Option<Self> {
        Some(UserVaultV1Position {
            id,
            shares: parse_bigint(shares)?,
            assets: parse_bigint(assets)?,
            assets_usd,
            vault,
            state,
        })
    }
}

impl UserVaultV2Position {
    /// Create a UserVaultV2Position from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        id: String,
        shares: &str,
        assets: &str,
        assets_usd: Option<f64>,
        pnl: Option<&str>,
        pnl_usd: Option<f64>,
        roe: Option<f64>,
        roe_usd: Option<f64>,
        vault: VaultInfo,
    ) -> Option<Self> {
        Some(UserVaultV2Position {
            id,
            shares: parse_bigint(shares)?,
            assets: parse_bigint(assets)?,
            assets_usd,
            pnl: pnl.and_then(parse_bigint),
            pnl_usd,
            roe,
            roe_usd,
            vault,
        })
    }
}

impl MarketInfo {
    /// Create a MarketInfo from GraphQL response data.
    pub fn from_gql(
        unique_key: String,
        loan_asset_symbol: Option<String>,
        loan_asset_address: Option<&str>,
        collateral_asset_symbol: Option<String>,
        collateral_asset_address: Option<&str>,
    ) -> Self {
        MarketInfo {
            unique_key,
            loan_asset_symbol,
            loan_asset_address: loan_asset_address.and_then(parse_address),
            collateral_asset_symbol,
            collateral_asset_address: collateral_asset_address.and_then(parse_address),
        }
    }
}

impl UserMarketPosition {
    /// Create a UserMarketPosition from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        id: String,
        supply_shares: &str,
        supply_assets: &str,
        supply_assets_usd: Option<f64>,
        borrow_shares: &str,
        borrow_assets: &str,
        borrow_assets_usd: Option<f64>,
        collateral: &str,
        collateral_usd: Option<f64>,
        health_factor: Option<f64>,
        market: MarketInfo,
    ) -> Option<Self> {
        Some(UserMarketPosition {
            id,
            supply_shares: parse_bigint(supply_shares)?,
            supply_assets: parse_bigint(supply_assets)?,
            supply_assets_usd,
            borrow_shares: parse_bigint(borrow_shares)?,
            borrow_assets: parse_bigint(borrow_assets)?,
            borrow_assets_usd,
            collateral: parse_bigint(collateral)?,
            collateral_usd,
            health_factor,
            market,
        })
    }
}

impl UserState {
    /// Create a UserState from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        vaults_pnl_usd: f64,
        vaults_roe_usd: f64,
        vaults_assets_usd: f64,
        vault_v2s_pnl_usd: f64,
        vault_v2s_roe_usd: f64,
        vault_v2s_assets_usd: f64,
        markets_pnl_usd: f64,
        markets_roe_usd: f64,
        markets_supply_pnl_usd: f64,
        markets_supply_roe_usd: f64,
        markets_borrow_pnl_usd: f64,
        markets_borrow_roe_usd: f64,
        markets_collateral_pnl_usd: f64,
        markets_collateral_roe_usd: f64,
        markets_margin_pnl_usd: f64,
        markets_margin_roe_usd: f64,
        markets_collateral_usd: f64,
        markets_supply_assets_usd: f64,
        markets_borrow_assets_usd: f64,
        markets_margin_usd: f64,
    ) -> Self {
        UserState {
            vaults_pnl_usd,
            vaults_roe_usd,
            vaults_assets_usd,
            vault_v2s_pnl_usd,
            vault_v2s_roe_usd,
            vault_v2s_assets_usd,
            markets_pnl_usd,
            markets_roe_usd,
            markets_supply_pnl_usd,
            markets_supply_roe_usd,
            markets_borrow_pnl_usd,
            markets_borrow_roe_usd,
            markets_collateral_pnl_usd,
            markets_collateral_roe_usd,
            markets_margin_pnl_usd,
            markets_margin_roe_usd,
            markets_collateral_usd,
            markets_supply_assets_usd,
            markets_borrow_assets_usd,
            markets_margin_usd,
        }
    }
}

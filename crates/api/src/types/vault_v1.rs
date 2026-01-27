//! V1 (MetaMorpho) vault types.

use alloy_chains::NamedChain;
use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;
use super::chain::{chain_from_id, chain_serde};
use super::scalars::{parse_address, parse_bigint};

/// Represents a Morpho V1 (MetaMorpho) vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV1 {
    /// The vault's contract address.
    pub address: Address,
    /// The vault's name.
    pub name: String,
    /// The vault's symbol.
    pub symbol: String,
    /// The blockchain the vault is deployed on.
    #[serde(with = "chain_serde")]
    pub chain: NamedChain,
    /// Whether the vault is listed on the Morpho UI.
    pub listed: bool,
    /// Whether the vault is featured.
    pub featured: bool,
    /// Whether the vault is whitelisted.
    pub whitelisted: bool,
    /// The vault's underlying asset.
    pub asset: Asset,
    /// Current vault state.
    pub state: Option<VaultStateV1>,
    /// Vault allocators.
    pub allocators: Vec<VaultAllocator>,
    /// Vault warnings.
    pub warnings: Vec<VaultWarning>,
}

/// Current state of a V1 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultStateV1 {
    /// The curator's address.
    pub curator: Option<Address>,
    /// The owner's address.
    pub owner: Option<Address>,
    /// The guardian's address.
    pub guardian: Option<Address>,
    /// Total assets in the vault (in asset's smallest unit).
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: Option<f64>,
    /// Total supply of vault shares.
    pub total_supply: U256,
    /// Performance fee (as a fraction, e.g., 0.1 = 10%).
    pub fee: f64,
    /// Timelock duration in seconds.
    pub timelock: u64,
    /// Current APY (as a fraction).
    pub apy: f64,
    /// Net APY after fees (as a fraction).
    pub net_apy: f64,
    /// Current share price.
    pub share_price: U256,
    /// Allocation across markets.
    pub allocation: Vec<VaultAllocation>,
}

/// Market state data needed for simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketStateV1 {
    /// Market unique identifier (32-byte hash).
    pub id: B256,
    /// Total loan assets supplied to the market.
    pub total_supply_assets: U256,
    /// Total loan assets borrowed from the market.
    pub total_borrow_assets: U256,
    /// Total supply shares representing lender positions.
    pub total_supply_shares: U256,
    /// Total borrow shares representing borrower debt.
    pub total_borrow_shares: U256,
    /// Timestamp of last interest accrual.
    pub last_update: u64,
    /// Protocol fee (WAD-scaled, e.g., 0.1 WAD = 10%).
    pub fee: U256,
    /// Rate at target utilization for Adaptive Curve IRM (None for other IRMs).
    pub rate_at_target: Option<U256>,
    /// Oracle price (collateral/loan, scaled by 1e36). None if unavailable.
    pub price: Option<U256>,
    /// Liquidation LTV (WAD-scaled).
    pub lltv: U256,
    /// Available liquidity (total_supply_assets - total_borrow_assets).
    pub liquidity: U256,
}

/// Vault allocation to a specific market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAllocation {
    /// Market unique key.
    pub market_key: String,
    /// Loan asset symbol.
    pub loan_asset_symbol: Option<String>,
    /// Loan asset address.
    pub loan_asset_address: Option<Address>,
    /// Collateral asset symbol.
    pub collateral_asset_symbol: Option<String>,
    /// Collateral asset address.
    pub collateral_asset_address: Option<Address>,
    /// Supply assets allocated.
    pub supply_assets: U256,
    /// Supply assets in USD.
    pub supply_assets_usd: Option<f64>,
    /// Supply cap.
    pub supply_cap: U256,
    /// Whether this market is enabled for the vault.
    pub enabled: bool,
    /// Position in the supply queue (None if not in queue).
    pub supply_queue_index: Option<i32>,
    /// Position in the withdraw queue (None if not in queue).
    pub withdraw_queue_index: Option<i32>,
    /// Full market state for simulation.
    pub market_state: Option<MarketStateV1>,
}

/// Vault allocator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAllocator {
    /// The allocator's address.
    pub address: Address,
}

/// Vault warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultWarning {
    /// Warning type.
    pub warning_type: String,
    /// Warning level.
    pub level: String,
}

impl VaultV1 {
    /// Create a VaultV1 from GraphQL response data.
    pub fn from_gql(
        address: &str,
        name: String,
        symbol: String,
        chain_id: i64,
        listed: bool,
        featured: bool,
        whitelisted: bool,
        asset: Asset,
        state: Option<VaultStateV1>,
        allocators: Vec<VaultAllocator>,
        warnings: Vec<VaultWarning>,
    ) -> Option<Self> {
        Some(VaultV1 {
            address: parse_address(address)?,
            name,
            symbol,
            chain: chain_from_id(chain_id)?,
            listed,
            featured,
            whitelisted,
            asset,
            state,
            allocators,
            warnings,
        })
    }
}

impl VaultStateV1 {
    /// Create a VaultStateV1 from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        curator: Option<&str>,
        owner: Option<&str>,
        guardian: Option<&str>,
        total_assets: &str,
        total_assets_usd: Option<f64>,
        total_supply: &str,
        fee: f64,
        timelock: &str,
        apy: f64,
        net_apy: f64,
        share_price: &str,
        allocation: Vec<VaultAllocation>,
    ) -> Option<Self> {
        Some(VaultStateV1 {
            curator: curator.and_then(parse_address),
            owner: owner.and_then(parse_address),
            guardian: guardian.and_then(parse_address),
            total_assets: parse_bigint(total_assets)?,
            total_assets_usd,
            total_supply: parse_bigint(total_supply)?,
            fee,
            timelock: parse_bigint(timelock)?.to::<u64>(),
            apy,
            net_apy,
            share_price: parse_bigint(share_price)?,
            allocation,
        })
    }
}

impl VaultAllocation {
    /// Create a VaultAllocation from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        market_key: String,
        loan_asset_symbol: Option<String>,
        loan_asset_address: Option<&str>,
        collateral_asset_symbol: Option<String>,
        collateral_asset_address: Option<&str>,
        supply_assets: &str,
        supply_assets_usd: Option<f64>,
        supply_cap: &str,
        enabled: bool,
        supply_queue_index: Option<i32>,
        withdraw_queue_index: Option<i32>,
        market_state: Option<MarketStateV1>,
    ) -> Option<Self> {
        Some(VaultAllocation {
            market_key,
            loan_asset_symbol,
            loan_asset_address: loan_asset_address.and_then(parse_address),
            collateral_asset_symbol,
            collateral_asset_address: collateral_asset_address.and_then(parse_address),
            supply_assets: parse_bigint(supply_assets)?,
            supply_assets_usd,
            supply_cap: parse_bigint(supply_cap)?,
            enabled,
            supply_queue_index,
            withdraw_queue_index,
            market_state,
        })
    }
}

impl VaultAllocator {
    /// Create a VaultAllocator from a GraphQL address string.
    pub fn from_gql(address: &str) -> Option<Self> {
        Some(VaultAllocator {
            address: parse_address(address)?,
        })
    }
}

// Simulation conversion methods (only available with "sim" feature)
#[cfg(feature = "sim")]
mod sim_conversion {
    use super::*;
    use morpho_rs_sim::{Market, Vault, VaultMarketConfig, VaultSimulation};
    use std::collections::HashMap;

    impl VaultV1 {
        /// Convert this vault to a VaultSimulation for APY and deposit/withdrawal calculations.
        ///
        /// Returns `None` if the vault has no state or if required market data is missing.
        ///
        /// # Example
        ///
        /// ```ignore
        /// use morpho_rs_api::VaultV1;
        ///
        /// let vault: VaultV1 = /* fetch from API */;
        /// if let Some(simulation) = vault.to_vault_simulation() {
        ///     let apy = simulation.get_net_apy(timestamp)?;
        ///     let (new_sim, shares) = simulation.simulate_deposit(amount, timestamp)?;
        /// }
        /// ```
        pub fn to_vault_simulation(&self) -> Option<VaultSimulation> {
            let state = self.state.as_ref()?;

            // Build supply and withdraw queues by sorting allocations by queue index
            let mut supply_queue_items: Vec<_> = state
                .allocation
                .iter()
                .filter_map(|a| {
                    let market_id = a.market_state.as_ref()?.id;
                    a.supply_queue_index.map(|idx| (idx, market_id))
                })
                .collect();
            supply_queue_items.sort_by_key(|(idx, _)| *idx);
            let supply_queue: Vec<B256> = supply_queue_items.into_iter().map(|(_, id)| id).collect();

            let mut withdraw_queue_items: Vec<_> = state
                .allocation
                .iter()
                .filter_map(|a| {
                    let market_id = a.market_state.as_ref()?.id;
                    a.withdraw_queue_index.map(|idx| (idx, market_id))
                })
                .collect();
            withdraw_queue_items.sort_by_key(|(idx, _)| *idx);
            let withdraw_queue: Vec<B256> = withdraw_queue_items.into_iter().map(|(_, id)| id).collect();

            // Build allocations HashMap
            let mut allocations = HashMap::new();
            for alloc in &state.allocation {
                if let Some(market_state) = &alloc.market_state {
                    let market_id = market_state.id;
                    allocations.insert(
                        market_id,
                        VaultMarketConfig {
                            market_id,
                            cap: alloc.supply_cap,
                            supply_assets: alloc.supply_assets,
                            enabled: alloc.enabled,
                            public_allocator_config: None, // Not available from API
                        },
                    );
                }
            }

            // Build markets HashMap
            let mut markets = HashMap::new();
            for alloc in &state.allocation {
                if let Some(ms) = &alloc.market_state {
                    let market = Market::new_with_oracle(
                        ms.id,
                        ms.total_supply_assets,
                        ms.total_borrow_assets,
                        ms.total_supply_shares,
                        ms.total_borrow_shares,
                        ms.last_update,
                        ms.fee,
                        ms.rate_at_target,
                        ms.price,
                        ms.lltv,
                    );
                    markets.insert(ms.id, market);
                }
            }

            // Convert fee from fraction to WAD-scaled
            // API returns fee as fraction (0.1 = 10%), sim expects WAD (0.1 * 1e18)
            let fee_wad = U256::from((state.fee * 1e18) as u128);

            let vault = Vault {
                address: self.address,
                asset_decimals: self.asset.decimals,
                fee: fee_wad,
                total_assets: state.total_assets,
                total_supply: state.total_supply,
                last_total_assets: state.total_assets, // Assume current state is synced
                supply_queue,
                withdraw_queue,
                allocations,
                owner: state.owner.unwrap_or(Address::ZERO),
                public_allocator_config: None, // Not available from API
            };

            Some(VaultSimulation::new(vault, markets))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_allocator_from_gql() {
        let allocator =
            VaultAllocator::from_gql("0x1234567890123456789012345678901234567890").unwrap();
        assert_eq!(
            allocator.address,
            parse_address("0x1234567890123456789012345678901234567890").unwrap()
        );
    }
}

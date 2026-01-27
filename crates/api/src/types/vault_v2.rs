//! V2 vault types.

use alloy_chains::NamedChain;
use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

use super::asset::Asset;
use super::chain::{chain_from_id, chain_serde};
use super::scalars::{parse_address, parse_bigint};

/// Represents a Morpho V2 vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV2 {
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
    /// Whether the vault is whitelisted.
    pub whitelisted: bool,
    /// The vault's underlying asset.
    pub asset: Asset,
    /// The curator's address.
    pub curator: Option<Address>,
    /// The owner's address.
    pub owner: Option<Address>,
    /// Total assets in the vault (in asset's smallest unit).
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: Option<f64>,
    /// Total supply of vault shares.
    pub total_supply: U256,
    /// Current share price.
    pub share_price: Option<f64>,
    /// Performance fee (as a fraction).
    pub performance_fee: Option<f64>,
    /// Management fee (as a fraction).
    pub management_fee: Option<f64>,
    /// Average APY.
    pub avg_apy: Option<f64>,
    /// Average net APY after fees.
    pub avg_net_apy: Option<f64>,
    /// Current APY.
    pub apy: Option<f64>,
    /// Current net APY.
    pub net_apy: Option<f64>,
    /// Liquidity available.
    pub liquidity: U256,
    /// Liquidity in USD.
    pub liquidity_usd: Option<f64>,
    /// Vault adapters.
    pub adapters: Vec<VaultAdapter>,
    /// Vault rewards.
    pub rewards: Vec<VaultReward>,
    /// Vault warnings.
    pub warnings: Vec<VaultV2Warning>,
}

/// Market state data needed for simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketStateV2 {
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
}

/// Position in a MorphoMarketV1 adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphoMarketPosition {
    /// Supply assets in this position.
    pub supply_assets: U256,
    /// Supply shares in this position.
    pub supply_shares: U256,
    /// Market unique key.
    pub market_id: B256,
    /// Full market state for simulation.
    pub market_state: Option<MarketStateV2>,
}

/// Allocation in a MetaMorpho adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaMorphoAllocation {
    /// Supply assets in this allocation.
    pub supply_assets: U256,
    /// Supply cap for this market.
    pub supply_cap: U256,
    /// Whether this market is enabled.
    pub enabled: bool,
    /// Position in the supply queue.
    pub supply_queue_index: Option<i32>,
    /// Position in the withdraw queue.
    pub withdraw_queue_index: Option<i32>,
    /// Market unique key.
    pub market_id: B256,
    /// Full market state for simulation.
    pub market_state: Option<MarketStateV2>,
}

/// Adapter-specific data for simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VaultAdapterData {
    /// MorphoMarketV1 adapter positions.
    MorphoMarketV1 {
        /// Positions in Morpho markets.
        positions: Vec<MorphoMarketPosition>,
    },
    /// MetaMorpho adapter allocations.
    MetaMorpho {
        /// Allocations to underlying markets.
        allocations: Vec<MetaMorphoAllocation>,
    },
}

/// Vault adapter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAdapter {
    /// Adapter ID.
    pub id: String,
    /// Adapter contract address.
    pub address: Address,
    /// Adapter type (e.g., MetaMorpho, MorphoMarketV1).
    pub adapter_type: String,
    /// Assets held by this adapter.
    pub assets: U256,
    /// Assets held in USD.
    pub assets_usd: Option<f64>,
    /// Adapter-specific data for simulation.
    pub data: Option<VaultAdapterData>,
}

/// Vault reward configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultReward {
    /// Reward asset address.
    pub asset_address: Address,
    /// Reward asset symbol.
    pub asset_symbol: String,
    /// Supply APR from rewards.
    pub supply_apr: Option<f64>,
    /// Yearly supply tokens.
    pub yearly_supply_tokens: Option<f64>,
}

/// V2 vault warning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultV2Warning {
    /// Warning type.
    pub warning_type: String,
    /// Warning level.
    pub level: String,
}

impl VaultV2 {
    /// Create a VaultV2 from GraphQL response data.
    #[allow(clippy::too_many_arguments)]
    pub fn from_gql(
        address: &str,
        name: String,
        symbol: String,
        chain_id: i64,
        listed: bool,
        whitelisted: bool,
        asset: Asset,
        curator: Option<&str>,
        owner: Option<&str>,
        total_assets: &str,
        total_assets_usd: Option<f64>,
        total_supply: &str,
        share_price: Option<f64>,
        performance_fee: Option<f64>,
        management_fee: Option<f64>,
        avg_apy: Option<f64>,
        avg_net_apy: Option<f64>,
        apy: Option<f64>,
        net_apy: Option<f64>,
        liquidity: &str,
        liquidity_usd: Option<f64>,
        adapters: Vec<VaultAdapter>,
        rewards: Vec<VaultReward>,
        warnings: Vec<VaultV2Warning>,
    ) -> Option<Self> {
        Some(VaultV2 {
            address: parse_address(address)?,
            name,
            symbol,
            chain: chain_from_id(chain_id)?,
            listed,
            whitelisted,
            asset,
            curator: curator.and_then(parse_address),
            owner: owner.and_then(parse_address),
            total_assets: parse_bigint(total_assets)?,
            total_assets_usd,
            total_supply: parse_bigint(total_supply)?,
            share_price,
            performance_fee,
            management_fee,
            avg_apy,
            avg_net_apy,
            apy,
            net_apy,
            liquidity: parse_bigint(liquidity).unwrap_or(U256::ZERO),
            liquidity_usd,
            adapters,
            rewards,
            warnings,
        })
    }
}

impl VaultAdapter {
    /// Create a VaultAdapter from GraphQL response data.
    pub fn from_gql(
        id: String,
        address: &str,
        adapter_type: String,
        assets: &str,
        assets_usd: Option<f64>,
        data: Option<VaultAdapterData>,
    ) -> Option<Self> {
        Some(VaultAdapter {
            id,
            address: parse_address(address)?,
            adapter_type,
            assets: parse_bigint(assets).unwrap_or(U256::ZERO),
            assets_usd,
            data,
        })
    }
}

impl VaultReward {
    /// Create a VaultReward from GraphQL response data.
    pub fn from_gql(
        asset_address: &str,
        asset_symbol: String,
        supply_apr: Option<f64>,
        yearly_supply_tokens: Option<f64>,
    ) -> Option<Self> {
        Some(VaultReward {
            asset_address: parse_address(asset_address)?,
            asset_symbol,
            supply_apr,
            yearly_supply_tokens,
        })
    }
}

// Implement Vault trait for VaultV2
use super::vault::{Vault, VaultVersion};

impl Vault for VaultV2 {
    fn address(&self) -> Address {
        self.address
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn chain(&self) -> NamedChain {
        self.chain
    }

    fn version(&self) -> VaultVersion {
        VaultVersion::V2
    }

    fn listed(&self) -> bool {
        self.listed
    }

    fn whitelisted(&self) -> bool {
        self.whitelisted
    }

    fn asset(&self) -> &super::asset::Asset {
        &self.asset
    }

    fn curator(&self) -> Option<Address> {
        self.curator
    }

    fn total_assets(&self) -> U256 {
        self.total_assets
    }

    fn total_assets_usd(&self) -> Option<f64> {
        self.total_assets_usd
    }

    fn total_supply(&self) -> U256 {
        self.total_supply
    }

    fn net_apy(&self) -> f64 {
        self.avg_net_apy.unwrap_or(0.0)
    }

    fn liquidity(&self) -> U256 {
        self.liquidity
    }

    fn has_critical_warnings(&self) -> bool {
        self.warnings.iter().any(|w| w.level == "CRITICAL")
    }
}

// Simulation conversion methods (only available with "sim" feature)
#[cfg(feature = "sim")]
mod sim_conversion {
    use super::*;
    use morpho_rs_sim::{Market, Vault, VaultMarketConfig, VaultSimulation};
    use std::collections::HashMap;

    impl VaultV2 {
        /// Convert this vault to a VaultSimulation for APY and deposit/withdrawal calculations.
        ///
        /// This method extracts allocation data from MetaMorpho adapters. V2 vaults may have
        /// multiple adapter types; only MetaMorpho adapters contain the queue-based allocation
        /// structure needed for simulation.
        ///
        /// Returns `None` if no MetaMorpho adapter with valid allocation data is found.
        ///
        /// # Example
        ///
        /// ```ignore
        /// use morpho_rs_api::VaultV2;
        ///
        /// let vault: VaultV2 = /* fetch from API */;
        /// if let Some(simulation) = vault.to_vault_simulation() {
        ///     let apy = simulation.get_net_apy(timestamp)?;
        ///     let (new_sim, shares) = simulation.simulate_deposit(amount, timestamp)?;
        /// }
        /// ```
        pub fn to_vault_simulation(&self) -> Option<VaultSimulation> {
            // Find MetaMorpho adapter with allocations
            let meta_morpho_allocations = self.adapters.iter().find_map(|adapter| {
                match &adapter.data {
                    Some(VaultAdapterData::MetaMorpho { allocations }) => Some(allocations.clone()),
                    _ => None,
                }
            })?;

            // Build supply and withdraw queues by sorting allocations by queue index
            let mut supply_queue_items: Vec<_> = meta_morpho_allocations
                .iter()
                .filter_map(|a| {
                    a.supply_queue_index.map(|idx| (idx, a.market_id))
                })
                .collect();
            supply_queue_items.sort_by_key(|(idx, _)| *idx);
            let supply_queue: Vec<B256> = supply_queue_items.into_iter().map(|(_, id)| id).collect();

            let mut withdraw_queue_items: Vec<_> = meta_morpho_allocations
                .iter()
                .filter_map(|a| {
                    a.withdraw_queue_index.map(|idx| (idx, a.market_id))
                })
                .collect();
            withdraw_queue_items.sort_by_key(|(idx, _)| *idx);
            let withdraw_queue: Vec<B256> = withdraw_queue_items.into_iter().map(|(_, id)| id).collect();

            // Build allocations HashMap
            let mut allocations = HashMap::new();
            for alloc in &meta_morpho_allocations {
                allocations.insert(
                    alloc.market_id,
                    VaultMarketConfig {
                        market_id: alloc.market_id,
                        cap: alloc.supply_cap,
                        supply_assets: alloc.supply_assets,
                        enabled: alloc.enabled,
                        public_allocator_config: None, // Not available from API
                    },
                );
            }

            // Build markets HashMap
            let mut markets = HashMap::new();
            for alloc in &meta_morpho_allocations {
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
            let fee = self.performance_fee.unwrap_or(0.0);
            let fee_wad = U256::from((fee * 1e18) as u128);

            let vault = Vault {
                address: self.address,
                asset_decimals: self.asset.decimals,
                fee: fee_wad,
                total_assets: self.total_assets,
                total_supply: self.total_supply,
                last_total_assets: self.total_assets, // Assume current state is synced
                supply_queue,
                withdraw_queue,
                allocations,
                owner: self.owner.unwrap_or(Address::ZERO),
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
    fn test_vault_adapter_from_gql() {
        let adapter = VaultAdapter::from_gql(
            "adapter-1".to_string(),
            "0x1234567890123456789012345678901234567890",
            "MetaMorpho".to_string(),
            "1000000000000000000",
            Some(1000.0),
            None,
        )
        .unwrap();
        assert_eq!(adapter.id, "adapter-1");
        assert_eq!(adapter.adapter_type, "MetaMorpho");
        assert_eq!(adapter.assets, U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_vault_reward_from_gql() {
        let reward = VaultReward::from_gql(
            "0x1234567890123456789012345678901234567890",
            "MORPHO".to_string(),
            Some(0.05),
            Some(1000.0),
        )
        .unwrap();
        assert_eq!(reward.asset_symbol, "MORPHO");
        assert_eq!(reward.supply_apr, Some(0.05));
    }
}

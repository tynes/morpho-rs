//! Types for vault simulation data fetched from the Morpho API.
//!
//! These types contain all data needed to build `VaultSimulation` objects
//! from `morpho-rs-sim`, enabling offline APY simulation.

use alloy_primitives::{Address, B256, U256};
use serde::{Deserialize, Serialize};

/// Market state data needed for simulation.
///
/// Contains all fields required to construct a `morpho_rs_sim::Market`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketStateForSim {
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

/// Vault allocation with simulation fields.
///
/// Extends basic allocation data with queue indices and enabled flag
/// needed for simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAllocationForSim {
    /// Market unique identifier.
    pub market_id: B256,
    /// Current supply to this market from the vault.
    pub supply_assets: U256,
    /// Maximum supply cap for this market.
    pub supply_cap: U256,
    /// Whether this market is enabled for the vault.
    pub enabled: bool,
    /// Position in the supply queue (None if not in queue).
    pub supply_queue_index: Option<i32>,
    /// Position in the withdraw queue (None if not in queue).
    pub withdraw_queue_index: Option<i32>,
}

/// Complete data needed to build a `VaultSimulation`.
///
/// This struct contains all vault and market data required to construct
/// a fully functional `morpho_rs_sim::VaultSimulation` for offline APY
/// calculations and deposit/withdraw simulations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultSimulationData {
    /// Vault contract address.
    pub address: Address,
    /// Decimals of the underlying asset.
    pub asset_decimals: u8,
    /// Performance fee (WAD-scaled, e.g., 0.1 WAD = 10%).
    pub fee: U256,
    /// Total assets under management.
    pub total_assets: U256,
    /// Total assets in USD (for sorting/filtering).
    pub total_assets_usd: Option<f64>,
    /// Total vault shares outstanding.
    pub total_supply: U256,
    /// Vault allocations with queue indices.
    pub allocations: Vec<VaultAllocationForSim>,
    /// Market state data for all markets the vault allocates to.
    pub markets: Vec<MarketStateForSim>,
}

impl VaultSimulationData {
    /// Get ordered supply queue (sorted by supply_queue_index).
    pub fn supply_queue(&self) -> Vec<B256> {
        let mut indexed: Vec<_> = self
            .allocations
            .iter()
            .filter_map(|a| a.supply_queue_index.map(|idx| (idx, a.market_id)))
            .collect();
        indexed.sort_by_key(|(idx, _)| *idx);
        indexed.into_iter().map(|(_, id)| id).collect()
    }

    /// Get ordered withdraw queue (sorted by withdraw_queue_index).
    pub fn withdraw_queue(&self) -> Vec<B256> {
        let mut indexed: Vec<_> = self
            .allocations
            .iter()
            .filter_map(|a| a.withdraw_queue_index.map(|idx| (idx, a.market_id)))
            .collect();
        indexed.sort_by_key(|(idx, _)| *idx);
        indexed.into_iter().map(|(_, id)| id).collect()
    }
}

/// Conversion implementations when the `sim` feature is enabled.
#[cfg(feature = "sim")]
mod sim_conversions {
    use super::*;
    use morpho_rs_sim::{Market, Vault, VaultMarketConfig, VaultSimulation};
    use std::collections::HashMap;

    impl MarketStateForSim {
        /// Convert to a morpho-rs-sim Market.
        pub fn to_sim_market(&self) -> Market {
            Market::new_with_oracle(
                self.id,
                self.total_supply_assets,
                self.total_borrow_assets,
                self.total_supply_shares,
                self.total_borrow_shares,
                self.last_update,
                self.fee,
                self.rate_at_target,
                self.price,
                self.lltv,
            )
        }
    }

    impl VaultSimulationData {
        /// Convert to a morpho-rs-sim VaultSimulation.
        ///
        /// # Example
        ///
        /// ```ignore
        /// use morpho_rs_api::VaultV1Client;
        ///
        /// let client = VaultV1Client::new();
        /// let data = client.get_vault_for_simulation("0x...", NamedChain::Mainnet).await?;
        /// let sim = data.to_vault_simulation()?;
        ///
        /// // Now you can simulate deposits
        /// let impact = sim.deposit_impact(amount, timestamp)?;
        /// ```
        pub fn to_vault_simulation(&self) -> Option<VaultSimulation> {
            // Build markets map
            let mut markets: HashMap<B256, Market> = HashMap::new();
            for market in &self.markets {
                markets.insert(market.id, market.to_sim_market());
            }

            // Build allocations map
            let mut allocations: HashMap<B256, VaultMarketConfig> = HashMap::new();
            for alloc in &self.allocations {
                allocations.insert(
                    alloc.market_id,
                    VaultMarketConfig {
                        market_id: alloc.market_id,
                        cap: alloc.supply_cap,
                        supply_assets: alloc.supply_assets,
                        enabled: alloc.enabled,
                        public_allocator_config: None,
                    },
                );
            }

            // Build queues
            let supply_queue = self.supply_queue();
            let withdraw_queue = self.withdraw_queue();

            // Build vault
            let vault = Vault {
                address: self.address,
                asset_decimals: self.asset_decimals,
                fee: self.fee,
                total_assets: self.total_assets,
                total_supply: self.total_supply,
                last_total_assets: self.total_assets, // Assume no pending interest
                supply_queue,
                withdraw_queue,
                allocations,
                owner: Address::ZERO, // Not available from API
                public_allocator_config: None,
            };

            Some(VaultSimulation::new(vault, markets))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    fn create_test_market_state(id: B256) -> MarketStateForSim {
        MarketStateForSim {
            id,
            total_supply_assets: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_borrow_assets: U256::from(800_000_000_000_000_000_000_000u128),
            total_supply_shares: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_borrow_shares: U256::from(800_000_000_000_000_000_000_000u128),
            last_update: 1704067200,
            fee: U256::from(100_000_000_000_000_000u64), // 10%
            rate_at_target: Some(U256::from(1_268_391_679u64)),
            price: Some(U256::from(1_000_000_000_000_000_000_000_000_000_000_000_000u128)),
            lltv: U256::from(800_000_000_000_000_000u64), // 80%
        }
    }

    fn create_test_allocation(market_id: B256, supply_idx: Option<i32>, withdraw_idx: Option<i32>) -> VaultAllocationForSim {
        VaultAllocationForSim {
            market_id,
            supply_assets: U256::from(500_000_000_000_000_000_000_000u128),
            supply_cap: U256::from(1_000_000_000_000_000_000_000_000u128),
            enabled: true,
            supply_queue_index: supply_idx,
            withdraw_queue_index: withdraw_idx,
        }
    }

    fn create_test_vault_simulation_data() -> VaultSimulationData {
        let market1_id = FixedBytes::from_slice(&[1u8; 32]);
        let market2_id = FixedBytes::from_slice(&[2u8; 32]);
        let market3_id = FixedBytes::from_slice(&[3u8; 32]);

        VaultSimulationData {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(50_000_000_000_000_000u64), // 5%
            total_assets: U256::from(1_500_000_000_000_000_000_000_000u128),
            total_assets_usd: Some(1_500_000.0),
            total_supply: U256::from(1_500_000_000_000_000_000_000_000u128),
            allocations: vec![
                create_test_allocation(market1_id, Some(0), Some(2)),
                create_test_allocation(market2_id, Some(2), Some(0)),
                create_test_allocation(market3_id, Some(1), Some(1)),
            ],
            markets: vec![
                create_test_market_state(market1_id),
                create_test_market_state(market2_id),
                create_test_market_state(market3_id),
            ],
        }
    }

    #[test]
    fn test_supply_queue_ordering() {
        let data = create_test_vault_simulation_data();
        let supply_queue = data.supply_queue();

        // Should be ordered by supply_queue_index: market1 (0), market3 (1), market2 (2)
        assert_eq!(supply_queue.len(), 3);
        assert_eq!(supply_queue[0], FixedBytes::from_slice(&[1u8; 32]));
        assert_eq!(supply_queue[1], FixedBytes::from_slice(&[3u8; 32]));
        assert_eq!(supply_queue[2], FixedBytes::from_slice(&[2u8; 32]));
    }

    #[test]
    fn test_withdraw_queue_ordering() {
        let data = create_test_vault_simulation_data();
        let withdraw_queue = data.withdraw_queue();

        // Should be ordered by withdraw_queue_index: market2 (0), market3 (1), market1 (2)
        assert_eq!(withdraw_queue.len(), 3);
        assert_eq!(withdraw_queue[0], FixedBytes::from_slice(&[2u8; 32]));
        assert_eq!(withdraw_queue[1], FixedBytes::from_slice(&[3u8; 32]));
        assert_eq!(withdraw_queue[2], FixedBytes::from_slice(&[1u8; 32]));
    }

    #[test]
    fn test_queue_with_none_indices() {
        let market1_id = FixedBytes::from_slice(&[1u8; 32]);
        let market2_id = FixedBytes::from_slice(&[2u8; 32]);

        let data = VaultSimulationData {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::ZERO,
            total_assets: U256::ZERO,
            total_assets_usd: None,
            total_supply: U256::ZERO,
            allocations: vec![
                VaultAllocationForSim {
                    market_id: market1_id,
                    supply_assets: U256::ZERO,
                    supply_cap: U256::ZERO,
                    enabled: true,
                    supply_queue_index: Some(0),
                    withdraw_queue_index: None, // Not in withdraw queue
                },
                VaultAllocationForSim {
                    market_id: market2_id,
                    supply_assets: U256::ZERO,
                    supply_cap: U256::ZERO,
                    enabled: true,
                    supply_queue_index: None, // Not in supply queue
                    withdraw_queue_index: Some(0),
                },
            ],
            markets: vec![],
        };

        let supply_queue = data.supply_queue();
        let withdraw_queue = data.withdraw_queue();

        assert_eq!(supply_queue.len(), 1);
        assert_eq!(supply_queue[0], market1_id);

        assert_eq!(withdraw_queue.len(), 1);
        assert_eq!(withdraw_queue[0], market2_id);
    }

    #[test]
    fn test_market_state_for_sim_serialization() {
        let market = create_test_market_state(FixedBytes::from_slice(&[1u8; 32]));

        let json = serde_json::to_string(&market).unwrap();
        let deserialized: MarketStateForSim = serde_json::from_str(&json).unwrap();

        assert_eq!(market, deserialized);
    }

    #[test]
    fn test_vault_allocation_for_sim_serialization() {
        let alloc = create_test_allocation(FixedBytes::from_slice(&[1u8; 32]), Some(0), Some(1));

        let json = serde_json::to_string(&alloc).unwrap();
        let deserialized: VaultAllocationForSim = serde_json::from_str(&json).unwrap();

        assert_eq!(alloc, deserialized);
    }

    #[test]
    fn test_vault_simulation_data_serialization() {
        let data = create_test_vault_simulation_data();

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: VaultSimulationData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_empty_allocations() {
        let data = VaultSimulationData {
            address: Address::ZERO,
            asset_decimals: 6,
            fee: U256::ZERO,
            total_assets: U256::ZERO,
            total_assets_usd: None,
            total_supply: U256::ZERO,
            allocations: vec![],
            markets: vec![],
        };

        assert!(data.supply_queue().is_empty());
        assert!(data.withdraw_queue().is_empty());
    }
}

#[cfg(all(test, feature = "sim"))]
mod sim_tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    fn create_test_market_state(id: B256) -> MarketStateForSim {
        MarketStateForSim {
            id,
            total_supply_assets: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_borrow_assets: U256::from(800_000_000_000_000_000_000_000u128),
            total_supply_shares: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_borrow_shares: U256::from(800_000_000_000_000_000_000_000u128),
            last_update: 1704067200,
            fee: U256::from(100_000_000_000_000_000u64),
            rate_at_target: Some(U256::from(1_268_391_679u64)),
            price: Some(U256::from(1_000_000_000_000_000_000_000_000_000_000_000_000u128)),
            lltv: U256::from(800_000_000_000_000_000u64),
        }
    }

    fn create_test_allocation(market_id: B256, supply_idx: Option<i32>, withdraw_idx: Option<i32>) -> VaultAllocationForSim {
        VaultAllocationForSim {
            market_id,
            supply_assets: U256::from(500_000_000_000_000_000_000_000u128),
            supply_cap: U256::from(1_000_000_000_000_000_000_000_000u128),
            enabled: true,
            supply_queue_index: supply_idx,
            withdraw_queue_index: withdraw_idx,
        }
    }

    #[test]
    fn test_market_state_to_sim_market() {
        let market_id = FixedBytes::from_slice(&[1u8; 32]);
        let market_state = create_test_market_state(market_id);

        let sim_market = market_state.to_sim_market();

        assert_eq!(sim_market.id, market_id);
        assert_eq!(sim_market.total_supply_assets, market_state.total_supply_assets);
        assert_eq!(sim_market.total_borrow_assets, market_state.total_borrow_assets);
        assert_eq!(sim_market.last_update, market_state.last_update);
        assert_eq!(sim_market.fee, market_state.fee);
        assert_eq!(sim_market.lltv, market_state.lltv);
    }

    #[test]
    fn test_vault_simulation_data_to_vault_simulation() {
        let market1_id = FixedBytes::from_slice(&[1u8; 32]);
        let market2_id = FixedBytes::from_slice(&[2u8; 32]);

        let data = VaultSimulationData {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(50_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_assets_usd: Some(1_000_000.0),
            total_supply: U256::from(1_000_000_000_000_000_000_000_000u128),
            allocations: vec![
                create_test_allocation(market1_id, Some(0), Some(1)),
                create_test_allocation(market2_id, Some(1), Some(0)),
            ],
            markets: vec![
                create_test_market_state(market1_id),
                create_test_market_state(market2_id),
            ],
        };

        let sim = data.to_vault_simulation().expect("Should convert successfully");

        // Verify vault properties
        assert_eq!(sim.vault.address, data.address);
        assert_eq!(sim.vault.asset_decimals, data.asset_decimals);
        assert_eq!(sim.vault.fee, data.fee);
        assert_eq!(sim.vault.total_assets, data.total_assets);
        assert_eq!(sim.vault.total_supply, data.total_supply);

        // Verify markets were created
        assert_eq!(sim.markets.len(), 2);
        assert!(sim.markets.contains_key(&market1_id));
        assert!(sim.markets.contains_key(&market2_id));

        // Verify allocations
        assert_eq!(sim.vault.allocations.len(), 2);

        // Verify queues are properly ordered
        assert_eq!(sim.vault.supply_queue.len(), 2);
        assert_eq!(sim.vault.supply_queue[0], market1_id);
        assert_eq!(sim.vault.supply_queue[1], market2_id);

        assert_eq!(sim.vault.withdraw_queue.len(), 2);
        assert_eq!(sim.vault.withdraw_queue[0], market2_id);
        assert_eq!(sim.vault.withdraw_queue[1], market1_id);
    }

    #[test]
    fn test_vault_simulation_can_calculate_apy() {
        let market_id = FixedBytes::from_slice(&[1u8; 32]);

        let data = VaultSimulationData {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(50_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000_000_000_000_000_000_000u128),
            total_assets_usd: None,
            total_supply: U256::from(1_000_000_000_000_000_000_000_000u128),
            allocations: vec![create_test_allocation(market_id, Some(0), Some(0))],
            markets: vec![create_test_market_state(market_id)],
        };

        let sim = data.to_vault_simulation().expect("Should convert successfully");

        // Should be able to calculate APY without panicking
        let apy = sim.get_net_apy(1704067200).expect("Should calculate APY");
        assert!(apy >= 0.0);
    }
}

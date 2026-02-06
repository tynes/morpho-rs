//! Vault simulation for MetaMorpho vaults.
//!
//! This module implements vault-level deposit/withdraw simulation and APY calculations
//! for [MetaMorpho](https://docs.morpho.org/metamorpho/overview) vaults, which distribute
//! assets across multiple Morpho Blue markets.
//!
//! # Overview
//!
//! MetaMorpho vaults are ERC4626-compliant yield aggregators that:
//! - **Diversify across markets**: Spread deposits across multiple lending markets
//! - **Manage risk**: Set supply caps per market to limit concentration
//! - **Optimize yield**: Allocators can rebalance to maximize returns
//! - **Charge fees**: Performance fees taken from accrued interest
//!
//! # Key Concepts
//!
//! - **Supply Queue**: Ordered list of markets for depositing (deposits fill first market first)
//! - **Withdraw Queue**: Ordered list for withdrawals (withdraws from first market first)
//! - **Supply Caps**: Maximum amount the vault can supply to each market
//! - **Public Allocator**: Permissionless reallocation with flow limits
//!
//! # Example
//!
//! ```rust,ignore
//! use morpho_rs_sim::{Vault, VaultSimulation, vault_deposit_apy_impact, WAD};
//!
//! // Create vault simulation with markets
//! let simulation = VaultSimulation::new(vault, markets);
//!
//! // Analyze deposit impact
//! let impact = vault_deposit_apy_impact(&simulation, deposit_amount, timestamp)?;
//!
//! println!("Net APY: {:.2}% -> {:.2}%", impact.apy_before * 100.0, impact.apy_after * 100.0);
//! ```

use std::collections::HashMap;

use alloy_primitives::{Address, U256};

use crate::error::{MarketId, SimError};
use crate::market::Market;
use crate::math::{
    self, mul_div, mul_div_down, rate_to_apy, w_mul_down, zero_floor_sub,
    RoundingDirection, WAD,
};

/// Virtual assets constant for vault share calculations (1)
pub const VAULT_VIRTUAL_ASSETS: U256 = U256::from_limbs([1, 0, 0, 0]);

/// Configuration for a market within a vault
#[derive(Debug, Clone)]
pub struct VaultMarketConfig {
    /// The market's unique identifier
    pub market_id: MarketId,
    /// Maximum supply cap for this market
    pub cap: U256,
    /// Current supply to this market from the vault
    pub supply_assets: U256,
    /// Whether this market is enabled
    pub enabled: bool,
    /// Public allocator configuration (if any)
    pub public_allocator_config: Option<PublicAllocatorMarketConfig>,
}

/// Public allocator configuration for a specific market
#[derive(Debug, Clone)]
pub struct PublicAllocatorMarketConfig {
    /// Maximum assets that can flow into this market
    pub max_in: U256,
    /// Maximum assets that can flow out of this market
    pub max_out: U256,
}

/// Vault-level public allocator configuration
#[derive(Debug, Clone)]
pub struct PublicAllocatorConfig {
    /// Fee to use public allocator (in native token)
    pub fee: U256,
    /// Accrued fees
    pub accrued_fee: U256,
}

/// Represents a MetaMorpho vault state
#[derive(Debug, Clone)]
pub struct Vault {
    /// The vault's address
    pub address: Address,
    /// Decimals of the underlying asset
    pub asset_decimals: u8,
    /// Performance fee (WAD-scaled)
    pub fee: U256,
    /// Total assets under management
    pub total_assets: U256,
    /// Total vault shares outstanding
    pub total_supply: U256,
    /// Last recorded total assets (for fee calculation)
    pub last_total_assets: U256,
    /// Ordered supply queue (markets to deposit into)
    pub supply_queue: Vec<MarketId>,
    /// Ordered withdraw queue (markets to withdraw from)
    pub withdraw_queue: Vec<MarketId>,
    /// Market configurations and current allocations
    pub allocations: HashMap<MarketId, VaultMarketConfig>,
    /// Vault owner
    pub owner: Address,
    /// Public allocator configuration
    pub public_allocator_config: Option<PublicAllocatorConfig>,
}

impl Vault {
    /// Calculate the decimals offset for share conversion
    pub fn decimals_offset(&self) -> u8 {
        18u8.saturating_sub(self.asset_decimals)
    }

    /// Virtual shares for this vault (10^decimals_offset)
    pub fn virtual_shares(&self) -> U256 {
        U256::from(10u64).pow(U256::from(self.decimals_offset()))
    }

    /// Convert vault shares to assets
    pub fn to_assets(&self, shares: U256, rounding: RoundingDirection) -> U256 {
        mul_div(
            shares,
            self.total_assets + VAULT_VIRTUAL_ASSETS,
            self.total_supply + self.virtual_shares(),
            rounding,
        )
    }

    /// Convert assets to vault shares
    pub fn to_shares(&self, assets: U256, rounding: RoundingDirection) -> U256 {
        mul_div(
            assets,
            self.total_supply + self.virtual_shares(),
            self.total_assets + VAULT_VIRTUAL_ASSETS,
            rounding,
        )
    }

    /// Get the total interest accrued since last update
    pub fn total_interest(&self) -> U256 {
        zero_floor_sub(self.total_assets, self.last_total_assets)
    }

    /// Calculate the maximum deposit capacity based on market caps
    pub fn max_deposit(&self) -> U256 {
        let mut suppliable = U256::ZERO;

        for market_id in &self.supply_queue {
            if let Some(config) = self.allocations.get(market_id) {
                let cap_room = zero_floor_sub(config.cap, config.supply_assets);
                suppliable = suppliable.saturating_add(cap_room);
            }
        }

        suppliable
    }

    /// Calculate the maximum withdraw capacity based on market liquidity
    pub fn max_withdraw(&self, markets: &HashMap<MarketId, Market>) -> U256 {
        let mut withdrawable = U256::ZERO;

        for market_id in &self.withdraw_queue {
            if let Some(config) = self.allocations.get(market_id) {
                if let Some(market) = markets.get(market_id) {
                    let liquidity = market.liquidity();
                    let available = math::min(config.supply_assets, liquidity);
                    withdrawable = withdrawable.saturating_add(available);
                }
            }
        }

        withdrawable
    }
}

/// Complete simulation state including vault and all its markets
#[derive(Debug, Clone)]
pub struct VaultSimulation {
    /// The vault state
    pub vault: Vault,
    /// All markets the vault has allocations in
    pub markets: HashMap<MarketId, Market>,
}

impl VaultSimulation {
    /// Create a new vault simulation
    pub fn new(vault: Vault, markets: HashMap<MarketId, Market>) -> Self {
        Self { vault, markets }
    }

    /// Accrue interest on all markets and update the vault state
    pub fn accrue_interest(&self, timestamp: u64) -> Result<VaultSimulation, SimError> {
        let mut new_markets = HashMap::new();
        let mut new_allocations = self.vault.allocations.clone();
        let mut new_total_assets = U256::ZERO;

        // Accrue interest on all markets and recalculate allocations
        for (market_id, config) in &self.vault.allocations {
            let market = self
                .markets
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?;

            let accrued_market = market.accrue_interest(timestamp)?;

            // Recalculate the vault's supply assets in this market
            // The vault holds supply shares, convert to assets at new rate
            let supply_shares = market.to_supply_shares(config.supply_assets, RoundingDirection::Up);
            let new_supply_assets =
                accrued_market.to_supply_assets(supply_shares, RoundingDirection::Down);

            new_allocations
                .get_mut(market_id)
                .ok_or(SimError::MarketNotFound { market_id: *market_id })?
                .supply_assets = new_supply_assets;
            new_total_assets += new_supply_assets;
            new_markets.insert(*market_id, accrued_market);
        }

        // Calculate vault interest and fee
        let mut new_vault = self.vault.clone();
        new_vault.allocations = new_allocations;
        new_vault.total_assets = new_total_assets;

        // Deduct performance fee
        let fee_assets = w_mul_down(new_vault.total_interest(), new_vault.fee);
        let temp_total_assets = new_vault.total_assets - fee_assets;

        let fee_shares = mul_div_down(
            fee_assets,
            new_vault.total_supply + new_vault.virtual_shares(),
            temp_total_assets + VAULT_VIRTUAL_ASSETS,
        );

        new_vault.total_supply += fee_shares;
        new_vault.last_total_assets = new_vault.total_assets;

        Ok(VaultSimulation {
            vault: new_vault,
            markets: new_markets,
        })
    }

    /// Calculate the weighted average supply rate across all allocations
    pub fn get_avg_supply_rate(&self, timestamp: u64) -> Result<U256, SimError> {
        if self.vault.total_assets.is_zero() {
            return Ok(U256::ZERO);
        }

        let mut weighted_rate = U256::ZERO;

        for (market_id, config) in &self.vault.allocations {
            if config.supply_assets.is_zero() {
                continue;
            }

            let market = self
                .markets
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?;

            let market_rate = market.get_avg_supply_rate(timestamp)?;
            weighted_rate += market_rate * config.supply_assets;
        }

        Ok(weighted_rate / self.vault.total_assets)
    }

    /// Calculate the vault's gross APY (before vault fee)
    pub fn get_apy(&self, timestamp: u64) -> Result<f64, SimError> {
        if self.vault.total_assets.is_zero() {
            return Ok(0.0);
        }

        let avg_rate = self.get_avg_supply_rate(timestamp)?;
        Ok(rate_to_apy(avg_rate))
    }

    /// Calculate the vault's net APY (after vault fee)
    pub fn get_net_apy(&self, timestamp: u64) -> Result<f64, SimError> {
        if self.vault.total_assets.is_zero() {
            return Ok(0.0);
        }

        let avg_rate = self.get_avg_supply_rate(timestamp)?;
        let net_rate = w_mul_down(avg_rate, WAD - self.vault.fee);
        Ok(rate_to_apy(net_rate))
    }

    /// Simulates a deposit to the vault.
    ///
    /// This simulates the full deposit flow: accrue interest on all markets,
    /// calculate vault shares to mint, then distribute the deposit across
    /// markets according to the supply queue.
    ///
    /// # Deposit Distribution
    ///
    /// Assets are allocated to markets in supply queue order:
    /// 1. For each market in `supply_queue`
    /// 2. Calculate available room: `cap - current_supply`
    /// 3. Supply `min(remaining_deposit, available_room)` to that market
    /// 4. Continue until deposit is fully allocated
    ///
    /// If the deposit exceeds total available capacity across all markets,
    /// an error is returned.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount of underlying assets to deposit (WAD-scaled)
    /// * `timestamp` - Current Unix timestamp
    ///
    /// # Returns
    ///
    /// A tuple of (new_simulation, shares_minted). The original simulation is unchanged.
    ///
    /// # Errors
    ///
    /// - [`SimError::AllCapsReached`] if deposit exceeds total market caps
    /// - Interest accrual errors from underlying markets
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (new_sim, shares) = simulation.simulate_deposit(
    ///     U256::from(100_000) * WAD,
    ///     timestamp
    /// )?;
    ///
    /// println!("Shares received: {}", shares);
    /// println!("New total assets: {}", new_sim.vault.total_assets);
    /// ```
    pub fn simulate_deposit(
        &self,
        amount: U256,
        timestamp: u64,
    ) -> Result<(VaultSimulation, U256), SimError> {
        // First accrue interest on all markets
        let mut sim = self.accrue_interest(timestamp)?;

        // Calculate shares to mint
        let shares = sim.vault.to_shares(amount, RoundingDirection::Down);

        // Distribute deposit across supply queue
        let mut to_supply = amount;

        for market_id in &sim.vault.supply_queue.clone() {
            let config = sim
                .vault
                .allocations
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?
                .clone();

            if config.cap.is_zero() {
                continue;
            }

            let suppliable = zero_floor_sub(config.cap, config.supply_assets);
            if suppliable.is_zero() {
                continue;
            }

            let supply_amount = math::min(to_supply, suppliable);

            // Supply to the market
            let market = sim
                .markets
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?;

            let (new_market, _) = market.supply(supply_amount, timestamp)?;
            sim.markets.insert(*market_id, new_market);

            // Update allocation
            let allocation = sim.vault.allocations.get_mut(market_id)
                .ok_or(SimError::MarketNotFound { market_id: *market_id })?;
            allocation.supply_assets += supply_amount;

            to_supply -= supply_amount;

            if to_supply.is_zero() {
                break;
            }
        }

        if !to_supply.is_zero() {
            return Err(SimError::AllCapsReached {
                vault: sim.vault.address,
                remaining: to_supply.saturating_to::<u128>(),
            });
        }

        // Update vault state
        sim.vault.total_assets += amount;
        sim.vault.last_total_assets = sim.vault.total_assets;
        sim.vault.total_supply += shares;

        Ok((sim, shares))
    }

    /// Simulate a withdrawal from the vault
    ///
    /// Returns the updated simulation state and the assets withdrawn
    pub fn simulate_withdraw(
        &self,
        shares: U256,
        timestamp: u64,
    ) -> Result<(VaultSimulation, U256), SimError> {
        // First accrue interest on all markets
        let mut sim = self.accrue_interest(timestamp)?;

        // Calculate assets to withdraw
        let assets = sim.vault.to_assets(shares, RoundingDirection::Down);

        // Distribute withdrawal across withdraw queue
        let mut to_withdraw = assets;

        for market_id in &sim.vault.withdraw_queue.clone() {
            let config = sim
                .vault
                .allocations
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?
                .clone();

            let market = sim
                .markets
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?;

            // Calculate withdrawable from this market
            let liquidity = market.liquidity();
            let withdrawable = math::min(config.supply_assets, liquidity);

            if withdrawable.is_zero() {
                continue;
            }

            let withdraw_amount = math::min(to_withdraw, withdrawable);

            // Withdraw from the market
            let (new_market, _) = market.withdraw(withdraw_amount, timestamp)?;
            sim.markets.insert(*market_id, new_market);

            // Update allocation
            let allocation = sim.vault.allocations.get_mut(market_id)
                .ok_or(SimError::MarketNotFound { market_id: *market_id })?;
            allocation.supply_assets -= withdraw_amount;

            to_withdraw -= withdraw_amount;

            if to_withdraw.is_zero() {
                break;
            }
        }

        if !to_withdraw.is_zero() {
            return Err(SimError::NotEnoughLiquidity {
                vault: sim.vault.address,
                remaining: to_withdraw.saturating_to::<u128>(),
            });
        }

        // Update vault state
        sim.vault.total_assets -= assets;
        sim.vault.last_total_assets = sim.vault.total_assets;
        sim.vault.total_supply -= shares;

        Ok((sim, assets))
    }

    /// Simulate a reallocation of assets between markets
    pub fn simulate_reallocate(
        &self,
        allocations: &[ReallocationStep],
        timestamp: u64,
    ) -> Result<VaultSimulation, SimError> {
        let mut sim = self.clone();
        let mut total_supplied = U256::ZERO;
        let mut total_withdrawn = U256::ZERO;

        for step in allocations {
            // Accrue interest on this market
            let market = sim
                .markets
                .get(&step.market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: step.market_id,
                })?;
            let accrued_market = market.accrue_interest(timestamp)?;
            sim.markets.insert(step.market_id, accrued_market);

            let config = sim
                .vault
                .allocations
                .get(&step.market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: step.market_id,
                })?
                .clone();

            let current_supply = config.supply_assets;

            if step.target_assets < current_supply {
                // Withdraw
                let to_withdraw = current_supply - step.target_assets;

                if !config.enabled {
                    return Err(SimError::MarketNotEnabled {
                        vault: sim.vault.address,
                        market_id: step.market_id,
                    });
                }

                let market = sim.markets.get(&step.market_id)
                    .ok_or(SimError::MarketNotFound { market_id: step.market_id })?;
                let (new_market, _) = market.withdraw(to_withdraw, timestamp)?;
                sim.markets.insert(step.market_id, new_market);

                let allocation = sim.vault.allocations.get_mut(&step.market_id)
                    .ok_or(SimError::MarketNotFound { market_id: step.market_id })?;
                allocation.supply_assets = step.target_assets;

                total_withdrawn += to_withdraw;
            } else if step.target_assets > current_supply {
                // Supply
                let to_supply = step.target_assets - current_supply;

                if config.cap.is_zero() {
                    return Err(SimError::UnauthorizedMarket {
                        vault: sim.vault.address,
                        market_id: step.market_id,
                    });
                }

                if step.target_assets > config.cap {
                    return Err(SimError::SupplyCapExceeded {
                        vault: sim.vault.address,
                        market_id: step.market_id,
                        cap: config.cap.saturating_to::<u128>(),
                    });
                }

                let market = sim.markets.get(&step.market_id)
                    .ok_or(SimError::MarketNotFound { market_id: step.market_id })?;
                let (new_market, _) = market.supply(to_supply, timestamp)?;
                sim.markets.insert(step.market_id, new_market);

                let allocation = sim.vault.allocations.get_mut(&step.market_id)
                    .ok_or(SimError::MarketNotFound { market_id: step.market_id })?;
                allocation.supply_assets = step.target_assets;

                total_supplied += to_supply;
            }
        }

        // Reallocation must be balanced
        if total_withdrawn != total_supplied {
            return Err(SimError::InconsistentReallocation {
                vault: sim.vault.address,
                supplied: total_supplied.saturating_to::<u128>(),
                withdrawn: total_withdrawn.saturating_to::<u128>(),
            });
        }

        Ok(sim)
    }

    /// Simulate a public reallocation
    pub fn simulate_public_reallocate(
        &self,
        withdrawals: &[(MarketId, U256)],
        supply_market_id: MarketId,
        timestamp: u64,
    ) -> Result<VaultSimulation, SimError> {
        // Validate public allocator is configured
        if self.vault.public_allocator_config.is_none() {
            return Err(SimError::PublicAllocatorNotConfigured {
                vault: self.vault.address,
            });
        }

        if withdrawals.is_empty() {
            return Err(SimError::EmptyWithdrawals {
                vault: self.vault.address,
            });
        }

        // Validate supply market
        let supply_config = self
            .vault
            .allocations
            .get(&supply_market_id)
            .ok_or(SimError::MarketNotFound {
                market_id: supply_market_id,
            })?;

        if !supply_config.enabled {
            return Err(SimError::MarketNotEnabled {
                vault: self.vault.address,
                market_id: supply_market_id,
            });
        }

        let supply_pa_config = supply_config
            .public_allocator_config
            .as_ref()
            .ok_or(SimError::PublicAllocatorNotConfigured {
                vault: self.vault.address,
            })?;

        let mut sim = self.clone();
        let mut total_withdrawn = U256::ZERO;
        let mut prev_id: Option<MarketId> = None;

        // Process withdrawals
        for (market_id, amount) in withdrawals {
            // Check ordering (must be sorted)
            if let Some(prev) = prev_id {
                if *market_id <= prev {
                    return Err(SimError::WithdrawalsNotSorted {
                        vault: sim.vault.address,
                    });
                }
            }
            prev_id = Some(*market_id);

            if *market_id == supply_market_id {
                return Err(SimError::DepositMarketInWithdrawals {
                    vault: sim.vault.address,
                    market_id: *market_id,
                });
            }

            let config = sim
                .vault
                .allocations
                .get(market_id)
                .ok_or(SimError::MarketNotFound {
                    market_id: *market_id,
                })?;

            if !config.enabled {
                return Err(SimError::MarketNotEnabled {
                    vault: sim.vault.address,
                    market_id: *market_id,
                });
            }

            let pa_config = config
                .public_allocator_config
                .as_ref()
                .ok_or(SimError::PublicAllocatorNotConfigured {
                    vault: sim.vault.address,
                })?;

            if pa_config.max_out < *amount {
                return Err(SimError::MaxOutflowExceeded {
                    vault: sim.vault.address,
                    market_id: *market_id,
                });
            }

            if config.supply_assets < *amount {
                return Err(SimError::InsufficientMarketLiquidity {
                    market_id: *market_id,
                });
            }

            total_withdrawn += *amount;
        }

        // Check max inflow on supply market
        if supply_pa_config.max_in < total_withdrawn {
            return Err(SimError::MaxInflowExceeded {
                vault: sim.vault.address,
                market_id: supply_market_id,
            });
        }

        // Build reallocation steps
        let mut steps: Vec<ReallocationStep> = withdrawals
            .iter()
            .map(|(market_id, amount)| {
                let current = sim.vault.allocations.get(market_id)
                    .ok_or(SimError::MarketNotFound { market_id: *market_id })?
                    .supply_assets;
                Ok(ReallocationStep {
                    market_id: *market_id,
                    target_assets: current - *amount,
                })
            })
            .collect::<Result<Vec<_>, SimError>>()?;

        // Add supply step
        let supply_current = sim
            .vault
            .allocations
            .get(&supply_market_id)
            .ok_or(SimError::MarketNotFound { market_id: supply_market_id })?
            .supply_assets;
        steps.push(ReallocationStep {
            market_id: supply_market_id,
            target_assets: supply_current + total_withdrawn,
        });

        // Update public allocator configs
        for (market_id, amount) in withdrawals {
            let config = sim.vault.allocations.get_mut(market_id)
                .ok_or(SimError::MarketNotFound { market_id: *market_id })?;
            let pa_config = config.public_allocator_config.as_mut()
                .ok_or(SimError::PublicAllocatorNotConfigured { vault: sim.vault.address })?;
            pa_config.max_in += *amount;
            pa_config.max_out -= *amount;
        }

        let supply_config = sim.vault.allocations.get_mut(&supply_market_id)
            .ok_or(SimError::MarketNotFound { market_id: supply_market_id })?;
        let supply_pa_config = supply_config.public_allocator_config.as_mut()
            .ok_or(SimError::PublicAllocatorNotConfigured { vault: sim.vault.address })?;
        supply_pa_config.max_in -= total_withdrawn;
        supply_pa_config.max_out += total_withdrawn;

        // Execute reallocation
        sim.simulate_reallocate(&steps, timestamp)
    }
}

/// A step in a reallocation operation
#[derive(Debug, Clone)]
pub struct ReallocationStep {
    /// Market to reallocate
    pub market_id: MarketId,
    /// Target allocation after reallocation
    pub target_assets: U256,
}

/// Result of APY impact calculation
#[derive(Debug, Clone)]
pub struct VaultApyImpact {
    /// APY before the operation (as decimal, e.g., 0.05 = 5%)
    pub apy_before: f64,
    /// APY after the operation
    pub apy_after: f64,
    /// Change in APY (negative = APY decreases)
    pub apy_delta: f64,
    /// Shares minted/burned
    pub shares: U256,
}

/// Calculates the APY impact of a vault deposit.
///
/// This function simulates a deposit and compares the vault's net APY
/// (after performance fees) before and after the operation.
///
/// # Why Vault APY Changes with Deposits
///
/// A deposit affects vault APY because:
/// 1. **Market APY dilution**: Depositing to underlying markets reduces their APY
/// 2. **Allocation changes**: Deposit may shift capital to different markets
/// 3. **Weighted average effect**: The vault's APY is a weighted average of market APYs
///
/// # Arguments
///
/// * `simulation` - The vault simulation state
/// * `amount` - Amount of assets to deposit (WAD-scaled)
/// * `timestamp` - Current Unix timestamp
///
/// # Returns
///
/// A [`VaultApyImpact`] containing:
/// - `apy_before`: Net APY before deposit (as decimal, e.g., 0.05 = 5%)
/// - `apy_after`: Net APY after deposit
/// - `apy_delta`: Change in APY (typically negative as supply dilutes returns)
/// - `shares`: Vault shares that would be minted
///
/// # Example
///
/// ```rust,ignore
/// use morpho_rs_sim::vault_deposit_apy_impact;
///
/// let impact = vault_deposit_apy_impact(&simulation, deposit, timestamp)?;
///
/// println!("APY impact: {:.4}%", impact.apy_delta * 100.0);
/// println!("Shares to receive: {}", impact.shares);
///
/// // Decide if the post-deposit APY is acceptable
/// if impact.apy_after < 0.03 {
///     println!("Warning: APY would drop below 3%");
/// }
/// ```
pub fn vault_deposit_apy_impact(
    simulation: &VaultSimulation,
    amount: U256,
    timestamp: u64,
) -> Result<VaultApyImpact, SimError> {
    let apy_before = simulation.get_net_apy(timestamp)?;
    let (new_sim, shares) = simulation.simulate_deposit(amount, timestamp)?;
    let apy_after = new_sim.get_net_apy(timestamp)?;

    Ok(VaultApyImpact {
        apy_before,
        apy_after,
        apy_delta: apy_after - apy_before,
        shares,
    })
}

/// Calculate the APY impact of a vault withdrawal
pub fn vault_withdraw_apy_impact(
    simulation: &VaultSimulation,
    shares: U256,
    timestamp: u64,
) -> Result<VaultApyImpact, SimError> {
    let apy_before = simulation.get_net_apy(timestamp)?;
    let (new_sim, _assets) = simulation.simulate_withdraw(shares, timestamp)?;
    let apy_after = new_sim.get_net_apy(timestamp)?;

    Ok(VaultApyImpact {
        apy_before,
        apy_after,
        apy_delta: apy_after - apy_before,
        shares,
    })
}

/// Calculate the deposit amount needed to achieve a target APY impact
pub fn amount_for_vault_apy_impact(
    simulation: &VaultSimulation,
    target_apy_delta: f64,
    timestamp: u64,
) -> Result<Option<U256>, SimError> {
    // Deposits can only decrease APY (or have no effect), so positive target is invalid
    if target_apy_delta > 0.0 {
        return Ok(None);
    }

    // If target is zero or very small, return zero
    if target_apy_delta.abs() < 1e-10 {
        return Ok(Some(U256::ZERO));
    }

    let current_apy = simulation.get_net_apy(timestamp)?;
    let target_apy = current_apy + target_apy_delta;

    // If target APY is negative, it's not achievable
    if target_apy < 0.0 {
        return Ok(None);
    }

    // Calculate maximum deposit capacity
    let max_deposit = simulation.vault.max_deposit();
    if max_deposit.is_zero() {
        return Ok(None);
    }

    // Binary search for the deposit amount
    let mut low = U256::ZERO;
    let mut high = max_deposit;
    let tolerance = 1e-8;
    let max_iterations = 100u32;

    for _ in 0..max_iterations {
        let mid = (low + high) / U256::from(2);

        if mid == low || mid == high {
            return Ok(Some(mid));
        }

        match simulation.simulate_deposit(mid, timestamp) {
            Ok((new_sim, _)) => {
                let new_apy = new_sim.get_net_apy(timestamp)?;
                let delta = new_apy - current_apy;

                if (delta - target_apy_delta).abs() < tolerance {
                    return Ok(Some(mid));
                }

                if delta > target_apy_delta {
                    low = mid;
                } else {
                    high = mid;
                }
            }
            Err(SimError::AllCapsReached { .. }) => {
                high = mid;
            }
            Err(e) => return Err(e),
        }
    }

    Err(SimError::ConvergenceFailure { max_iterations })
}

/// Vault ranking entry
#[derive(Debug, Clone)]
pub struct VaultRanking {
    pub vault_address: Address,
    pub net_apy: f64,
    pub gross_apy: f64,
    pub total_assets: U256,
    pub available_capacity: U256,
}

/// Rank vaults by net APY (descending)
pub fn rank_vaults_by_apy(
    vaults: &[&VaultSimulation],
    timestamp: u64,
) -> Result<Vec<VaultRanking>, SimError> {
    let mut rankings: Vec<VaultRanking> = vaults
        .iter()
        .filter_map(|sim| {
            let net_apy = sim.get_net_apy(timestamp).ok()?;
            let gross_apy = sim.get_apy(timestamp).ok()?;
            Some(VaultRanking {
                vault_address: sim.vault.address,
                net_apy,
                gross_apy,
                total_assets: sim.vault.total_assets,
                available_capacity: sim.vault.max_deposit(),
            })
        })
        .collect();

    rankings.sort_by(|a, b| {
        b.net_apy
            .partial_cmp(&a.net_apy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(rankings)
}

/// Find the best vault for a given deposit amount
pub fn find_best_vault_for_deposit(
    vaults: &[&VaultSimulation],
    amount: U256,
    timestamp: u64,
) -> Result<Option<(Address, f64)>, SimError> {
    let mut best: Option<(Address, f64)> = None;

    for sim in vaults {
        // Check if vault has capacity
        if sim.vault.max_deposit() < amount {
            continue;
        }

        if let Ok(impact) = vault_deposit_apy_impact(sim, amount, timestamp) {
            match best {
                None => best = Some((sim.vault.address, impact.apy_after)),
                Some((_, best_apy)) if impact.apy_after > best_apy => {
                    best = Some((sim.vault.address, impact.apy_after));
                }
                _ => {}
            }
        }
    }

    Ok(best)
}

/// Optimal allocation result
#[derive(Debug, Clone)]
pub struct OptimalAllocation {
    /// Market ID
    pub market_id: MarketId,
    /// Amount to allocate
    pub amount: U256,
    /// Expected APY after allocation
    pub expected_apy: f64,
}

/// Finds an optimal allocation across multiple markets.
///
/// Given a set of markets, a total amount to allocate, and per-market caps,
/// this function uses a greedy algorithm to maximize yield. It allocates to
/// the highest-APY market first, then the next highest, and so on.
///
/// # Algorithm
///
/// 1. Rank all markets by current supply APY (descending)
/// 2. For each market in order:
///    - Calculate allocable amount: `min(remaining, cap)`
///    - Simulate supply to get expected post-allocation APY
///    - Add allocation to result
/// 3. Continue until amount is fully allocated or caps exhausted
///
/// # Limitations
///
/// The greedy approach is fast but may not find the global optimum when:
/// - Large deposits significantly move APYs
/// - Multiple markets have similar APYs but different sensitivities
///
/// For most practical cases, the greedy approach provides good results.
///
/// # Arguments
///
/// * `markets` - Slice of (market_id, market) tuples
/// * `total_amount` - Total assets to allocate (WAD-scaled)
/// * `caps` - Maximum allocation per market (missing = unlimited)
/// * `timestamp` - Current Unix timestamp
///
/// # Returns
///
/// A vector of [`OptimalAllocation`] entries, each containing:
/// - `market_id`: The market to allocate to
/// - `amount`: Amount to allocate
/// - `expected_apy`: APY after allocation
///
/// # Example
///
/// ```rust,ignore
/// use morpho_rs_sim::find_optimal_market_allocation;
///
/// let total = U256::from(1_000_000) * WAD;
/// let allocations = find_optimal_market_allocation(
///     &markets,
///     total,
///     &caps,
///     timestamp
/// )?;
///
/// for alloc in &allocations {
///     println!(
///         "Market {:?}: {} ({:.2}% APY)",
///         alloc.market_id,
///         alloc.amount,
///         alloc.expected_apy * 100.0
///     );
/// }
/// ```
pub fn find_optimal_market_allocation(
    markets: &[(MarketId, &Market)],
    total_amount: U256,
    caps: &HashMap<MarketId, U256>,
    timestamp: u64,
) -> Result<Vec<OptimalAllocation>, SimError> {
    // Simple greedy allocation: allocate to highest APY markets first
    let mut rankings: Vec<_> = markets
        .iter()
        .filter_map(|(id, market)| {
            let apy = market.get_supply_apy(timestamp).ok()?;
            let cap = caps.get(id).copied().unwrap_or(U256::MAX);
            Some((*id, *market, apy, cap))
        })
        .collect();

    // Sort by APY descending
    rankings.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut allocations = Vec::new();
    let mut remaining = total_amount;

    for (market_id, market, _, cap) in rankings {
        if remaining.is_zero() {
            break;
        }

        let available_cap = cap;
        let allocate = math::min(remaining, available_cap);

        if allocate.is_zero() {
            continue;
        }

        // Calculate APY after allocation
        let (new_market, _) = market.supply(allocate, timestamp)?;
        let expected_apy = new_market.get_supply_apy(timestamp)?;

        allocations.push(OptimalAllocation {
            market_id,
            amount: allocate,
            expected_apy,
        });

        remaining -= allocate;
    }

    Ok(allocations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    fn create_test_market(id: u8, total_supply: u64, total_borrow: u64) -> (MarketId, Market) {
        let market_id = FixedBytes::from([id; 32]);
        let market = Market::new(
            market_id,
            U256::from(total_supply) * WAD,
            U256::from(total_borrow) * WAD,
            U256::from(total_supply) * WAD,
            U256::from(total_borrow) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
        );
        (market_id, market)
    }

    fn create_test_simulation() -> VaultSimulation {
        let (market_id_1, market_1) = create_test_market(1, 1_000_000, 800_000);
        let (market_id_2, market_2) = create_test_market(2, 500_000, 400_000);

        let mut markets = HashMap::new();
        markets.insert(market_id_1, market_1);
        markets.insert(market_id_2, market_2);

        let mut allocations = HashMap::new();
        allocations.insert(
            market_id_1,
            VaultMarketConfig {
                market_id: market_id_1,
                cap: U256::from(2_000_000) * WAD,
                supply_assets: U256::from(600_000) * WAD,
                enabled: true,
                public_allocator_config: None,
            },
        );
        allocations.insert(
            market_id_2,
            VaultMarketConfig {
                market_id: market_id_2,
                cap: U256::from(1_000_000) * WAD,
                supply_assets: U256::from(400_000) * WAD,
                enabled: true,
                public_allocator_config: None,
            },
        );

        let vault = Vault {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(100_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000) * WAD,
            total_supply: U256::from(1_000_000) * WAD,
            last_total_assets: U256::from(1_000_000) * WAD,
            supply_queue: vec![market_id_1, market_id_2],
            withdraw_queue: vec![market_id_1, market_id_2],
            allocations,
            owner: Address::ZERO,
            public_allocator_config: None,
        };

        VaultSimulation::new(vault, markets)
    }

    #[test]
    fn test_vault_to_shares() {
        let sim = create_test_simulation();
        let shares = sim
            .vault
            .to_shares(U256::from(1000) * WAD, RoundingDirection::Down);
        assert!(shares > U256::from(990) * WAD);
        assert!(shares <= U256::from(1000) * WAD);
    }

    #[test]
    fn test_vault_to_assets() {
        let sim = create_test_simulation();
        let assets = sim
            .vault
            .to_assets(U256::from(1000) * WAD, RoundingDirection::Down);
        assert!(assets > U256::from(990) * WAD);
        assert!(assets <= U256::from(1000) * WAD);
    }

    #[test]
    fn test_get_net_apy() {
        let sim = create_test_simulation();
        let apy = sim.get_net_apy(1000).unwrap();
        assert!(apy > 0.0);
        assert!(apy < 0.5);
    }

    #[test]
    fn test_simulate_deposit() {
        let sim = create_test_simulation();
        let deposit = U256::from(100_000) * WAD;

        let (new_sim, shares) = sim.simulate_deposit(deposit, 1000).unwrap();

        assert!(shares > U256::ZERO);
        assert_eq!(
            new_sim.vault.total_assets,
            sim.vault.total_assets + deposit
        );
    }

    #[test]
    fn test_simulate_withdraw() {
        let sim = create_test_simulation();
        let shares = U256::from(100_000) * WAD;

        let (new_sim, assets) = sim.simulate_withdraw(shares, 1000).unwrap();

        assert!(assets > U256::ZERO);
        assert!(new_sim.vault.total_assets < sim.vault.total_assets);
    }

    #[test]
    fn test_deposit_apy_impact() {
        let sim = create_test_simulation();
        let deposit = U256::from(100_000) * WAD;

        let impact = vault_deposit_apy_impact(&sim, deposit, 1000).unwrap();

        assert!(impact.apy_delta <= 0.0);
        assert!(impact.shares > U256::ZERO);
    }

    #[test]
    fn test_withdraw_apy_impact() {
        let sim = create_test_simulation();
        let shares = U256::from(100_000) * WAD;

        let impact = vault_withdraw_apy_impact(&sim, shares, 1000).unwrap();

        // Withdrawing should increase APY (less dilution)
        assert!(impact.apy_delta >= 0.0);
    }

    #[test]
    fn test_max_deposit() {
        let sim = create_test_simulation();
        let max_dep = sim.vault.max_deposit();
        assert_eq!(max_dep, U256::from(2_000_000) * WAD);
    }

    #[test]
    fn test_max_withdraw() {
        let sim = create_test_simulation();
        let max_withdraw = sim.vault.max_withdraw(&sim.markets);
        // Limited by liquidity: 200K + 100K = 300K
        assert!(max_withdraw > U256::ZERO);
    }

    #[test]
    fn test_deposit_exceeds_caps() {
        let sim = create_test_simulation();
        let deposit = U256::from(3_000_000) * WAD;

        let result = sim.simulate_deposit(deposit, 1000);
        assert!(matches!(result, Err(SimError::AllCapsReached { .. })));
    }

    #[test]
    fn test_reallocate() {
        let sim = create_test_simulation();
        let market_id_1 = sim.vault.supply_queue[0];
        let market_id_2 = sim.vault.supply_queue[1];

        // Move 100K from market 1 to market 2
        let steps = vec![
            ReallocationStep {
                market_id: market_id_1,
                target_assets: U256::from(500_000) * WAD,
            },
            ReallocationStep {
                market_id: market_id_2,
                target_assets: U256::from(500_000) * WAD,
            },
        ];

        let new_sim = sim.simulate_reallocate(&steps, 1000).unwrap();

        // Verify allocations changed
        let alloc_1 = new_sim.vault.allocations.get(&market_id_1).unwrap();
        let alloc_2 = new_sim.vault.allocations.get(&market_id_2).unwrap();

        assert!(alloc_1.supply_assets < sim.vault.allocations.get(&market_id_1).unwrap().supply_assets);
        assert!(alloc_2.supply_assets > sim.vault.allocations.get(&market_id_2).unwrap().supply_assets);
    }

    #[test]
    fn test_amount_for_apy_impact_positive_delta() {
        let sim = create_test_simulation();
        let result = amount_for_vault_apy_impact(&sim, 0.01, 1000).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_amount_for_apy_impact_zero_delta() {
        let sim = create_test_simulation();
        let result = amount_for_vault_apy_impact(&sim, 0.0, 1000).unwrap();
        assert_eq!(result, Some(U256::ZERO));
    }

    // ==================== New Tests ====================

    #[test]
    fn test_deposit_to_multiple_markets() {
        // Test that large deposit distributes across multiple markets
        let sim = create_test_simulation();

        // Deposit more than first market's cap allows
        let large_deposit = U256::from(1_500_000) * WAD;
        let (new_sim, shares) = sim.simulate_deposit(large_deposit, 1000).unwrap();

        assert!(shares > U256::ZERO);

        // Both markets should have received deposits
        let market_id_1 = sim.vault.supply_queue[0];
        let market_id_2 = sim.vault.supply_queue[1];

        let alloc_1 = new_sim.vault.allocations.get(&market_id_1).unwrap();
        let alloc_2 = new_sim.vault.allocations.get(&market_id_2).unwrap();

        // Market 1 should be at cap
        assert!(alloc_1.supply_assets >= alloc_1.cap - U256::from(1));

        // Market 2 should have received the overflow
        let original_alloc_2 = sim.vault.allocations.get(&market_id_2).unwrap();
        assert!(alloc_2.supply_assets > original_alloc_2.supply_assets);
    }

    #[test]
    fn test_withdraw_from_multiple_markets() {
        let sim = create_test_simulation();

        // Withdraw more than first market's liquidity
        let large_withdraw = U256::from(250_000) * WAD; // More than 200K liquidity in market 1

        let result = sim.simulate_withdraw(large_withdraw, 1000);

        // Should succeed by withdrawing from multiple markets
        assert!(result.is_ok());
        let (new_sim, assets) = result.unwrap();

        // Both markets should have reduced supply
        let market_id_1 = sim.vault.supply_queue[0];

        let alloc_1 = new_sim.vault.allocations.get(&market_id_1).unwrap();
        let orig_alloc_1 = sim.vault.allocations.get(&market_id_1).unwrap();

        assert!(alloc_1.supply_assets < orig_alloc_1.supply_assets);
        assert!(assets > U256::ZERO);
    }

    #[test]
    fn test_reallocate_inconsistent() {
        let sim = create_test_simulation();
        let market_id_1 = sim.vault.supply_queue[0];
        let market_id_2 = sim.vault.supply_queue[1];

        // Try to reallocate with inconsistent amounts (total doesn't match)
        let steps = vec![
            ReallocationStep {
                market_id: market_id_1,
                target_assets: U256::from(500_000) * WAD, // -100K
            },
            ReallocationStep {
                market_id: market_id_2,
                target_assets: U256::from(450_000) * WAD, // +50K (not balanced)
            },
        ];

        let result = sim.simulate_reallocate(&steps, 1000);
        assert!(matches!(result, Err(SimError::InconsistentReallocation { .. })));
    }

    #[test]
    fn test_reallocate_to_unknown_market() {
        let sim = create_test_simulation();
        let unknown_market_id = FixedBytes::from([99; 32]);

        let steps = vec![ReallocationStep {
            market_id: unknown_market_id,
            target_assets: U256::from(500_000) * WAD,
        }];

        let result = sim.simulate_reallocate(&steps, 1000);
        assert!(matches!(result, Err(SimError::MarketNotFound { .. })));
    }

    #[test]
    fn test_public_reallocate() {
        // Create simulation with public allocator config
        let (market_id_1, market_1) = create_test_market(1, 1_000_000, 800_000);
        let (market_id_2, market_2) = create_test_market(2, 500_000, 400_000);

        let mut markets = HashMap::new();
        markets.insert(market_id_1, market_1);
        markets.insert(market_id_2, market_2);

        let mut allocations = HashMap::new();
        allocations.insert(
            market_id_1,
            VaultMarketConfig {
                market_id: market_id_1,
                cap: U256::from(2_000_000) * WAD,
                supply_assets: U256::from(600_000) * WAD,
                enabled: true,
                public_allocator_config: Some(PublicAllocatorMarketConfig {
                    max_in: U256::from(200_000) * WAD,
                    max_out: U256::from(200_000) * WAD,
                }),
            },
        );
        allocations.insert(
            market_id_2,
            VaultMarketConfig {
                market_id: market_id_2,
                cap: U256::from(1_000_000) * WAD,
                supply_assets: U256::from(400_000) * WAD,
                enabled: true,
                public_allocator_config: Some(PublicAllocatorMarketConfig {
                    max_in: U256::from(200_000) * WAD,
                    max_out: U256::from(200_000) * WAD,
                }),
            },
        );

        let vault = Vault {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(100_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000) * WAD,
            total_supply: U256::from(1_000_000) * WAD,
            last_total_assets: U256::from(1_000_000) * WAD,
            supply_queue: vec![market_id_1, market_id_2],
            withdraw_queue: vec![market_id_1, market_id_2],
            allocations,
            owner: Address::ZERO,
            public_allocator_config: Some(PublicAllocatorConfig {
                fee: U256::from(1_000_000_000_000_000u64), // 0.1%
                accrued_fee: U256::ZERO,
            }),
        };

        let sim = VaultSimulation::new(vault, markets);

        // Public reallocate: withdraw 100K from market_id_1, supply to market_id_2
        let withdrawals = vec![(market_id_1, U256::from(100_000) * WAD)];

        let new_sim = sim.simulate_public_reallocate(&withdrawals, market_id_2, 1000).unwrap();

        // Verify allocations changed
        let alloc_1 = new_sim.vault.allocations.get(&market_id_1).unwrap();
        let alloc_2 = new_sim.vault.allocations.get(&market_id_2).unwrap();

        assert!(alloc_1.supply_assets < sim.vault.allocations.get(&market_id_1).unwrap().supply_assets);
        assert!(alloc_2.supply_assets > sim.vault.allocations.get(&market_id_2).unwrap().supply_assets);
    }

    #[test]
    fn test_public_reallocate_exceeds_max_out() {
        // Create simulation with limited max_out
        let (market_id_1, market_1) = create_test_market(1, 1_000_000, 800_000);
        let (market_id_2, market_2) = create_test_market(2, 500_000, 400_000);

        let mut markets = HashMap::new();
        markets.insert(market_id_1, market_1);
        markets.insert(market_id_2, market_2);

        let mut allocations = HashMap::new();
        allocations.insert(
            market_id_1,
            VaultMarketConfig {
                market_id: market_id_1,
                cap: U256::from(2_000_000) * WAD,
                supply_assets: U256::from(600_000) * WAD,
                enabled: true,
                public_allocator_config: Some(PublicAllocatorMarketConfig {
                    max_in: U256::from(200_000) * WAD,
                    max_out: U256::from(50_000) * WAD, // Limited to 50K
                }),
            },
        );
        allocations.insert(
            market_id_2,
            VaultMarketConfig {
                market_id: market_id_2,
                cap: U256::from(1_000_000) * WAD,
                supply_assets: U256::from(400_000) * WAD,
                enabled: true,
                public_allocator_config: Some(PublicAllocatorMarketConfig {
                    max_in: U256::from(200_000) * WAD,
                    max_out: U256::from(200_000) * WAD,
                }),
            },
        );

        let vault = Vault {
            address: Address::ZERO,
            asset_decimals: 18,
            fee: U256::from(100_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000) * WAD,
            total_supply: U256::from(1_000_000) * WAD,
            last_total_assets: U256::from(1_000_000) * WAD,
            supply_queue: vec![market_id_1, market_id_2],
            withdraw_queue: vec![market_id_1, market_id_2],
            allocations,
            owner: Address::ZERO,
            public_allocator_config: Some(PublicAllocatorConfig {
                fee: U256::from(1_000_000_000_000_000u64),
                accrued_fee: U256::ZERO,
            }),
        };

        let sim = VaultSimulation::new(vault, markets);

        // Try to withdraw 100K from market 1 (exceeds 50K max_out)
        let withdrawals = vec![(market_id_1, U256::from(100_000) * WAD)];

        let result = sim.simulate_public_reallocate(&withdrawals, market_id_2, 1000);
        assert!(matches!(result, Err(SimError::MaxOutflowExceeded { .. })));
    }

    #[test]
    fn test_vault_apy_calculation() {
        let sim = create_test_simulation();

        let gross_apy = sim.get_apy(1000).unwrap();
        let net_apy = sim.get_net_apy(1000).unwrap();

        // Net APY should be less than gross APY due to fees
        assert!(net_apy < gross_apy);
        assert!(net_apy > 0.0);
    }

    #[test]
    fn test_vault_ranking() {
        let sim1 = create_test_simulation();

        // Create a second simulation with different markets
        let (market_id_3, market_3) = create_test_market(3, 1_000_000, 900_000); // Higher utilization
        let (market_id_4, market_4) = create_test_market(4, 1_000_000, 950_000); // Even higher

        let mut markets2 = HashMap::new();
        markets2.insert(market_id_3, market_3);
        markets2.insert(market_id_4, market_4);

        let mut allocations2 = HashMap::new();
        allocations2.insert(
            market_id_3,
            VaultMarketConfig {
                market_id: market_id_3,
                cap: U256::from(2_000_000) * WAD,
                supply_assets: U256::from(500_000) * WAD,
                enabled: true,
                public_allocator_config: None,
            },
        );
        allocations2.insert(
            market_id_4,
            VaultMarketConfig {
                market_id: market_id_4,
                cap: U256::from(2_000_000) * WAD,
                supply_assets: U256::from(500_000) * WAD,
                enabled: true,
                public_allocator_config: None,
            },
        );

        let vault2 = Vault {
            address: Address::from([1u8; 20]),
            asset_decimals: 18,
            fee: U256::from(100_000_000_000_000_000u64),
            total_assets: U256::from(1_000_000) * WAD,
            total_supply: U256::from(1_000_000) * WAD,
            last_total_assets: U256::from(1_000_000) * WAD,
            supply_queue: vec![market_id_3, market_id_4],
            withdraw_queue: vec![market_id_3, market_id_4],
            allocations: allocations2,
            owner: Address::ZERO,
            public_allocator_config: None,
        };

        let sim2 = VaultSimulation::new(vault2, markets2);

        let vaults: Vec<&VaultSimulation> = vec![&sim1, &sim2];

        let rankings = rank_vaults_by_apy(&vaults, 1000).unwrap();

        // Should have 2 rankings
        assert_eq!(rankings.len(), 2);

        // Higher utilization vault should have higher APY
        // (sim2 has 90-95% utilization vs sim1 has 80%)
        assert_eq!(rankings[0].vault_address, Address::from([1u8; 20]));
    }

    #[test]
    fn test_find_best_vault_for_deposit() {
        let sim1 = create_test_simulation();

        // Create a second simulation with different available capacity
        let (market_id_3, market_3) = create_test_market(3, 1_000_000, 900_000);

        let mut markets2 = HashMap::new();
        markets2.insert(market_id_3, market_3);

        let mut allocations2 = HashMap::new();
        allocations2.insert(
            market_id_3,
            VaultMarketConfig {
                market_id: market_id_3,
                cap: U256::from(5_000_000) * WAD, // Large cap
                supply_assets: U256::from(500_000) * WAD,
                enabled: true,
                public_allocator_config: None,
            },
        );

        let vault2 = Vault {
            address: Address::from([2u8; 20]),
            asset_decimals: 18,
            fee: U256::from(50_000_000_000_000_000u64), // Lower fee
            total_assets: U256::from(500_000) * WAD,
            total_supply: U256::from(500_000) * WAD,
            last_total_assets: U256::from(500_000) * WAD,
            supply_queue: vec![market_id_3],
            withdraw_queue: vec![market_id_3],
            allocations: allocations2,
            owner: Address::ZERO,
            public_allocator_config: None,
        };

        let sim2 = VaultSimulation::new(vault2, markets2);

        let vaults: Vec<&VaultSimulation> = vec![&sim1, &sim2];

        let deposit = U256::from(100_000) * WAD;
        let result = find_best_vault_for_deposit(&vaults, deposit, 1000).unwrap();

        assert!(result.is_some());
    }

    #[test]
    fn test_optimal_allocation() {
        let sim = create_test_simulation();

        // Build markets and caps from simulation
        let markets: Vec<(MarketId, &Market)> = sim.markets.iter()
            .map(|(id, m)| (*id, m))
            .collect();

        let caps: HashMap<MarketId, U256> = sim.vault.allocations.iter()
            .map(|(id, config)| (*id, config.cap - config.supply_assets))
            .collect();

        let amount = U256::from(500_000) * WAD;
        let allocations = find_optimal_market_allocation(&markets, amount, &caps, 1000).unwrap();

        // Should have allocations
        assert!(!allocations.is_empty());

        // Total should equal input amount (or less if caps limit)
        let total: U256 = allocations.iter().map(|a| a.amount).sum();
        assert!(total <= amount);
    }

    #[test]
    fn test_withdraw_exceeds_liquidity() {
        let sim = create_test_simulation();

        // Try to withdraw more than available liquidity
        let huge_withdraw = U256::from(500_000) * WAD; // More than 300K combined liquidity

        let result = sim.simulate_withdraw(huge_withdraw, 1000);
        assert!(matches!(result, Err(SimError::NotEnoughLiquidity { .. })));
    }

    #[test]
    fn test_vault_interest_accrual() {
        let sim = create_test_simulation();

        // Simulate deposit after time has passed
        let future_timestamp = 1000 + 86400; // 1 day later
        let (new_sim, _) = sim.simulate_deposit(U256::from(1000) * WAD, future_timestamp).unwrap();

        // After interest accrual, market totals should have increased
        let market_id_1 = sim.vault.supply_queue[0];

        let old_market = sim.markets.get(&market_id_1).unwrap();
        let new_market = new_sim.markets.get(&market_id_1).unwrap();

        // Interest should have accrued
        assert!(new_market.total_supply_assets > old_market.total_supply_assets);
    }
}

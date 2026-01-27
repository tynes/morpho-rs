//! Type definitions for strategy crate.
//!
//! These types are used internally by the strategy crate and can be converted
//! from morpho-rs-api types.

use alloy_chains::NamedChain;
use alloy_primitives::{Address, B256, U256};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Vault version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VaultVersion {
    #[default]
    V1,
    V2,
}

/// Vault warning severity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VaultWarning {
    /// Non-critical warning (informational).
    Info(String),
    /// Critical warning that should prevent deposits.
    Critical(String),
}

/// Vault reward token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultReward {
    /// Reward token address.
    pub token: Address,
    /// Reward APY (as decimal, e.g., 0.05 = 5%).
    pub apy: Decimal,
}

/// Vault adapter (for MetaMorpho allocations).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VaultAdapter {
    /// Adapter address.
    pub address: Address,
    /// Allocation percentage (0.0 to 1.0).
    pub allocation: f64,
    /// Underlying Morpho Blue market ID (if this adapter is a market).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_id: Option<B256>,
}

/// Unified vault representation for strategy calculations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vault {
    /// Vault contract address.
    pub address: Address,
    /// Chain the vault is deployed on.
    pub chain: NamedChain,
    /// Vault name.
    pub name: String,
    /// Vault symbol.
    pub symbol: String,
    /// Underlying asset address.
    pub asset: Address,
    /// Whether the vault is whitelisted.
    pub whitelisted: bool,
    /// Curator address (if any).
    pub curator: Option<Address>,
    /// Net APY after fees (as decimal).
    pub net_apy: Decimal,
    /// Total assets in the vault.
    pub total_assets: U256,
    /// Total assets in USD.
    pub total_assets_usd: f64,
    /// Adapters/allocations.
    pub adapters: Vec<VaultAdapter>,
    /// Reward tokens and APYs.
    pub rewards: Vec<VaultReward>,
    /// Warnings.
    pub warnings: Vec<VaultWarning>,
    /// Vault version.
    pub version: VaultVersion,
    /// Available liquidity for withdrawals.
    /// For V2 vaults: directly from API.
    /// For V1 vaults: calculated from simulation via `max_withdraw()`.
    pub liquidity: U256,
}

impl Vault {
    /// Calculate total APY including rewards.
    pub fn total_apy(&self) -> Decimal {
        let rewards_apy: Decimal = self.rewards.iter().map(|r| r.apy).sum();
        self.net_apy + rewards_apy
    }

    /// Check if vault has critical warnings.
    pub fn has_critical_warnings(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| matches!(w, VaultWarning::Critical(_)))
    }
}

/// Type of position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PositionType {
    /// Position in a vault (V1 or V2).
    VaultV2,
    /// Wallet balance.
    WalletBalance,
}

/// User position in Morpho protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphoPosition {
    /// Type of position.
    pub position_type: PositionType,
    /// Chain the position is on.
    pub chain: NamedChain,
    /// Target address (vault address or zero for wallet).
    pub target: Address,
    /// Position name.
    pub name: String,
    /// Shares held (for vault positions).
    pub shares: U256,
    /// Assets value.
    pub assets: U256,
    /// Current APY.
    pub apy: Decimal,
    /// Position warnings.
    pub warnings: Vec<VaultWarning>,
    /// Available liquidity for withdrawals from this vault.
    /// For wallet positions, this equals assets (fully liquid).
    /// For vault positions, this is the vault's available liquidity.
    pub liquidity: U256,
}

impl MorphoPosition {
    /// Create a wallet position.
    pub fn wallet(chain: NamedChain, balance: U256) -> Self {
        Self {
            position_type: PositionType::WalletBalance,
            chain,
            target: Address::ZERO,
            name: format!("USDC Wallet ({chain})"),
            shares: U256::ZERO,
            assets: balance,
            apy: Decimal::ZERO,
            warnings: vec![],
            // Wallet balances are fully liquid
            liquidity: balance,
        }
    }

    /// Check if position is withdrawable (no critical warnings).
    pub fn is_withdrawable(&self) -> bool {
        !self.warnings
            .iter()
            .any(|w| matches!(w, VaultWarning::Critical(_)))
    }

    /// Get assets as u128 for calculations.
    pub fn assets_u128(&self) -> u128 {
        self.assets.to::<u128>()
    }

    /// Check if this is a zero-yield position (wallet or 0 APY).
    pub fn is_idle(&self) -> bool {
        self.position_type == PositionType::WalletBalance || self.apy == Decimal::ZERO
    }
}

/// Math utilities for Morpho calculations.
pub struct MorphoMath;

impl MorphoMath {
    /// Convert a decimal to basis points (integer).
    ///
    /// Uses direct Decimal conversion instead of string parsing for efficiency
    /// and to avoid masking precision errors.
    pub fn decimal_to_bps(d: Decimal) -> i64 {
        let bps = (d * Decimal::from(10000)).round();
        bps.try_into().unwrap_or_else(|_| {
            if bps.is_sign_positive() {
                i64::MAX
            } else {
                i64::MIN
            }
        })
    }

    /// Convert f64 to Decimal.
    pub fn f64_to_decimal(f: f64) -> Decimal {
        Decimal::try_from(f).unwrap_or(Decimal::ZERO)
    }
}

/// Convert U256 to USDC amount (6 decimals) as f64.
pub fn u256_to_usdc(amount: U256) -> f64 {
    let amount_u128 = amount.to::<u128>();
    (amount_u128 as f64) / 1_000_000.0
}

/// Convert USDC amount (f64) to U256.
pub fn try_usdc_to_u256(usdc: f64) -> Result<U256, &'static str> {
    if usdc < 0.0 {
        return Err("USDC amount cannot be negative");
    }
    if usdc > (u128::MAX as f64) / 1_000_000.0 {
        return Err("USDC amount too large");
    }
    let raw = (usdc * 1_000_000.0).round() as u128;
    Ok(U256::from(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_vault_total_apy() {
        let vault = Vault {
            address: Address::ZERO,
            chain: NamedChain::Base,
            name: "Test".to_string(),
            symbol: "TEST".to_string(),
            asset: Address::ZERO,
            whitelisted: true,
            curator: None,
            net_apy: dec!(0.05),
            total_assets: U256::ZERO,
            total_assets_usd: 0.0,
            adapters: vec![],
            rewards: vec![VaultReward {
                token: Address::ZERO,
                apy: dec!(0.02),
            }],
            warnings: vec![],
            version: VaultVersion::V1,
            liquidity: U256::MAX,
        };

        assert_eq!(vault.total_apy(), dec!(0.07));
    }

    #[test]
    fn test_vault_has_critical_warnings() {
        let mut vault = Vault {
            address: Address::ZERO,
            chain: NamedChain::Base,
            name: "Test".to_string(),
            symbol: "TEST".to_string(),
            asset: Address::ZERO,
            whitelisted: true,
            curator: None,
            net_apy: dec!(0.05),
            total_assets: U256::ZERO,
            total_assets_usd: 0.0,
            adapters: vec![],
            rewards: vec![],
            warnings: vec![],
            version: VaultVersion::V1,
            liquidity: U256::MAX,
        };

        assert!(!vault.has_critical_warnings());

        vault.warnings.push(VaultWarning::Info("info".to_string()));
        assert!(!vault.has_critical_warnings());

        vault
            .warnings
            .push(VaultWarning::Critical("critical".to_string()));
        assert!(vault.has_critical_warnings());
    }

    #[test]
    fn test_decimal_to_bps() {
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.01)), 100);
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.001)), 10);
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.0001)), 1);
    }

    #[test]
    fn test_decimal_to_bps_negative() {
        assert_eq!(MorphoMath::decimal_to_bps(dec!(-0.01)), -100);
        assert_eq!(MorphoMath::decimal_to_bps(dec!(-0.001)), -10);
        assert_eq!(MorphoMath::decimal_to_bps(dec!(-0.0001)), -1);
    }

    #[test]
    fn test_decimal_to_bps_zero() {
        assert_eq!(MorphoMath::decimal_to_bps(Decimal::ZERO), 0);
    }

    #[test]
    fn test_decimal_to_bps_rounding() {
        // Test rounding behavior - Decimal::round() uses banker's rounding
        // 0.00006 * 10000 = 0.6 -> rounds to 1
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.00006)), 1);
        // 0.00004 * 10000 = 0.4 -> rounds to 0
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.00004)), 0);
        // 0.00016 * 10000 = 1.6 -> rounds to 2
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.00016)), 2);
        // 0.00014 * 10000 = 1.4 -> rounds to 1
        assert_eq!(MorphoMath::decimal_to_bps(dec!(0.00014)), 1);
    }

    #[test]
    fn test_decimal_to_bps_large_values() {
        // Large positive value within i64 range
        // 922337203685477 * 10000 = ~9.2e18, which is close to i64::MAX (~9.2e18)
        let large = Decimal::from(922_337_203_685_477_i64);
        let bps = MorphoMath::decimal_to_bps(large);
        assert!(bps > 0);

        // Value that would overflow i64 after multiplication
        // i64::MAX / 10000 * 10000 = i64::MAX - (i64::MAX % 10000)
        // But i64::MAX / 10000 + 1 * 10000 would overflow
        let just_under_overflow = Decimal::from(i64::MAX / 10000);
        let bps = MorphoMath::decimal_to_bps(just_under_overflow);
        assert!(bps > 0);

        // Test negative large value
        let large_neg = Decimal::from(-922_337_203_685_477_i64);
        let bps = MorphoMath::decimal_to_bps(large_neg);
        assert!(bps < 0);
    }

    #[test]
    fn test_u256_to_usdc() {
        assert_eq!(u256_to_usdc(U256::from(1_000_000u64)), 1.0);
        assert_eq!(u256_to_usdc(U256::from(1_500_000u64)), 1.5);
    }

    #[test]
    fn test_try_usdc_to_u256() {
        assert_eq!(try_usdc_to_u256(1.0).unwrap(), U256::from(1_000_000u64));
        assert_eq!(try_usdc_to_u256(1.5).unwrap(), U256::from(1_500_000u64));
        assert!(try_usdc_to_u256(-1.0).is_err());
    }
}

//! Shared ERC-4626 client trait and macro for vault clients.
//!
//! This module provides a trait with default implementations for ERC-4626 view functions,
//! and a macro for transaction methods that return `PreparedCall`.

#![allow(async_fn_in_trait)]

use alloy::primitives::{Address, U256};

use crate::erc4626::IERC4626;
use crate::error::{ContractError, Result};
use crate::provider::HttpProvider;

/// Trait for ERC-4626 vault client functionality.
///
/// Provides default implementations for all ERC-4626 view functions.
/// Implementors only need to provide `provider()` and `signer_address()`.
pub trait Erc4626Client {
    /// Returns a reference to the HTTP provider.
    fn provider(&self) -> &HttpProvider;

    /// Returns the signer's address.
    fn signer_address(&self) -> Address;

    /// Get the underlying asset address of a vault.
    async fn get_asset(&self, vault: Address) -> Result<Address> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract
            .asset()
            .call()
            .await
            .map_err(|e| ContractError::TransactionFailed(format!("Failed to get asset: {}", e)))?;
        Ok(result)
    }

    /// Get the total assets managed by a vault.
    async fn total_assets(&self, vault: Address) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.totalAssets().call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get total assets: {}", e))
        })?;
        Ok(result)
    }

    /// Convert an asset amount to shares.
    async fn convert_to_shares(&self, vault: Address, assets: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.convertToShares(assets).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to convert to shares: {}", e))
        })?;
        Ok(result)
    }

    /// Convert a share amount to assets.
    async fn convert_to_assets(&self, vault: Address, shares: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.convertToAssets(shares).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to convert to assets: {}", e))
        })?;
        Ok(result)
    }

    /// Get the maximum deposit amount for a receiver.
    async fn max_deposit(&self, vault: Address, receiver: Address) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.maxDeposit(receiver).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get max deposit: {}", e))
        })?;
        Ok(result)
    }

    /// Get the maximum withdraw amount for an owner.
    async fn max_withdraw(&self, vault: Address, owner: Address) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.maxWithdraw(owner).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get max withdraw: {}", e))
        })?;
        Ok(result)
    }

    /// Get the maximum mint amount (in shares) for a receiver.
    async fn max_mint(&self, vault: Address, receiver: Address) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.maxMint(receiver).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get max mint: {}", e))
        })?;
        Ok(result)
    }

    /// Get the maximum redeem amount (in shares) for an owner.
    async fn max_redeem(&self, vault: Address, owner: Address) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.maxRedeem(owner).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get max redeem: {}", e))
        })?;
        Ok(result)
    }

    /// Preview the shares that would be received for a deposit.
    async fn preview_deposit(&self, vault: Address, assets: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.previewDeposit(assets).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to preview deposit: {}", e))
        })?;
        Ok(result)
    }

    /// Preview the assets required to mint a specific amount of shares.
    async fn preview_mint(&self, vault: Address, shares: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.previewMint(shares).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to preview mint: {}", e))
        })?;
        Ok(result)
    }

    /// Preview the shares that would be burned for a withdrawal.
    async fn preview_withdraw(&self, vault: Address, assets: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.previewWithdraw(assets).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to preview withdraw: {}", e))
        })?;
        Ok(result)
    }

    /// Preview the assets that would be received for redeeming shares.
    async fn preview_redeem(&self, vault: Address, shares: U256) -> Result<U256> {
        let contract = IERC4626::new(vault, self.provider());
        let result = contract.previewRedeem(shares).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to preview redeem: {}", e))
        })?;
        Ok(result)
    }
}

/// Macro to implement ERC-4626 transaction methods on a client struct.
///
/// This macro generates `deposit`, `withdraw`, `mint`, and `redeem` methods
/// that return `PreparedCall` types. It's needed because trait methods cannot
/// return types with lifetime parameters tied to `self`.
///
/// # Usage
///
/// ```rust,ignore
/// impl_erc4626_transactions!(MyVaultClient);
/// ```
#[macro_export]
macro_rules! impl_erc4626_transactions {
    ($client:ty) => {
        impl $client {
            /// Create a prepared deposit transaction.
            /// Returns a `PreparedCall` that can be sent or used with `MulticallBuilder`.
            pub fn deposit(
                &self,
                vault: alloy::primitives::Address,
                amount: alloy::primitives::U256,
                receiver: alloy::primitives::Address,
            ) -> $crate::prepared_call::PreparedCall<'_, $crate::erc4626::IERC4626::depositCall> {
                let call = $crate::erc4626::IERC4626::depositCall {
                    assets: amount,
                    receiver,
                };
                $crate::prepared_call::PreparedCall::new(
                    vault,
                    call,
                    alloy::primitives::U256::ZERO,
                    &self.provider,
                )
            }

            /// Create a prepared withdraw transaction.
            /// Returns a `PreparedCall` that can be sent or used with `MulticallBuilder`.
            pub fn withdraw(
                &self,
                vault: alloy::primitives::Address,
                amount: alloy::primitives::U256,
                receiver: alloy::primitives::Address,
                owner: alloy::primitives::Address,
            ) -> $crate::prepared_call::PreparedCall<'_, $crate::erc4626::IERC4626::withdrawCall>
            {
                let call = $crate::erc4626::IERC4626::withdrawCall {
                    assets: amount,
                    receiver,
                    owner,
                };
                $crate::prepared_call::PreparedCall::new(
                    vault,
                    call,
                    alloy::primitives::U256::ZERO,
                    &self.provider,
                )
            }

            /// Create a prepared mint transaction.
            /// Returns a `PreparedCall` that can be sent or used with `MulticallBuilder`.
            pub fn mint(
                &self,
                vault: alloy::primitives::Address,
                shares: alloy::primitives::U256,
                receiver: alloy::primitives::Address,
            ) -> $crate::prepared_call::PreparedCall<'_, $crate::erc4626::IERC4626::mintCall> {
                let call = $crate::erc4626::IERC4626::mintCall { shares, receiver };
                $crate::prepared_call::PreparedCall::new(
                    vault,
                    call,
                    alloy::primitives::U256::ZERO,
                    &self.provider,
                )
            }

            /// Create a prepared redeem transaction.
            /// Returns a `PreparedCall` that can be sent or used with `MulticallBuilder`.
            pub fn redeem(
                &self,
                vault: alloy::primitives::Address,
                shares: alloy::primitives::U256,
                receiver: alloy::primitives::Address,
                owner: alloy::primitives::Address,
            ) -> $crate::prepared_call::PreparedCall<'_, $crate::erc4626::IERC4626::redeemCall>
            {
                let call = $crate::erc4626::IERC4626::redeemCall {
                    shares,
                    receiver,
                    owner,
                };
                $crate::prepared_call::PreparedCall::new(
                    vault,
                    call,
                    alloy::primitives::U256::ZERO,
                    &self.provider,
                )
            }
        }
    };
}

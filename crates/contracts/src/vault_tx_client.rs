//! Macros for defining vault transaction clients.
//!
//! This module provides macros to reduce boilerplate when creating vault transaction
//! clients that implement ERC-4626 functionality.

/// Macro to define a complete vault transaction client.
///
/// This macro generates:
/// - The struct definition with `provider` and `signer_address` fields
/// - The `new()` constructor
/// - ERC-20 helper methods: `get_decimals`, `get_balance`, `get_allowance`, `approve`, `approve_if_needed`
/// - `Erc4626Client` trait implementation
/// - ERC-4626 transaction methods via `impl_erc4626_transactions!`
///
/// # Usage
///
/// ```rust,ignore
/// define_vault_transaction_client!(
///     /// Client for executing transactions against V1 (MetaMorpho) vaults.
///     VaultV1TransactionClient,
///     "V1"
/// );
/// ```
#[macro_export]
macro_rules! define_vault_transaction_client {
    (
        $(#[$meta:meta])*
        $client_name:ident,
        $version:literal
    ) => {
        use alloy::{
            network::EthereumWallet,
            primitives::{Address, U256},
            providers::ProviderBuilder,
            signers::local::PrivateKeySigner,
        };

        use $crate::erc20::IERC20;
        use $crate::erc4626_client::Erc4626Client;
        use $crate::error::{ContractError, Result};
        use $crate::prepared_call::PreparedCall;
        use $crate::provider::HttpProvider;

        $(#[$meta])*
        pub struct $client_name {
            provider: HttpProvider,
            signer_address: Address,
        }

        impl $client_name {
            #[doc = concat!("Create a new ", $version, " transaction client.")]
            pub fn new(rpc_url: &str, private_key: &str) -> Result<Self> {
                let signer: PrivateKeySigner = private_key
                    .parse()
                    .map_err(|_| ContractError::InvalidPrivateKey)?;
                let signer_address = signer.address();
                let wallet = EthereumWallet::from(signer);

                let url: url::Url = rpc_url
                    .parse()
                    .map_err(|e| ContractError::RpcConnection(format!("{}", e)))?;

                let provider = ProviderBuilder::new()
                    .wallet(wallet)
                    .connect_http(url);

                Ok(Self {
                    provider,
                    signer_address,
                })
            }

            /// Get the decimals of a token.
            pub async fn get_decimals(&self, token: Address) -> Result<u8> {
                let contract = IERC20::new(token, &self.provider);
                let result = contract.decimals().call().await.map_err(|e| {
                    ContractError::TransactionFailed(format!("Failed to get decimals: {}", e))
                })?;
                Ok(result)
            }

            /// Get the balance of a token for an address.
            pub async fn get_balance(&self, token: Address, owner: Address) -> Result<U256> {
                let contract = IERC20::new(token, &self.provider);
                let result = contract.balanceOf(owner).call().await.map_err(|e| {
                    ContractError::TransactionFailed(format!("Failed to get balance: {}", e))
                })?;
                Ok(result)
            }

            /// Get the allowance of a token for a spender.
            pub async fn get_allowance(
                &self,
                token: Address,
                owner: Address,
                spender: Address,
            ) -> Result<U256> {
                let contract = IERC20::new(token, &self.provider);
                let result = contract.allowance(owner, spender).call().await.map_err(|e| {
                    ContractError::TransactionFailed(format!("Failed to get allowance: {}", e))
                })?;
                Ok(result)
            }

            /// Create a prepared approval transaction.
            /// Returns a `PreparedCall` that can be sent or used with `MulticallBuilder`.
            pub fn approve(
                &self,
                token: Address,
                spender: Address,
                amount: U256,
            ) -> PreparedCall<'_, IERC20::approveCall> {
                let call = IERC20::approveCall { spender, amount };
                PreparedCall::new(token, call, U256::ZERO, &self.provider)
            }

            /// Approve a spender to use tokens if needed.
            /// Returns a `PreparedCall` if approval is needed, None otherwise.
            pub async fn approve_if_needed(
                &self,
                token: Address,
                spender: Address,
                amount: U256,
            ) -> Result<Option<PreparedCall<'_, IERC20::approveCall>>> {
                let current_allowance = self
                    .get_allowance(token, self.signer_address, spender)
                    .await?;

                if current_allowance >= amount {
                    return Ok(None);
                }

                Ok(Some(self.approve(token, spender, amount)))
            }
        }

        // Implement the Erc4626Client trait for view functions
        impl Erc4626Client for $client_name {
            fn provider(&self) -> &HttpProvider {
                &self.provider
            }

            fn signer_address(&self) -> Address {
                self.signer_address
            }
        }

        // Use macro to generate ERC-4626 transaction methods (deposit, withdraw, mint, redeem)
        $crate::impl_erc4626_transactions!($client_name);
    };
}

/// Macro to define tests for vault transaction clients.
///
/// This macro generates standard tests for:
/// - Invalid private key handling
/// - Invalid RPC URL handling
/// - Valid construction
///
/// # Usage
///
/// ```rust,ignore
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use crate::define_vault_client_tests;
///     define_vault_client_tests!(VaultV1TransactionClient);
/// }
/// ```
#[macro_export]
macro_rules! define_vault_client_tests {
    ($client_name:ident) => {
        #[test]
        fn test_invalid_private_key() {
            let result = $client_name::new("http://localhost:8545", "invalid_key");
            assert!(matches!(result, Err(ContractError::InvalidPrivateKey)));
        }

        #[test]
        fn test_invalid_rpc_url() {
            // Valid private key (32 bytes hex)
            let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
            let result = $client_name::new("not a valid url", private_key);
            assert!(matches!(result, Err(ContractError::RpcConnection(_))));
        }

        #[test]
        fn test_valid_construction() {
            let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
            let result = $client_name::new("http://localhost:8545", private_key);
            assert!(result.is_ok());
        }
    };
}

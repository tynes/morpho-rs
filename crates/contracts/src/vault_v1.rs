//! V1 Vault transaction client for executing deposits and withdrawals.

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};

use crate::erc20::IERC20;
use crate::erc4626_client::Erc4626Client;
use crate::error::{ContractError, Result};
use crate::prepared_call::PreparedCall;
use crate::provider::HttpProvider;
use crate::impl_erc4626_transactions;

/// Client for executing transactions against V1 (MetaMorpho) vaults.
pub struct VaultV1TransactionClient {
    provider: HttpProvider,
    signer_address: Address,
}

impl VaultV1TransactionClient {
    /// Create a new V1 transaction client.
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
impl Erc4626Client for VaultV1TransactionClient {
    fn provider(&self) -> &HttpProvider {
        &self.provider
    }

    fn signer_address(&self) -> Address {
        self.signer_address
    }
}

// Use macro to generate ERC-4626 transaction methods (deposit, withdraw, mint, redeem)
impl_erc4626_transactions!(VaultV1TransactionClient);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_private_key() {
        let result = VaultV1TransactionClient::new("http://localhost:8545", "invalid_key");
        assert!(matches!(result, Err(ContractError::InvalidPrivateKey)));
    }

    #[test]
    fn test_invalid_rpc_url() {
        // Valid private key (32 bytes hex)
        let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = VaultV1TransactionClient::new("not a valid url", private_key);
        assert!(matches!(result, Err(ContractError::RpcConnection(_))));
    }

    #[test]
    fn test_valid_construction() {
        let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = VaultV1TransactionClient::new("http://localhost:8545", private_key);
        assert!(result.is_ok());
    }
}

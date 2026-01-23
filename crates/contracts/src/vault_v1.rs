//! V1 Vault transaction client for executing deposits and withdrawals.

use alloy::{
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::ProviderBuilder,
    rpc::types::TransactionReceipt,
    signers::local::PrivateKeySigner,
};

use crate::erc20::IERC20;
use crate::erc4626::IERC4626;
use crate::error::{ContractError, Result};
use crate::provider::HttpProvider;

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
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        Ok(Self {
            provider,
            signer_address,
        })
    }

    /// Get the underlying asset address of a vault.
    pub async fn get_asset(&self, vault: Address) -> Result<Address> {
        let contract = IERC4626::new(vault, &self.provider);
        let result = contract
            .asset()
            .call()
            .await
            .map_err(|e| ContractError::TransactionFailed(format!("Failed to get asset: {}", e)))?;
        Ok(result._0)
    }

    /// Get the decimals of a token.
    pub async fn get_decimals(&self, token: Address) -> Result<u8> {
        let contract = IERC20::new(token, &self.provider);
        let result = contract.decimals().call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get decimals: {}", e))
        })?;
        Ok(result._0)
    }

    /// Get the balance of a token for an address.
    pub async fn get_balance(&self, token: Address, owner: Address) -> Result<U256> {
        let contract = IERC20::new(token, &self.provider);
        let result = contract.balanceOf(owner).call().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get balance: {}", e))
        })?;
        Ok(result._0)
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
        Ok(result._0)
    }

    /// Approve a spender to use tokens if needed.
    /// Returns the transaction receipt if approval was needed, None otherwise.
    pub async fn approve_if_needed(
        &self,
        token: Address,
        spender: Address,
        amount: U256,
    ) -> Result<Option<TransactionReceipt>> {
        let current_allowance = self
            .get_allowance(token, self.signer_address, spender)
            .await?;

        if current_allowance >= amount {
            return Ok(None);
        }

        let contract = IERC20::new(token, &self.provider);
        let tx = contract.approve(spender, amount);

        let pending = tx.send().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to send approval: {}", e))
        })?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get approval receipt: {}", e))
        })?;

        Ok(Some(receipt))
    }

    /// Deposit assets into a vault.
    /// Returns the transaction receipt.
    pub async fn deposit(
        &self,
        vault: Address,
        amount: U256,
        receiver: Address,
    ) -> Result<TransactionReceipt> {
        let contract = IERC4626::new(vault, &self.provider);
        let tx = contract.deposit(amount, receiver);

        let pending = tx.send().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to send deposit: {}", e))
        })?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get deposit receipt: {}", e))
        })?;

        Ok(receipt)
    }

    /// Withdraw assets from a vault.
    /// Returns the transaction receipt.
    pub async fn withdraw(
        &self,
        vault: Address,
        amount: U256,
        receiver: Address,
        owner: Address,
    ) -> Result<TransactionReceipt> {
        let contract = IERC4626::new(vault, &self.provider);
        let tx = contract.withdraw(amount, receiver, owner);

        let pending = tx.send().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to send withdraw: {}", e))
        })?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get withdraw receipt: {}", e))
        })?;

        Ok(receipt)
    }

    /// Get the signer's address.
    pub fn signer_address(&self) -> Address {
        self.signer_address
    }
}

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

//! V2 Vault transaction client for executing deposits and withdrawals.

use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::{Address, U256},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, ProviderBuilder, RootProvider,
    },
    rpc::types::TransactionReceipt,
    signers::local::PrivateKeySigner,
    transports::http::{Client, Http},
};

use crate::error::{ApiError, Result};
use crate::tx::erc20::IERC20;
use crate::tx::erc4626::IERC4626;

/// The recommended fillers type from `with_recommended_fillers()`.
type RecommendedFillers =
    JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>;

/// The concrete provider type used by the transaction client.
/// This matches what `ProviderBuilder::new().with_recommended_fillers().wallet().on_http()` returns.
pub type HttpProvider = FillProvider<
    JoinFill<JoinFill<Identity, RecommendedFillers>, WalletFiller<EthereumWallet>>,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

/// Client for executing transactions against V2 vaults.
pub struct VaultV2TransactionClient {
    provider: HttpProvider,
    signer_address: Address,
}

impl VaultV2TransactionClient {
    /// Create a new V2 transaction client.
    pub fn new(rpc_url: &str, private_key: &str) -> Result<Self> {
        let signer: PrivateKeySigner = private_key
            .parse()
            .map_err(|_| ApiError::InvalidPrivateKey)?;
        let signer_address = signer.address();
        let wallet = EthereumWallet::from(signer);

        let url: url::Url = rpc_url
            .parse()
            .map_err(|e| ApiError::RpcConnection(format!("{}", e)))?;

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
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to get asset: {}", e)))?;
        Ok(result._0)
    }

    /// Get the decimals of a token.
    pub async fn get_decimals(&self, token: Address) -> Result<u8> {
        let contract = IERC20::new(token, &self.provider);
        let result = contract
            .decimals()
            .call()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to get decimals: {}", e)))?;
        Ok(result._0)
    }

    /// Get the balance of a token for an address.
    pub async fn get_balance(&self, token: Address, owner: Address) -> Result<U256> {
        let contract = IERC20::new(token, &self.provider);
        let result = contract
            .balanceOf(owner)
            .call()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to get balance: {}", e)))?;
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
        let result = contract
            .allowance(owner, spender)
            .call()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to get allowance: {}", e)))?;
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

        let pending = tx
            .send()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to send approval: {}", e)))?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ApiError::TransactionFailed(format!("Failed to get approval receipt: {}", e))
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

        let pending = tx
            .send()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to send deposit: {}", e)))?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ApiError::TransactionFailed(format!("Failed to get deposit receipt: {}", e))
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

        let pending = tx
            .send()
            .await
            .map_err(|e| ApiError::TransactionFailed(format!("Failed to send withdraw: {}", e)))?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ApiError::TransactionFailed(format!("Failed to get withdraw receipt: {}", e))
        })?;

        Ok(receipt)
    }

    /// Get the signer's address.
    pub fn signer_address(&self) -> Address {
        self.signer_address
    }
}

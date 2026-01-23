//! Error types for the Morpho API client.

use alloy_primitives::U256;
use thiserror::Error;

/// Errors that can occur when using the Morpho API client.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// GraphQL query returned errors.
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    /// Failed to parse response.
    #[error("Failed to parse response: {0}")]
    Parse(String),

    /// Vault not found.
    #[error("Vault not found: {address} on chain {chain_id}")]
    VaultNotFound { address: String, chain_id: i64 },

    /// Invalid address format.
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    /// Invalid chain ID.
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(i64),

    /// RPC connection failed.
    #[error("RPC connection failed: {0}")]
    RpcConnection(String),

    /// Transaction failed.
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Insufficient balance.
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: U256, need: U256 },

    /// Invalid private key.
    #[error("Invalid private key")]
    InvalidPrivateKey,
}

/// Result type alias for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

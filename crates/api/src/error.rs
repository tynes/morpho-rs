//! Error types for the Morpho API client.

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

    /// Contract error.
    #[error("Contract error: {0}")]
    Contract(#[from] morpho_rs_contracts::ContractError),

    /// Simulation error.
    #[cfg(feature = "sim")]
    #[error("Simulation error: {0}")]
    Simulation(#[from] morpho_rs_sim::SimError),

    /// Transaction support not configured.
    #[error("Transaction support not configured: RPC URL and private key required")]
    TransactionNotConfigured,
}

impl ApiError {
    /// Returns `true` if the error is retryable (transient network errors).
    pub fn is_retryable(&self) -> bool {
        matches!(self, ApiError::Request(_))
    }
}

/// Result type alias for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_request_error() {
        // reqwest errors are retryable (transient network issues)
        let err = reqwest::Client::builder()
            .build()
            .unwrap()
            .get("http://invalid url with spaces")
            .build()
            .unwrap_err();
        let api_err = ApiError::Request(err);
        assert!(api_err.is_retryable());
    }

    #[test]
    fn test_is_retryable_graphql_error() {
        let err = ApiError::GraphQL("some error".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_parse_error() {
        let err = ApiError::Parse("bad data".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_vault_not_found() {
        let err = ApiError::VaultNotFound {
            address: "0x123".to_string(),
            chain_id: 1,
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_invalid_address() {
        let err = ApiError::InvalidAddress("bad".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_invalid_chain_id() {
        let err = ApiError::InvalidChainId(999);
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_transaction_not_configured() {
        let err = ApiError::TransactionNotConfigured;
        assert!(!err.is_retryable());
    }
}

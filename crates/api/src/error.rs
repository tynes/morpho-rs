//! Error types for the Morpho API client.
//!
//! All three crate-level error types ([`ApiError`], [`ContractError`](morpho_rs_contracts::ContractError),
//! `SimError`) share a consistent interface:
//!
//! - `is_retryable()` — whether the error may succeed on retry (transient network issues)
//! - `is_user_error()` — whether the error was caused by invalid user input
//!
//! [`ApiError`] wraps both `ContractError` and `SimError` (the latter behind the `sim` feature),
//! so callers only need to handle `ApiError` for unified error classification via
//! [`error_category()`](ApiError::error_category).

use thiserror::Error;

/// High-level classification of errors across all Morpho crates.
///
/// Use [`ApiError::error_category`] to classify any error for logging, metrics, or user-facing messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Transient network or HTTP errors (retryable).
    Network,
    /// GraphQL or API-level errors from the Morpho API.
    Api,
    /// Invalid user input (address format, chain ID, amount).
    Validation,
    /// Requested resource not found.
    NotFound,
    /// Missing configuration (RPC URL, private key).
    Configuration,
    /// On-chain contract interaction error.
    Contract,
    /// Simulation engine error.
    Simulation,
}

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
    ///
    /// Only [`ApiError::Request`] and retryable [`ApiError::Contract`] errors
    /// (e.g., RPC connection failures) are considered retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Request(_) => true,
            ApiError::Contract(e) => e.is_retryable(),
            _ => false,
        }
    }

    /// Returns `true` if the error is caused by invalid user input or configuration.
    pub fn is_user_error(&self) -> bool {
        match self {
            ApiError::InvalidAddress(_) | ApiError::InvalidChainId(_) => true,
            ApiError::TransactionNotConfigured => true,
            ApiError::Contract(e) => e.is_user_error(),
            #[cfg(feature = "sim")]
            ApiError::Simulation(e) => e.is_user_error(),
            _ => false,
        }
    }

    /// Returns the high-level [`ErrorCategory`] for this error.
    ///
    /// Useful for logging, metrics, and determining how to present the error to users.
    pub fn error_category(&self) -> ErrorCategory {
        match self {
            ApiError::Request(_) => ErrorCategory::Network,
            ApiError::GraphQL(_) | ApiError::Parse(_) => ErrorCategory::Api,
            ApiError::VaultNotFound { .. } => ErrorCategory::NotFound,
            ApiError::InvalidAddress(_) | ApiError::InvalidChainId(_) => ErrorCategory::Validation,
            ApiError::TransactionNotConfigured => ErrorCategory::Configuration,
            ApiError::Contract(_) => ErrorCategory::Contract,
            #[cfg(feature = "sim")]
            ApiError::Simulation(_) => ErrorCategory::Simulation,
        }
    }
}

/// Result type alias for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use morpho_rs_contracts::ContractError;

    // --- is_retryable tests ---

    #[test]
    fn test_is_retryable_request_error() {
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
    fn test_is_retryable_contract_rpc_error() {
        let err = ApiError::Contract(ContractError::RpcConnection("timeout".to_string()));
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_contract_tx_error() {
        let err = ApiError::Contract(ContractError::TransactionFailed("reverted".to_string()));
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_retryable_graphql_error() {
        assert!(!ApiError::GraphQL("some error".to_string()).is_retryable());
    }

    #[test]
    fn test_is_retryable_parse_error() {
        assert!(!ApiError::Parse("bad data".to_string()).is_retryable());
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
        assert!(!ApiError::InvalidAddress("bad".to_string()).is_retryable());
    }

    #[test]
    fn test_is_retryable_invalid_chain_id() {
        assert!(!ApiError::InvalidChainId(999).is_retryable());
    }

    #[test]
    fn test_is_retryable_transaction_not_configured() {
        assert!(!ApiError::TransactionNotConfigured.is_retryable());
    }

    // --- is_user_error tests ---

    #[test]
    fn test_is_user_error_invalid_address() {
        assert!(ApiError::InvalidAddress("bad".to_string()).is_user_error());
    }

    #[test]
    fn test_is_user_error_invalid_chain_id() {
        assert!(ApiError::InvalidChainId(999).is_user_error());
    }

    #[test]
    fn test_is_user_error_transaction_not_configured() {
        assert!(ApiError::TransactionNotConfigured.is_user_error());
    }

    #[test]
    fn test_is_user_error_contract_invalid_key() {
        let err = ApiError::Contract(ContractError::InvalidPrivateKey);
        assert!(err.is_user_error());
    }

    #[test]
    fn test_is_not_user_error_graphql() {
        assert!(!ApiError::GraphQL("error".to_string()).is_user_error());
    }

    #[test]
    fn test_is_not_user_error_request() {
        let err = reqwest::Client::builder()
            .build()
            .unwrap()
            .get("http://invalid url with spaces")
            .build()
            .unwrap_err();
        assert!(!ApiError::Request(err).is_user_error());
    }

    // --- error_category tests ---

    #[test]
    fn test_category_network() {
        let err = reqwest::Client::builder()
            .build()
            .unwrap()
            .get("http://invalid url with spaces")
            .build()
            .unwrap_err();
        assert_eq!(ApiError::Request(err).error_category(), ErrorCategory::Network);
    }

    #[test]
    fn test_category_api() {
        assert_eq!(
            ApiError::GraphQL("err".to_string()).error_category(),
            ErrorCategory::Api
        );
        assert_eq!(
            ApiError::Parse("err".to_string()).error_category(),
            ErrorCategory::Api
        );
    }

    #[test]
    fn test_category_not_found() {
        let err = ApiError::VaultNotFound {
            address: "0x123".to_string(),
            chain_id: 1,
        };
        assert_eq!(err.error_category(), ErrorCategory::NotFound);
    }

    #[test]
    fn test_category_validation() {
        assert_eq!(
            ApiError::InvalidAddress("bad".to_string()).error_category(),
            ErrorCategory::Validation
        );
        assert_eq!(
            ApiError::InvalidChainId(999).error_category(),
            ErrorCategory::Validation
        );
    }

    #[test]
    fn test_category_configuration() {
        assert_eq!(
            ApiError::TransactionNotConfigured.error_category(),
            ErrorCategory::Configuration
        );
    }

    #[test]
    fn test_category_contract() {
        let err = ApiError::Contract(ContractError::TransactionFailed("err".to_string()));
        assert_eq!(err.error_category(), ErrorCategory::Contract);
    }
}

//! Error types for the contracts crate.

use thiserror::Error;

/// Errors that can occur when using contract clients.
#[derive(Debug, Error)]
pub enum ContractError {
    /// RPC connection failed.
    #[error("RPC connection failed: {0}")]
    RpcConnection(String),

    /// Transaction failed.
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    /// Invalid private key.
    #[error("Invalid private key")]
    InvalidPrivateKey,
}

impl ContractError {
    /// Returns `true` if the error is retryable (transient network errors).
    ///
    /// Currently, only [`ContractError::RpcConnection`] is considered retryable,
    /// as it may indicate a transient network issue.
    pub fn is_retryable(&self) -> bool {
        matches!(self, ContractError::RpcConnection(_))
    }

    /// Returns `true` if the error is caused by invalid user input.
    ///
    /// [`ContractError::InvalidPrivateKey`] is a user configuration error.
    /// [`ContractError::TransactionFailed`] may be user error (insufficient funds, etc.)
    /// but could also be a contract-level revert, so it is not classified as user error.
    pub fn is_user_error(&self) -> bool {
        matches!(self, ContractError::InvalidPrivateKey)
    }
}

/// Result type alias for contract operations.
pub type Result<T> = std::result::Result<T, ContractError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_rpc_connection() {
        let error = ContractError::RpcConnection("connection refused".to_string());
        assert_eq!(
            error.to_string(),
            "RPC connection failed: connection refused"
        );
    }

    #[test]
    fn test_error_display_transaction_failed() {
        let error = ContractError::TransactionFailed("out of gas".to_string());
        assert_eq!(error.to_string(), "Transaction failed: out of gas");
    }

    #[test]
    fn test_error_display_invalid_private_key() {
        let error = ContractError::InvalidPrivateKey;
        assert_eq!(error.to_string(), "Invalid private key");
    }

    #[test]
    fn test_is_retryable_rpc_connection() {
        let error = ContractError::RpcConnection("timeout".to_string());
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_transaction_failed() {
        let error = ContractError::TransactionFailed("reverted".to_string());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_invalid_key() {
        assert!(!ContractError::InvalidPrivateKey.is_retryable());
    }

    #[test]
    fn test_is_user_error_invalid_key() {
        assert!(ContractError::InvalidPrivateKey.is_user_error());
    }

    #[test]
    fn test_is_not_user_error_rpc() {
        let error = ContractError::RpcConnection("timeout".to_string());
        assert!(!error.is_user_error());
    }

    #[test]
    fn test_is_not_user_error_tx_failed() {
        let error = ContractError::TransactionFailed("reverted".to_string());
        assert!(!error.is_user_error());
    }
}

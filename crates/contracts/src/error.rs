//! Error types for the contracts crate.

use alloy_primitives::U256;
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

    /// Insufficient balance.
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: U256, need: U256 },

    /// Invalid private key.
    #[error("Invalid private key")]
    InvalidPrivateKey,
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
    fn test_error_display_insufficient_balance() {
        let error = ContractError::InsufficientBalance {
            have: U256::from(100),
            need: U256::from(200),
        };
        assert_eq!(
            error.to_string(),
            "Insufficient balance: have 100, need 200"
        );
    }

    #[test]
    fn test_error_display_invalid_private_key() {
        let error = ContractError::InvalidPrivateKey;
        assert_eq!(error.to_string(), "Invalid private key");
    }
}

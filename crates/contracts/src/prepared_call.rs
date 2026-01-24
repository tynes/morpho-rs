//! Prepared call types for deferred transaction execution.
//!
//! This module provides `PreparedCall`, a type that represents a transaction
//! that has been constructed but not yet sent. This enables:
//! - Direct execution via `.send()`
//! - Integration with `safe-rs` `MulticallBuilder::add_typed()`

use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;
use alloy::sol_types::SolCall;

use crate::error::{ContractError, Result};
use crate::provider::HttpProvider;

/// A prepared transaction that can be inspected, executed, or used with MulticallBuilder.
///
/// This type is generic over the `SolCall` type, allowing type-safe integration
/// with `safe-rs` `MulticallBuilder::add_typed()`.
///
/// # Example
///
/// ```rust,ignore
/// // Direct execution
/// let receipt = client.deposit(vault, amount, receiver).send().await?;
///
/// // Integration with MulticallBuilder
/// let (addr, call) = client.deposit(vault, amount, receiver).prepare();
/// builder.add_typed(addr, call);
/// ```
pub struct PreparedCall<'a, C: SolCall> {
    to: Address,
    call: C,
    value: U256,
    provider: &'a HttpProvider,
}

impl<'a, C: SolCall> PreparedCall<'a, C> {
    /// Create a new prepared call.
    pub fn new(to: Address, call: C, value: U256, provider: &'a HttpProvider) -> Self {
        Self {
            to,
            call,
            value,
            provider,
        }
    }

    /// Consumes self and returns `(address, call)` for `MulticallBuilder::add_typed()`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (addr, call) = client.deposit(vault, amount, receiver).prepare();
    /// builder.add_typed(addr, call);
    /// ```
    pub fn prepare(self) -> (Address, C) {
        (self.to, self.call)
    }

    /// Returns the target address for this call.
    pub fn to(&self) -> Address {
        self.to
    }

    /// Returns the value (ETH) to send with this call.
    pub fn value(&self) -> U256 {
        self.value
    }

    /// Sends the transaction and waits for the receipt.
    pub async fn send(self) -> Result<TransactionReceipt> {
        use alloy::rpc::types::TransactionRequest;

        let calldata = self.call.abi_encode();
        let tx = TransactionRequest::default()
            .to(self.to)
            .input(calldata.into())
            .value(self.value);

        let pending = self.provider.send_transaction(tx).await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to send transaction: {}", e))
        })?;

        let receipt = pending.get_receipt().await.map_err(|e| {
            ContractError::TransactionFailed(format!("Failed to get receipt: {}", e))
        })?;

        Ok(receipt)
    }
}

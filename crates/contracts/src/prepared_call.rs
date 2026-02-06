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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::network::EthereumWallet;
    use alloy::signers::local::PrivateKeySigner;
    use alloy::sol;

    // Define a simple test call type
    sol! {
        #[sol(rpc)]
        interface ITestContract {
            function testFunction(uint256 value, address receiver) external returns (bool);
        }
    }

    // Anvil's default account 0 private key
    const TEST_PRIVATE_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    /// Helper to create a mock provider for testing.
    fn create_test_provider() -> HttpProvider {
        use alloy::providers::ProviderBuilder;

        let signer: PrivateKeySigner = TEST_PRIVATE_KEY.parse().expect("invalid private key");
        let wallet = EthereumWallet::from(signer);
        let url: url::Url = "http://localhost:8545".parse().unwrap();

        ProviderBuilder::new().wallet(wallet).connect_http(url)
    }

    #[test]
    fn test_to_returns_target_address() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let call = ITestContract::testFunctionCall {
            value: U256::from(100),
            receiver: Address::repeat_byte(0x01),
        };

        let prepared = PreparedCall::new(target, call, U256::ZERO, &provider);

        assert_eq!(prepared.to(), target);
    }

    #[test]
    fn test_value_returns_eth_amount() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let value = U256::from(1_000_000_000_000_000_000u64); // 1 ETH
        let call = ITestContract::testFunctionCall {
            value: U256::from(100),
            receiver: Address::repeat_byte(0x01),
        };

        let prepared = PreparedCall::new(target, call, value, &provider);

        assert_eq!(prepared.value(), value);
    }

    #[test]
    fn test_value_returns_zero_when_no_eth() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let call = ITestContract::testFunctionCall {
            value: U256::from(100),
            receiver: Address::repeat_byte(0x01),
        };

        let prepared = PreparedCall::new(target, call, U256::ZERO, &provider);

        assert_eq!(prepared.value(), U256::ZERO);
    }

    #[test]
    fn test_prepare_returns_address_and_call_tuple() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let receiver = Address::repeat_byte(0x01);
        let amount = U256::from(100);
        let call = ITestContract::testFunctionCall {
            value: amount,
            receiver,
        };

        let prepared = PreparedCall::new(target, call.clone(), U256::ZERO, &provider);
        let (addr, returned_call) = prepared.prepare();

        assert_eq!(addr, target);
        assert_eq!(returned_call.value, amount);
        assert_eq!(returned_call.receiver, receiver);
    }

    #[test]
    fn test_prepare_consumes_self() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let call = ITestContract::testFunctionCall {
            value: U256::from(100),
            receiver: Address::repeat_byte(0x01),
        };

        let prepared = PreparedCall::new(target, call, U256::ZERO, &provider);

        // After calling prepare(), the PreparedCall is consumed
        let (addr, _) = prepared.prepare();
        assert_eq!(addr, target);

        // The following would not compile because prepared was moved:
        // let _ = prepared.to();
    }

    #[test]
    fn test_new_stores_all_fields() {
        let provider = create_test_provider();
        let target = Address::repeat_byte(0x42);
        let receiver = Address::repeat_byte(0x01);
        let amount = U256::from(100);
        let value = U256::from(500);
        let call = ITestContract::testFunctionCall {
            value: amount,
            receiver,
        };

        let prepared = PreparedCall::new(target, call, value, &provider);

        // Verify all accessors return correct values
        assert_eq!(prepared.to(), target);
        assert_eq!(prepared.value(), value);
    }
}

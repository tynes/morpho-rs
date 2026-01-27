//! V1 Vault transaction client for executing deposits and withdrawals.

use crate::define_vault_transaction_client;

define_vault_transaction_client!(
    /// Client for executing transactions against V1 (MetaMorpho) vaults.
    VaultV1TransactionClient,
    "V1"
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::define_vault_client_tests;
    define_vault_client_tests!(VaultV1TransactionClient);
}

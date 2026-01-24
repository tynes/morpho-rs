//! Provider type definitions for contract clients.

use alloy::{
    network::EthereumWallet,
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, RootProvider,
    },
};

/// The recommended fillers type (default in alloy 1.x).
pub type RecommendedFillers =
    JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>;

/// The concrete provider type used by transaction clients.
/// This matches what `ProviderBuilder::new().wallet().on_http()` returns.
pub type HttpProvider = FillProvider<
    JoinFill<JoinFill<Identity, RecommendedFillers>, WalletFiller<EthereumWallet>>,
    RootProvider,
>;

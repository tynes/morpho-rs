//! Provider type definitions for contract clients.

use alloy::{
    network::{Ethereum, EthereumWallet},
    providers::{
        fillers::{
            BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
            WalletFiller,
        },
        Identity, RootProvider,
    },
    transports::http::{Client, Http},
};

/// The recommended fillers type from `with_recommended_fillers()`.
pub type RecommendedFillers =
    JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>;

/// The concrete provider type used by transaction clients.
/// This matches what `ProviderBuilder::new().with_recommended_fillers().wallet().on_http()` returns.
pub type HttpProvider = FillProvider<
    JoinFill<JoinFill<Identity, RecommendedFillers>, WalletFiller<EthereumWallet>>,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

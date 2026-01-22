//! Chain types for Morpho-supported networks.

use serde::{Deserialize, Serialize};

/// Supported blockchain networks for Morpho.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub enum Chain {
    EthMainnet,
    BaseMainnet,
    PolygonMainnet,
    ArbitrumMainnet,
    OptimismMainnet,
    WorldChainMainnet,
    FraxtalMainnet,
    ScrollMainnet,
    InkMainnet,
    Unichain,
    SonicMainnet,
    HemiMainnet,
    ModeMainnet,
    CornMainnet,
    PlumeMainnet,
    CampMainnet,
    KatanaMainnet,
    EtherlinkMainnet,
    TacMainnet,
    LiskMainnet,
    HyperliquidMainnet,
    SeiMainnet,
    ZeroGMainnet,
    LineaMainnet,
    MonadMainnet,
    StableMainnet,
    CronosMainnet,
    CeloMainnet,
    AbstractMainnet,
}

impl Chain {
    /// Returns the chain ID.
    pub const fn id(self) -> i64 {
        match self {
            Chain::EthMainnet => 1,
            Chain::BaseMainnet => 8453,
            Chain::PolygonMainnet => 137,
            Chain::ArbitrumMainnet => 42161,
            Chain::OptimismMainnet => 10,
            Chain::WorldChainMainnet => 480,
            Chain::FraxtalMainnet => 252,
            Chain::ScrollMainnet => 534352,
            Chain::InkMainnet => 57073,
            Chain::Unichain => 130,
            Chain::SonicMainnet => 146,
            Chain::HemiMainnet => 43111,
            Chain::ModeMainnet => 34443,
            Chain::CornMainnet => 21000000,
            Chain::PlumeMainnet => 98866,
            Chain::CampMainnet => 123420001114,
            Chain::KatanaMainnet => 747474,
            Chain::EtherlinkMainnet => 42793,
            Chain::TacMainnet => 239,
            Chain::LiskMainnet => 1135,
            Chain::HyperliquidMainnet => 999,
            Chain::SeiMainnet => 1329,
            Chain::ZeroGMainnet => 16661,
            Chain::LineaMainnet => 59144,
            Chain::MonadMainnet => 143,
            Chain::StableMainnet => 988,
            Chain::CronosMainnet => 25,
            Chain::CeloMainnet => 42220,
            Chain::AbstractMainnet => 2741,
        }
    }

    /// Returns the network name.
    pub const fn network(self) -> &'static str {
        match self {
            Chain::EthMainnet => "ethereum",
            Chain::BaseMainnet => "base",
            Chain::PolygonMainnet => "polygon",
            Chain::ArbitrumMainnet => "arbitrum",
            Chain::OptimismMainnet => "optimism",
            Chain::WorldChainMainnet => "worldchain",
            Chain::FraxtalMainnet => "fraxtal",
            Chain::ScrollMainnet => "scroll",
            Chain::InkMainnet => "ink",
            Chain::Unichain => "unichain",
            Chain::SonicMainnet => "sonic",
            Chain::HemiMainnet => "hemi",
            Chain::ModeMainnet => "mode",
            Chain::CornMainnet => "corn",
            Chain::PlumeMainnet => "plume",
            Chain::CampMainnet => "camp",
            Chain::KatanaMainnet => "katana",
            Chain::EtherlinkMainnet => "etherlink",
            Chain::TacMainnet => "tac",
            Chain::LiskMainnet => "lisk",
            Chain::HyperliquidMainnet => "hyperliquid",
            Chain::SeiMainnet => "sei",
            Chain::ZeroGMainnet => "zerog",
            Chain::LineaMainnet => "linea",
            Chain::MonadMainnet => "monad",
            Chain::StableMainnet => "stable",
            Chain::CronosMainnet => "cronos",
            Chain::CeloMainnet => "celo",
            Chain::AbstractMainnet => "abstract",
        }
    }

    /// Try to create a Chain from a chain ID.
    pub fn from_id(id: i64) -> Option<Chain> {
        match id {
            1 => Some(Chain::EthMainnet),
            8453 => Some(Chain::BaseMainnet),
            137 => Some(Chain::PolygonMainnet),
            42161 => Some(Chain::ArbitrumMainnet),
            10 => Some(Chain::OptimismMainnet),
            480 => Some(Chain::WorldChainMainnet),
            252 => Some(Chain::FraxtalMainnet),
            534352 => Some(Chain::ScrollMainnet),
            57073 => Some(Chain::InkMainnet),
            130 => Some(Chain::Unichain),
            146 => Some(Chain::SonicMainnet),
            43111 => Some(Chain::HemiMainnet),
            34443 => Some(Chain::ModeMainnet),
            21000000 => Some(Chain::CornMainnet),
            98866 => Some(Chain::PlumeMainnet),
            123420001114 => Some(Chain::CampMainnet),
            747474 => Some(Chain::KatanaMainnet),
            42793 => Some(Chain::EtherlinkMainnet),
            239 => Some(Chain::TacMainnet),
            1135 => Some(Chain::LiskMainnet),
            999 => Some(Chain::HyperliquidMainnet),
            1329 => Some(Chain::SeiMainnet),
            16661 => Some(Chain::ZeroGMainnet),
            59144 => Some(Chain::LineaMainnet),
            143 => Some(Chain::MonadMainnet),
            988 => Some(Chain::StableMainnet),
            25 => Some(Chain::CronosMainnet),
            42220 => Some(Chain::CeloMainnet),
            2741 => Some(Chain::AbstractMainnet),
            _ => None,
        }
    }

    /// Returns all supported chains.
    pub const fn all() -> &'static [Chain] {
        &[
            Chain::EthMainnet,
            Chain::BaseMainnet,
            Chain::PolygonMainnet,
            Chain::ArbitrumMainnet,
            Chain::OptimismMainnet,
            Chain::WorldChainMainnet,
            Chain::FraxtalMainnet,
            Chain::ScrollMainnet,
            Chain::InkMainnet,
            Chain::Unichain,
            Chain::SonicMainnet,
            Chain::HemiMainnet,
            Chain::ModeMainnet,
            Chain::CornMainnet,
            Chain::PlumeMainnet,
            Chain::CampMainnet,
            Chain::KatanaMainnet,
            Chain::EtherlinkMainnet,
            Chain::TacMainnet,
            Chain::LiskMainnet,
            Chain::HyperliquidMainnet,
            Chain::SeiMainnet,
            Chain::ZeroGMainnet,
            Chain::LineaMainnet,
            Chain::MonadMainnet,
            Chain::StableMainnet,
            Chain::CronosMainnet,
            Chain::CeloMainnet,
            Chain::AbstractMainnet,
        ]
    }
}

impl From<Chain> for i64 {
    fn from(chain: Chain) -> i64 {
        chain.id()
    }
}

impl TryFrom<i64> for Chain {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Chain::from_id(id).ok_or_else(|| format!("Unknown chain ID: {}", id))
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.network(), self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_roundtrip() {
        for chain in Chain::all() {
            let id = chain.id();
            let recovered = Chain::from_id(id).unwrap();
            assert_eq!(*chain, recovered);
        }
    }

    #[test]
    fn test_chain_display() {
        assert_eq!(Chain::EthMainnet.to_string(), "ethereum (1)");
        assert_eq!(Chain::BaseMainnet.to_string(), "base (8453)");
    }
}

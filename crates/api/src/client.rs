//! Vault client implementations for V1 and V2 vaults.

use graphql_client::{GraphQLQuery, Response};
use reqwest::Client;
use url::Url;

use crate::error::{ApiError, Result};
use crate::filters::{VaultFiltersV1, VaultFiltersV2};
use crate::queries::v1::{
    get_vault_v1_by_address, get_vaults_v1, GetVaultV1ByAddress, GetVaultsV1,
};
use crate::queries::user::{
    get_user_account_overview, get_user_vault_positions, GetUserAccountOverview,
    GetUserVaultPositions,
};
use crate::queries::v2::{
    get_vault_v2_by_address, get_vaults_v2, GetVaultV2ByAddress, GetVaultsV2,
};
use crate::types::{
    Asset, Chain, MarketInfo, UserAccountOverview, UserMarketPosition, UserState,
    UserVaultPositions, UserVaultV1Position, UserVaultV2Position, Vault, VaultAdapter,
    VaultAllocation, VaultAllocator, VaultInfo, VaultPositionState, VaultReward, VaultStateV1,
    VaultV1, VaultV2, VaultV2Warning, VaultWarning,
};

/// Default Morpho GraphQL API endpoint.
pub const DEFAULT_API_URL: &str = "https://api.morpho.org/graphql";

/// Default page size for paginated queries.
pub const DEFAULT_PAGE_SIZE: i64 = 100;

/// Configuration for vault clients.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// GraphQL API URL.
    pub api_url: Url,
    /// Default page size for queries.
    pub page_size: i64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_url: Url::parse(DEFAULT_API_URL).expect("Invalid default API URL"),
            page_size: DEFAULT_PAGE_SIZE,
        }
    }
}

impl ClientConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom API URL.
    pub fn with_api_url(mut self, url: Url) -> Self {
        self.api_url = url;
        self
    }

    /// Set a custom page size.
    pub fn with_page_size(mut self, size: i64) -> Self {
        self.page_size = size;
        self
    }
}

/// Client for querying V1 (MetaMorpho) vaults.
#[derive(Debug, Clone)]
pub struct VaultV1Client {
    http_client: Client,
    config: ClientConfig,
}

impl Default for VaultV1Client {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultV1Client {
    /// Create a new V1 vault client with default configuration.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            config: ClientConfig::default(),
        }
    }

    /// Create a new V1 vault client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            http_client: Client::new(),
            config,
        }
    }

    /// Execute a GraphQL query.
    async fn execute<Q: GraphQLQuery>(
        &self,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData> {
        let request_body = Q::build_query(variables);
        let response = self
            .http_client
            .post(self.config.api_url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<Q::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            if !errors.is_empty() {
                return Err(ApiError::GraphQL(
                    errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
            }
        }

        response_body
            .data
            .ok_or_else(|| ApiError::Parse("No data in response".to_string()))
    }

    /// Get V1 vaults with optional filters.
    pub async fn get_vaults(&self, filters: Option<VaultFiltersV1>) -> Result<Vec<VaultV1>> {
        let variables = get_vaults_v1::Variables {
            first: Some(self.config.page_size),
            skip: Some(0),
            where_: filters.map(|f| f.to_gql()),
        };

        let data = self.execute::<GetVaultsV1>(variables).await?;

        let items = match data.vaults.items {
            Some(items) => items,
            None => return Ok(Vec::new()),
        };

        let vaults: Vec<VaultV1> = items
            .into_iter()
            .filter_map(convert_v1_vault)
            .collect();

        Ok(vaults)
    }

    /// Get a single V1 vault by address and chain.
    pub async fn get_vault(&self, address: &str, chain: Chain) -> Result<VaultV1> {
        let variables = get_vault_v1_by_address::Variables {
            address: address.to_string(),
            chain_id: chain.id() as i64,
        };

        let data = self.execute::<GetVaultV1ByAddress>(variables).await?;

        convert_v1_vault_single(data.vault_by_address).ok_or_else(|| ApiError::VaultNotFound {
            address: address.to_string(),
            chain_id: chain.id(),
        })
    }

    /// Get V1 vaults on a specific chain.
    pub async fn get_vaults_by_chain(&self, chain: Chain) -> Result<Vec<VaultV1>> {
        let filters = VaultFiltersV1::new().chain(chain);
        self.get_vaults(Some(filters)).await
    }

    /// Get V1 vaults by curator address.
    pub async fn get_vaults_by_curator(
        &self,
        curator: &str,
        chain: Option<Chain>,
    ) -> Result<Vec<VaultV1>> {
        let mut filters = VaultFiltersV1::new().curators([curator]);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }

    /// Get whitelisted (listed) V1 vaults.
    pub async fn get_whitelisted_vaults(&self, chain: Option<Chain>) -> Result<Vec<VaultV1>> {
        let mut filters = VaultFiltersV1::new().listed(true);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }
}

/// Client for querying V2 vaults.
#[derive(Debug, Clone)]
pub struct VaultV2Client {
    http_client: Client,
    config: ClientConfig,
}

impl Default for VaultV2Client {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultV2Client {
    /// Create a new V2 vault client with default configuration.
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
            config: ClientConfig::default(),
        }
    }

    /// Create a new V2 vault client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            http_client: Client::new(),
            config,
        }
    }

    /// Execute a GraphQL query.
    async fn execute<Q: GraphQLQuery>(
        &self,
        variables: Q::Variables,
    ) -> Result<Q::ResponseData> {
        let request_body = Q::build_query(variables);
        let response = self
            .http_client
            .post(self.config.api_url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<Q::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            if !errors.is_empty() {
                return Err(ApiError::GraphQL(
                    errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
            }
        }

        response_body
            .data
            .ok_or_else(|| ApiError::Parse("No data in response".to_string()))
    }

    /// Get V2 vaults with optional filters.
    pub async fn get_vaults(&self, filters: Option<VaultFiltersV2>) -> Result<Vec<VaultV2>> {
        let variables = get_vaults_v2::Variables {
            first: Some(self.config.page_size),
            skip: Some(0),
            where_: filters.map(|f| f.to_gql()),
        };

        let data = self.execute::<GetVaultsV2>(variables).await?;

        let items = match data.vault_v2s.items {
            Some(items) => items,
            None => return Ok(Vec::new()),
        };

        let vaults: Vec<VaultV2> = items
            .into_iter()
            .filter_map(convert_v2_vault)
            .collect();

        Ok(vaults)
    }

    /// Get a single V2 vault by address and chain.
    pub async fn get_vault(&self, address: &str, chain: Chain) -> Result<VaultV2> {
        let variables = get_vault_v2_by_address::Variables {
            address: address.to_string(),
            chain_id: chain.id() as i64,
        };

        let data = self.execute::<GetVaultV2ByAddress>(variables).await?;

        convert_v2_vault_single(data.vault_v2_by_address).ok_or_else(|| ApiError::VaultNotFound {
            address: address.to_string(),
            chain_id: chain.id(),
        })
    }

    /// Get V2 vaults on a specific chain.
    pub async fn get_vaults_by_chain(&self, chain: Chain) -> Result<Vec<VaultV2>> {
        let filters = VaultFiltersV2::new().chain(chain);
        self.get_vaults(Some(filters)).await
    }

    /// Get whitelisted (listed) V2 vaults.
    pub async fn get_whitelisted_vaults(&self, chain: Option<Chain>) -> Result<Vec<VaultV2>> {
        let mut filters = VaultFiltersV2::new().listed(true);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }
}

/// Combined client for querying both V1 and V2 vaults.
#[derive(Debug, Clone)]
pub struct VaultClient {
    /// V1 vault client.
    pub v1: VaultV1Client,
    /// V2 vault client.
    pub v2: VaultV2Client,
}

impl Default for VaultClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VaultClient {
    /// Create a new combined vault client with default configuration.
    pub fn new() -> Self {
        Self {
            v1: VaultV1Client::new(),
            v2: VaultV2Client::new(),
        }
    }

    /// Create a new combined vault client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            v1: VaultV1Client::with_config(config.clone()),
            v2: VaultV2Client::with_config(config),
        }
    }

    /// Get vaults (V1 and V2) on a specific chain as unified Vault type.
    pub async fn get_vaults_by_chain(&self, chain: Chain) -> Result<Vec<Vault>> {
        let (v1_vaults, v2_vaults) = tokio::try_join!(
            self.v1.get_vaults_by_chain(chain),
            self.v2.get_vaults_by_chain(chain),
        )?;

        let mut vaults: Vec<Vault> = Vec::with_capacity(v1_vaults.len() + v2_vaults.len());
        vaults.extend(v1_vaults.into_iter().map(Vault::from));
        vaults.extend(v2_vaults.into_iter().map(Vault::from));

        Ok(vaults)
    }

    /// Get whitelisted vaults (V1 and V2) as unified Vault type.
    pub async fn get_whitelisted_vaults(&self, chain: Option<Chain>) -> Result<Vec<Vault>> {
        let (v1_vaults, v2_vaults) = tokio::try_join!(
            self.v1.get_whitelisted_vaults(chain),
            self.v2.get_whitelisted_vaults(chain),
        )?;

        let mut vaults: Vec<Vault> = Vec::with_capacity(v1_vaults.len() + v2_vaults.len());
        vaults.extend(v1_vaults.into_iter().map(Vault::from));
        vaults.extend(v2_vaults.into_iter().map(Vault::from));

        Ok(vaults)
    }

    /// Execute a GraphQL query.
    async fn execute<Q: GraphQLQuery>(&self, variables: Q::Variables) -> Result<Q::ResponseData> {
        let request_body = Q::build_query(variables);
        let response = self
            .v1
            .http_client
            .post(self.v1.config.api_url.as_str())
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<Q::ResponseData> = response.json().await?;

        if let Some(errors) = response_body.errors {
            if !errors.is_empty() {
                return Err(ApiError::GraphQL(
                    errors
                        .iter()
                        .map(|e| e.message.clone())
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
            }
        }

        response_body
            .data
            .ok_or_else(|| ApiError::Parse("No data in response".to_string()))
    }

    /// Get all vault positions (V1 and V2) for a user.
    ///
    /// If `chain` is `Some`, queries only that chain.
    /// If `chain` is `None`, queries all supported chains and aggregates results.
    pub async fn get_user_vault_positions(
        &self,
        address: &str,
        chain: Option<Chain>,
    ) -> Result<UserVaultPositions> {
        match chain {
            Some(c) => self.get_user_vault_positions_single_chain(address, c).await,
            None => self.get_user_vault_positions_all_chains(address).await,
        }
    }

    /// Get vault positions for a user on a single chain.
    async fn get_user_vault_positions_single_chain(
        &self,
        address: &str,
        chain: Chain,
    ) -> Result<UserVaultPositions> {
        let variables = get_user_vault_positions::Variables {
            address: address.to_string(),
            chain_id: chain.id(),
        };

        let data = self.execute::<GetUserVaultPositions>(variables).await?;
        let user = data.user_by_address;

        let vault_positions: Vec<UserVaultV1Position> = user
            .vault_positions
            .into_iter()
            .filter_map(convert_user_vault_v1_position)
            .collect();

        let vault_v2_positions: Vec<UserVaultV2Position> = user
            .vault_v2_positions
            .into_iter()
            .filter_map(convert_user_vault_v2_position)
            .collect();

        Ok(UserVaultPositions {
            address: user
                .address
                .parse()
                .map_err(|_| ApiError::Parse("Invalid address".to_string()))?,
            vault_positions,
            vault_v2_positions,
        })
    }

    /// Get vault positions for a user across all chains.
    async fn get_user_vault_positions_all_chains(
        &self,
        address: &str,
    ) -> Result<UserVaultPositions> {
        use futures::future::join_all;

        // Filter chains to those with IDs that fit in GraphQL Int (32-bit signed)
        let valid_chains: Vec<_> = Chain::all()
            .iter()
            .filter(|chain| chain.id() <= i32::MAX as i64)
            .copied()
            .collect();

        let futures: Vec<_> = valid_chains
            .iter()
            .map(|chain| self.get_user_vault_positions_single_chain(address, *chain))
            .collect();

        let results = join_all(futures).await;

        let parsed_address = address
            .parse()
            .map_err(|_| ApiError::Parse("Invalid address".to_string()))?;

        let mut all_v1_positions = Vec::new();
        let mut all_v2_positions = Vec::new();

        for result in results {
            match result {
                Ok(positions) => {
                    all_v1_positions.extend(positions.vault_positions);
                    all_v2_positions.extend(positions.vault_v2_positions);
                }
                // Ignore "No results" errors - user just has no positions on that chain
                Err(ApiError::GraphQL(msg)) if msg.contains("No results") => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(UserVaultPositions {
            address: parsed_address,
            vault_positions: all_v1_positions,
            vault_v2_positions: all_v2_positions,
        })
    }

    /// Get complete account overview for a user on a specific chain.
    pub async fn get_user_account_overview(
        &self,
        address: &str,
        chain: Chain,
    ) -> Result<UserAccountOverview> {
        let variables = get_user_account_overview::Variables {
            address: address.to_string(),
            chain_id: chain.id(),
        };

        let data = self.execute::<GetUserAccountOverview>(variables).await?;
        let user = data.user_by_address;

        let state = UserState::from_gql(
            user.state.vaults_pnl_usd,
            user.state.vaults_roe_usd,
            user.state.vaults_assets_usd,
            user.state.vault_v2s_pnl_usd,
            user.state.vault_v2s_roe_usd,
            user.state.vault_v2s_assets_usd,
            user.state.markets_pnl_usd,
            user.state.markets_roe_usd,
            user.state.markets_supply_pnl_usd,
            user.state.markets_supply_roe_usd,
            user.state.markets_borrow_pnl_usd,
            user.state.markets_borrow_roe_usd,
            user.state.markets_collateral_pnl_usd,
            user.state.markets_collateral_roe_usd,
            user.state.markets_margin_pnl_usd,
            user.state.markets_margin_roe_usd,
            user.state.markets_collateral_usd,
            user.state.markets_supply_assets_usd,
            user.state.markets_borrow_assets_usd,
            user.state.markets_margin_usd,
        );

        let vault_positions: Vec<UserVaultV1Position> = user
            .vault_positions
            .into_iter()
            .filter_map(convert_user_vault_v1_position_overview)
            .collect();

        let vault_v2_positions: Vec<UserVaultV2Position> = user
            .vault_v2_positions
            .into_iter()
            .filter_map(convert_user_vault_v2_position_overview)
            .collect();

        let market_positions: Vec<UserMarketPosition> = user
            .market_positions
            .into_iter()
            .filter_map(convert_user_market_position)
            .collect();

        Ok(UserAccountOverview {
            address: user
                .address
                .parse()
                .map_err(|_| ApiError::Parse("Invalid address".to_string()))?,
            state,
            vault_positions,
            vault_v2_positions,
            market_positions,
        })
    }
}

// Conversion functions from GraphQL types to our types

fn convert_v1_vault(v: get_vaults_v1::GetVaultsV1VaultsItems) -> Option<VaultV1> {
    let chain_id = v.chain.id;
    let asset = &v.asset;

    VaultV1::from_gql(
        &v.address,
        v.name,
        v.symbol,
        chain_id,
        v.listed,
        v.featured,
        v.whitelisted,
        Asset::from_gql(
            &asset.address,
            asset.symbol.clone(),
            Some(asset.name.clone()),
            asset.decimals,
            asset.price_usd,
        )?,
        v.state.as_ref().and_then(convert_v1_state),
        v.allocators
            .into_iter()
            .filter_map(|a| VaultAllocator::from_gql(&a.address))
            .collect(),
        v.warnings
            .into_iter()
            .map(|w| VaultWarning {
                warning_type: w.type_.clone(),
                level: format!("{:?}", w.level),
            })
            .collect(),
    )
}

fn convert_v1_vault_single(
    v: get_vault_v1_by_address::GetVaultV1ByAddressVaultByAddress,
) -> Option<VaultV1> {
    let chain_id = v.chain.id;
    let asset = &v.asset;

    VaultV1::from_gql(
        &v.address,
        v.name,
        v.symbol,
        chain_id,
        v.listed,
        v.featured,
        v.whitelisted,
        Asset::from_gql(
            &asset.address,
            asset.symbol.clone(),
            Some(asset.name.clone()),
            asset.decimals,
            asset.price_usd,
        )?,
        v.state.as_ref().and_then(convert_v1_state_single),
        v.allocators
            .into_iter()
            .filter_map(|a| VaultAllocator::from_gql(&a.address))
            .collect(),
        v.warnings
            .into_iter()
            .map(|w| VaultWarning {
                warning_type: w.type_.clone(),
                level: format!("{:?}", w.level),
            })
            .collect(),
    )
}

fn convert_v1_state(s: &get_vaults_v1::GetVaultsV1VaultsItemsState) -> Option<VaultStateV1> {
    VaultStateV1::from_gql(
        Some(s.curator.as_str()),
        Some(s.owner.as_str()),
        Some(s.guardian.as_str()),
        &s.total_assets,
        s.total_assets_usd,
        &s.total_supply,
        s.fee,
        &s.timelock,
        s.apy,
        s.net_apy,
        s.share_price.as_deref().unwrap_or("0"),
        s.allocation
            .iter()
            .filter_map(|a| {
                let market = &a.market;
                VaultAllocation::from_gql(
                    market.unique_key.clone(),
                    Some(market.loan_asset.symbol.clone()),
                    Some(market.loan_asset.address.as_str()),
                    market.collateral_asset.as_ref().map(|ca| ca.symbol.clone()),
                    market.collateral_asset.as_ref().map(|ca| ca.address.as_str()),
                    &a.supply_assets,
                    a.supply_assets_usd,
                    &a.supply_cap,
                )
            })
            .collect(),
    )
}

fn convert_v1_state_single(
    s: &get_vault_v1_by_address::GetVaultV1ByAddressVaultByAddressState,
) -> Option<VaultStateV1> {
    VaultStateV1::from_gql(
        Some(s.curator.as_str()),
        Some(s.owner.as_str()),
        Some(s.guardian.as_str()),
        &s.total_assets,
        s.total_assets_usd,
        &s.total_supply,
        s.fee,
        &s.timelock,
        s.apy,
        s.net_apy,
        s.share_price.as_deref().unwrap_or("0"),
        s.allocation
            .iter()
            .filter_map(|a| {
                let market = &a.market;
                VaultAllocation::from_gql(
                    market.unique_key.clone(),
                    Some(market.loan_asset.symbol.clone()),
                    Some(market.loan_asset.address.as_str()),
                    market.collateral_asset.as_ref().map(|ca| ca.symbol.clone()),
                    market.collateral_asset.as_ref().map(|ca| ca.address.as_str()),
                    &a.supply_assets,
                    a.supply_assets_usd,
                    &a.supply_cap,
                )
            })
            .collect(),
    )
}

fn convert_v2_vault(v: get_vaults_v2::GetVaultsV2VaultV2sItems) -> Option<VaultV2> {
    let chain_id = v.chain.id;
    let asset = &v.asset;

    VaultV2::from_gql(
        &v.address,
        v.name,
        v.symbol,
        chain_id,
        v.listed,
        v.whitelisted,
        Asset::from_gql(
            &asset.address,
            asset.symbol.clone(),
            Some(asset.name.clone()),
            asset.decimals,
            asset.price_usd,
        )?,
        Some(v.curator.address.as_str()),
        Some(v.owner.address.as_str()),
        v.total_assets.as_deref().unwrap_or("0"),
        v.total_assets_usd,
        &v.total_supply,
        Some(v.share_price),
        Some(v.performance_fee),
        Some(v.management_fee),
        v.avg_apy,
        v.avg_net_apy,
        v.apy,
        v.net_apy,
        &v.liquidity,
        v.liquidity_usd,
        v.adapters
            .items
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(convert_v2_adapter)
                    .collect()
            })
            .unwrap_or_default(),
        v.rewards
            .into_iter()
            .filter_map(|r| {
                VaultReward::from_gql(
                    &r.asset.address,
                    r.asset.symbol.clone(),
                    r.supply_apr,
                    parse_yearly_supply(&r.yearly_supply_tokens),
                )
            })
            .collect(),
        v.warnings
            .into_iter()
            .map(|w| VaultV2Warning {
                warning_type: w.type_.clone(),
                level: format!("{:?}", w.level),
            })
            .collect(),
    )
}

/// Parse yearly supply tokens from string to f64.
fn parse_yearly_supply(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

fn convert_v2_vault_single(
    v: get_vault_v2_by_address::GetVaultV2ByAddressVaultV2ByAddress,
) -> Option<VaultV2> {
    let chain_id = v.chain.id;
    let asset = &v.asset;

    VaultV2::from_gql(
        &v.address,
        v.name,
        v.symbol,
        chain_id,
        v.listed,
        v.whitelisted,
        Asset::from_gql(
            &asset.address,
            asset.symbol.clone(),
            Some(asset.name.clone()),
            asset.decimals,
            asset.price_usd,
        )?,
        Some(v.curator.address.as_str()),
        Some(v.owner.address.as_str()),
        v.total_assets.as_deref().unwrap_or("0"),
        v.total_assets_usd,
        &v.total_supply,
        Some(v.share_price),
        Some(v.performance_fee),
        Some(v.management_fee),
        v.avg_apy,
        v.avg_net_apy,
        v.apy,
        v.net_apy,
        &v.liquidity,
        v.liquidity_usd,
        v.adapters
            .items
            .map(|items| {
                items
                    .into_iter()
                    .filter_map(convert_v2_adapter_single)
                    .collect()
            })
            .unwrap_or_default(),
        v.rewards
            .into_iter()
            .filter_map(|r| {
                VaultReward::from_gql(
                    &r.asset.address,
                    r.asset.symbol.clone(),
                    r.supply_apr,
                    parse_yearly_supply(&r.yearly_supply_tokens),
                )
            })
            .collect(),
        v.warnings
            .into_iter()
            .map(|w| VaultV2Warning {
                warning_type: w.type_.clone(),
                level: format!("{:?}", w.level),
            })
            .collect(),
    )
}

fn convert_v2_adapter(
    a: get_vaults_v2::GetVaultsV2VaultV2sItemsAdaptersItems,
) -> Option<VaultAdapter> {
    VaultAdapter::from_gql(
        a.id,
        &a.address,
        format!("{:?}", a.type_),
        &a.assets,
        a.assets_usd,
    )
}

fn convert_v2_adapter_single(
    a: get_vault_v2_by_address::GetVaultV2ByAddressVaultV2ByAddressAdaptersItems,
) -> Option<VaultAdapter> {
    VaultAdapter::from_gql(
        a.id,
        &a.address,
        format!("{:?}", a.type_),
        &a.assets,
        a.assets_usd,
    )
}

// User position conversion functions

fn convert_user_vault_v1_position(
    p: get_user_vault_positions::GetUserVaultPositionsUserByAddressVaultPositions,
) -> Option<UserVaultV1Position> {
    let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol)?;

    let state = p.state.as_ref().and_then(|s| {
        VaultPositionState::from_gql(
            &s.shares,
            s.assets.as_deref(),
            s.assets_usd,
            s.pnl.as_deref(),
            s.pnl_usd,
            s.roe,
            s.roe_usd,
        )
    });

    UserVaultV1Position::from_gql(p.id, &p.shares, &p.assets, p.assets_usd, vault, state)
}

fn convert_user_vault_v2_position(
    p: get_user_vault_positions::GetUserVaultPositionsUserByAddressVaultV2Positions,
) -> Option<UserVaultV2Position> {
    let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol)?;

    UserVaultV2Position::from_gql(
        p.id,
        &p.shares,
        &p.assets,
        p.assets_usd,
        p.pnl.as_deref(),
        p.pnl_usd,
        p.roe,
        p.roe_usd,
        vault,
    )
}

fn convert_user_vault_v1_position_overview(
    p: get_user_account_overview::GetUserAccountOverviewUserByAddressVaultPositions,
) -> Option<UserVaultV1Position> {
    let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol)?;

    let state = p.state.as_ref().and_then(|s| {
        VaultPositionState::from_gql(
            &s.shares,
            s.assets.as_deref(),
            s.assets_usd,
            s.pnl.as_deref(),
            s.pnl_usd,
            s.roe,
            s.roe_usd,
        )
    });

    UserVaultV1Position::from_gql(p.id, &p.shares, &p.assets, p.assets_usd, vault, state)
}

fn convert_user_vault_v2_position_overview(
    p: get_user_account_overview::GetUserAccountOverviewUserByAddressVaultV2Positions,
) -> Option<UserVaultV2Position> {
    let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol)?;

    UserVaultV2Position::from_gql(
        p.id,
        &p.shares,
        &p.assets,
        p.assets_usd,
        p.pnl.as_deref(),
        p.pnl_usd,
        p.roe,
        p.roe_usd,
        vault,
    )
}

fn convert_user_market_position(
    p: get_user_account_overview::GetUserAccountOverviewUserByAddressMarketPositions,
) -> Option<UserMarketPosition> {
    let market = MarketInfo::from_gql(
        p.market.unique_key,
        Some(p.market.loan_asset.symbol),
        Some(p.market.loan_asset.address.as_str()),
        p.market.collateral_asset.as_ref().map(|c| c.symbol.clone()),
        p.market.collateral_asset.as_ref().map(|c| c.address.as_str()),
    );

    UserMarketPosition::from_gql(
        p.id,
        &p.supply_shares,
        &p.supply_assets,
        p.supply_assets_usd,
        &p.borrow_shares,
        &p.borrow_assets,
        p.borrow_assets_usd,
        &p.collateral,
        p.collateral_usd,
        p.health_factor,
        market,
    )
}

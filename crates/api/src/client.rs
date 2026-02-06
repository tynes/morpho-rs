//! Vault client implementations for V1 and V2 vaults.

use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionReceipt;
use graphql_client::{GraphQLQuery, Response};
use morpho_rs_contracts::{Erc4626Client, VaultV1TransactionClient, VaultV2TransactionClient};
use reqwest::Client;
use url::Url;

use crate::error::{ApiError, Result};
use crate::filters::{VaultFiltersV1, VaultFiltersV2, VaultQueryOptionsV1, VaultQueryOptionsV2};
use crate::types::ordering::{OrderDirection, VaultOrderByV1, VaultOrderByV2};
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
use crate::types::vault_v1::MarketStateV1;
use crate::types::vault_v2::{MarketStateV2, MetaMorphoAllocation, MorphoMarketPosition, VaultAdapterData};
use crate::types::{
    Asset, MarketInfo, NamedChain, UserAccountOverview, UserMarketPosition,
    UserState, UserVaultPositions, UserVaultV1Position, UserVaultV2Position, VaultAdapter,
    VaultAllocation, VaultAllocator, VaultInfo, VaultPositionState,
    VaultReward, VaultStateV1, VaultV1, VaultV2, VaultV2Warning,
    VaultWarning, SUPPORTED_CHAINS,
};
use crate::types::vault::Vault as VaultTrait;

/// Macro to define a vault API client struct with common infrastructure.
///
/// Generates:
/// - The struct with `http_client: Client` and `config: ClientConfig`
/// - `Default` implementation
/// - `new()` and `with_config()` constructors
/// - `execute()` method for GraphQL queries
/// - `http_client()` and `config()` accessors
macro_rules! define_vault_client_core {
    (
        $(#[$meta:meta])*
        $client_name:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $client_name {
            http_client: Client,
            config: ClientConfig,
        }

        impl Default for $client_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $client_name {
            /// Create a new vault client with default configuration.
            pub fn new() -> Self {
                let config = ClientConfig::default();
                let http_client = Client::builder()
                    .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
                    .build()
                    .expect("Failed to build HTTP client");
                Self { http_client, config }
            }

            /// Create a new vault client with custom configuration.
            pub fn with_config(config: ClientConfig) -> Self {
                let http_client = Client::builder()
                    .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
                    .build()
                    .expect("Failed to build HTTP client");
                Self { http_client, config }
            }

            /// Get a reference to the HTTP client.
            pub fn http_client(&self) -> &Client {
                &self.http_client
            }

            /// Get a reference to the client configuration.
            pub fn config(&self) -> &ClientConfig {
                &self.config
            }

            /// Execute a GraphQL query with retry/backoff for transient failures.
            async fn execute<Q: GraphQLQuery>(
                &self,
                variables: Q::Variables,
            ) -> Result<Q::ResponseData>
            where
                Q::Variables: serde::Serialize,
            {
                let request_body = Q::build_query(variables);

                for attempt in 0..=self.config.max_retries {
                    if attempt > 0 {
                        let delay = self.config.retry_base_delay_ms * 2u64.pow(attempt - 1);
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                    }

                    let send_result = self
                        .http_client
                        .post(self.config.api_url.as_str())
                        .json(&request_body)
                        .send()
                        .await;

                    let response = match send_result {
                        Ok(resp) => resp,
                        Err(e) => {
                            let err = ApiError::Request(e);
                            if err.is_retryable() && attempt < self.config.max_retries {
                                continue;
                            }
                            return Err(err);
                        }
                    };

                    let json_result: std::result::Result<Response<Q::ResponseData>, _> =
                        response.json().await;

                    let response_body = match json_result {
                        Ok(body) => body,
                        Err(e) => {
                            let err = ApiError::Request(e);
                            if err.is_retryable() && attempt < self.config.max_retries {
                                continue;
                            }
                            return Err(err);
                        }
                    };

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

                    return response_body
                        .data
                        .ok_or_else(|| ApiError::Parse("No data in response".to_string()));
                }

                unreachable!("retry loop always returns")
            }
        }
    };
}

/// Macro to define vault operations wrapper with common transaction methods.
///
/// Generates a wrapper struct with:
/// - `deposit()`, `withdraw()`, `balance()` methods
/// - `approve()`, `get_allowance()`, `get_asset()`, `get_decimals()` methods
/// - `signer_address()`, `auto_approve()` accessors
macro_rules! define_vault_operations {
    (
        $(#[$meta:meta])*
        $ops_name:ident,
        $tx_client:ty
    ) => {
        $(#[$meta])*
        pub struct $ops_name<'a> {
            client: &'a $tx_client,
            auto_approve: bool,
        }

        impl<'a> $ops_name<'a> {
            /// Create a new operations wrapper.
            fn new(client: &'a $tx_client, auto_approve: bool) -> Self {
                Self { client, auto_approve }
            }

            /// Deposit assets into a vault, receiving shares to the signer's address.
            ///
            /// If `auto_approve` is enabled (default), this will approve the deposit amount
            /// if the current allowance is insufficient.
            pub async fn deposit(&self, vault: Address, amount: U256) -> Result<TransactionReceipt> {
                if self.auto_approve {
                    let asset = self.client.get_asset(vault).await?;
                    if let Some(approval) = self.client.approve_if_needed(asset, vault, amount).await? {
                        approval.send().await?;
                    }
                }

                let receipt = self
                    .client
                    .deposit(vault, amount, self.client.signer_address())
                    .send()
                    .await?;
                Ok(receipt)
            }

            /// Withdraw assets from a vault to the signer's address (withdrawing signer's shares).
            pub async fn withdraw(&self, vault: Address, amount: U256) -> Result<TransactionReceipt> {
                let signer = self.client.signer_address();
                let receipt = self.client.withdraw(vault, amount, signer, signer).send().await?;
                Ok(receipt)
            }

            /// Get the signer's vault share balance.
            pub async fn balance(&self, vault: Address) -> Result<U256> {
                let balance = self
                    .client
                    .get_balance(vault, self.client.signer_address())
                    .await?;
                Ok(balance)
            }

            /// Approve a vault to spend the signer's tokens if needed.
            /// Returns the transaction receipt if approval was performed, None if already approved.
            pub async fn approve(
                &self,
                vault: Address,
                amount: U256,
            ) -> Result<Option<TransactionReceipt>> {
                let asset = self.client.get_asset(vault).await?;
                if let Some(approval) = self.client.approve_if_needed(asset, vault, amount).await? {
                    let receipt = approval.send().await?;
                    Ok(Some(receipt))
                } else {
                    Ok(None)
                }
            }

            /// Get the current allowance for the vault to spend the signer's tokens.
            pub async fn get_allowance(&self, vault: Address) -> Result<U256> {
                let asset = self.client.get_asset(vault).await?;
                let allowance = self
                    .client
                    .get_allowance(asset, self.client.signer_address(), vault)
                    .await?;
                Ok(allowance)
            }

            /// Get the underlying asset address of a vault.
            pub async fn get_asset(&self, vault: Address) -> Result<Address> {
                let asset = self.client.get_asset(vault).await?;
                Ok(asset)
            }

            /// Get the decimals of a token.
            pub async fn get_decimals(&self, token: Address) -> Result<u8> {
                let decimals = self.client.get_decimals(token).await?;
                Ok(decimals)
            }

            /// Get the maximum amount of assets the signer can withdraw from a vault.
            pub async fn max_withdraw(&self, vault: Address) -> Result<U256> {
                let max = self.client.max_withdraw(vault, self.client.signer_address()).await?;
                Ok(max)
            }

            /// Convert a share amount to the equivalent asset amount.
            pub async fn convert_to_assets(&self, vault: Address, shares: U256) -> Result<U256> {
                let assets = self.client.convert_to_assets(vault, shares).await?;
                Ok(assets)
            }

            /// Get the signer's address.
            pub fn signer_address(&self) -> Address {
                self.client.signer_address()
            }

            /// Check if auto_approve is enabled.
            pub fn auto_approve(&self) -> bool {
                self.auto_approve
            }
        }
    };
}

/// Default Morpho GraphQL API endpoint.
pub const DEFAULT_API_URL: &str = "https://api.morpho.org/graphql";

/// Default page size for paginated queries.
pub const DEFAULT_PAGE_SIZE: i64 = 100;

/// Maximum number of pagination pages to prevent runaway loops.
const MAX_PAGINATION_PAGES: usize = 50;

/// Configuration for vault clients.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// GraphQL API URL.
    pub api_url: Url,
    /// Default page size for queries.
    pub page_size: i64,
    /// Maximum number of retry attempts for transient failures.
    pub max_retries: u32,
    /// Base delay in milliseconds between retries (doubled each attempt).
    pub retry_base_delay_ms: u64,
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_url: Url::parse(DEFAULT_API_URL).expect("Invalid default API URL"),
            page_size: DEFAULT_PAGE_SIZE,
            max_retries: 3,
            retry_base_delay_ms: 200,
            request_timeout_secs: 30,
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

    /// Set the maximum number of retry attempts for transient failures.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the base delay in milliseconds between retries (doubled each attempt).
    pub fn with_retry_base_delay_ms(mut self, delay_ms: u64) -> Self {
        self.retry_base_delay_ms = delay_ms;
        self
    }

    /// Set the request timeout in seconds.
    pub fn with_request_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.request_timeout_secs = timeout_secs;
        self
    }
}

// Generate VaultV1Client struct and core methods
define_vault_client_core! {
    /// Client for querying V1 (MetaMorpho) vaults.
    VaultV1Client
}

impl VaultV1Client {
    /// Get V1 vaults with optional filters, auto-paginating through all results.
    pub async fn get_vaults(&self, filters: Option<VaultFiltersV1>) -> Result<Vec<VaultV1>> {
        let gql_filters = filters.as_ref().map(|f| f.to_gql());
        let page_size = self.config.page_size;
        let mut all_vaults = Vec::new();
        let mut skip: i64 = 0;

        for _ in 0..MAX_PAGINATION_PAGES {
            let variables = get_vaults_v1::Variables {
                first: Some(page_size),
                skip: Some(skip),
                where_: gql_filters.clone(),
                order_by: Some(VaultOrderByV1::default().to_gql()),
                order_direction: Some(OrderDirection::default().to_gql_v1()),
            };

            let data = self.execute::<GetVaultsV1>(variables).await?;

            let items = match data.vaults.items {
                Some(items) => items,
                None => break,
            };

            let page_count = items.len() as i64;
            let vaults: Vec<VaultV1> = items.into_iter().filter_map(convert_v1_vault).collect();
            all_vaults.extend(vaults);

            let count_total = data
                .vaults
                .page_info
                .as_ref()
                .map(|p| p.count_total)
                .unwrap_or(0);

            skip += page_count;
            if page_count < page_size || skip >= count_total {
                break;
            }
        }

        Ok(all_vaults)
    }

    /// Get a single V1 vault by address and chain.
    pub async fn get_vault(&self, address: &str, chain: NamedChain) -> Result<VaultV1> {
        let variables = get_vault_v1_by_address::Variables {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
        };

        let data = self.execute::<GetVaultV1ByAddress>(variables).await?;

        convert_v1_vault_single(data.vault_by_address).ok_or_else(|| ApiError::VaultNotFound {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
        })
    }

    /// Get V1 vaults on a specific chain.
    pub async fn get_vaults_by_chain(&self, chain: NamedChain) -> Result<Vec<VaultV1>> {
        let filters = VaultFiltersV1::new().chain(chain);
        self.get_vaults(Some(filters)).await
    }

    /// Get V1 vaults by curator address.
    pub async fn get_vaults_by_curator(
        &self,
        curator: &str,
        chain: Option<NamedChain>,
    ) -> Result<Vec<VaultV1>> {
        let mut filters = VaultFiltersV1::new().curators([curator]);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }

    /// Get whitelisted (listed) V1 vaults.
    pub async fn get_whitelisted_vaults(&self, chain: Option<NamedChain>) -> Result<Vec<VaultV1>> {
        let mut filters = VaultFiltersV1::new().listed(true);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }

    /// Get V1 vaults with query options (filters, ordering, and limit).
    ///
    /// This method provides full control over the query parameters including
    /// ordering by various fields like APY, total assets, etc.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV1Client, VaultQueryOptionsV1, VaultFiltersV1, VaultOrderByV1, OrderDirection, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV1Client::new();
    ///
    ///     // Get top 10 USDC vaults by APY on Ethereum
    ///     let options = VaultQueryOptionsV1::new()
    ///         .filters(VaultFiltersV1::new()
    ///             .chain(NamedChain::Mainnet)
    ///             .asset_symbols(["USDC"]))
    ///         .order_by(VaultOrderByV1::NetApy)
    ///         .order_direction(OrderDirection::Desc)
    ///         .limit(10);
    ///
    ///     let vaults = client.get_vaults_with_options(options).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_vaults_with_options(
        &self,
        options: VaultQueryOptionsV1,
    ) -> Result<Vec<VaultV1>> {
        let gql_filters = options.filters.as_ref().map(|f| f.to_gql());
        let order_by = Some(options.order_by.unwrap_or_default().to_gql());
        let order_direction = Some(options.order_direction.unwrap_or_default().to_gql_v1());
        let page_size = self.config.page_size;
        let limit = options.limit;
        let mut all_vaults = Vec::new();
        let mut skip: i64 = 0;

        for _ in 0..MAX_PAGINATION_PAGES {
            let variables = get_vaults_v1::Variables {
                first: Some(page_size),
                skip: Some(skip),
                where_: gql_filters.clone(),
                order_by: order_by.clone(),
                order_direction: order_direction.clone(),
            };

            let data = self.execute::<GetVaultsV1>(variables).await?;

            let items = match data.vaults.items {
                Some(items) => items,
                None => break,
            };

            let page_count = items.len() as i64;
            let vaults: Vec<VaultV1> = items.into_iter().filter_map(convert_v1_vault).collect();
            all_vaults.extend(vaults);

            // Respect user-specified limit
            if let Some(lim) = limit {
                if all_vaults.len() >= lim as usize {
                    all_vaults.truncate(lim as usize);
                    return Ok(all_vaults);
                }
            }

            let count_total = data
                .vaults
                .page_info
                .as_ref()
                .map(|p| p.count_total)
                .unwrap_or(0);

            skip += page_count;
            if page_count < page_size || skip >= count_total {
                break;
            }
        }

        Ok(all_vaults)
    }

    /// Get top N V1 vaults ordered by APY (highest first).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV1Client, VaultFiltersV1, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV1Client::new();
    ///
    ///     // Get top 10 vaults by APY on Ethereum
    ///     let filters = VaultFiltersV1::new().chain(NamedChain::Mainnet);
    ///     let vaults = client.get_top_vaults_by_apy(10, Some(filters)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_top_vaults_by_apy(
        &self,
        limit: i64,
        filters: Option<VaultFiltersV1>,
    ) -> Result<Vec<VaultV1>> {
        let options = VaultQueryOptionsV1 {
            filters,
            order_by: Some(VaultOrderByV1::NetApy),
            order_direction: Some(OrderDirection::Desc),
            limit: Some(limit),
        };
        self.get_vaults_with_options(options).await
    }

    /// Get V1 vaults for a specific deposit asset.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV1Client, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV1Client::new();
    ///
    ///     // Get all USDC vaults
    ///     let vaults = client.get_vaults_by_asset("USDC", None).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_vaults_by_asset(
        &self,
        asset_symbol: &str,
        chain: Option<NamedChain>,
    ) -> Result<Vec<VaultV1>> {
        let mut filters = VaultFiltersV1::new().asset_symbols([asset_symbol]);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }
}

// Generate VaultV2Client struct and core methods
define_vault_client_core! {
    /// Client for querying V2 vaults.
    VaultV2Client
}

impl VaultV2Client {
    /// Get V2 vaults with optional filters, auto-paginating through all results.
    pub async fn get_vaults(&self, filters: Option<VaultFiltersV2>) -> Result<Vec<VaultV2>> {
        let gql_filters = filters.as_ref().map(|f| f.to_gql());
        let page_size = self.config.page_size;
        let mut all_vaults = Vec::new();
        let mut skip: i64 = 0;

        for _ in 0..MAX_PAGINATION_PAGES {
            let variables = get_vaults_v2::Variables {
                first: Some(page_size),
                skip: Some(skip),
                where_: gql_filters.clone(),
                order_by: Some(VaultOrderByV2::default().to_gql()),
                order_direction: Some(OrderDirection::default().to_gql_v2()),
            };

            let data = self.execute::<GetVaultsV2>(variables).await?;

            let items = match data.vault_v2s.items {
                Some(items) => items,
                None => break,
            };

            let page_count = items.len() as i64;
            let vaults: Vec<VaultV2> = items.into_iter().filter_map(convert_v2_vault).collect();
            all_vaults.extend(vaults);

            let count_total = data
                .vault_v2s
                .page_info
                .as_ref()
                .map(|p| p.count_total)
                .unwrap_or(0);

            skip += page_count;
            if page_count < page_size || skip >= count_total {
                break;
            }
        }

        Ok(all_vaults)
    }

    /// Get a single V2 vault by address and chain.
    pub async fn get_vault(&self, address: &str, chain: NamedChain) -> Result<VaultV2> {
        let variables = get_vault_v2_by_address::Variables {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
        };

        let data = self.execute::<GetVaultV2ByAddress>(variables).await?;

        convert_v2_vault_single(data.vault_v2_by_address).ok_or_else(|| ApiError::VaultNotFound {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
        })
    }

    /// Get V2 vaults on a specific chain.
    pub async fn get_vaults_by_chain(&self, chain: NamedChain) -> Result<Vec<VaultV2>> {
        let filters = VaultFiltersV2::new().chain(chain);
        self.get_vaults(Some(filters)).await
    }

    /// Get whitelisted (listed) V2 vaults.
    pub async fn get_whitelisted_vaults(&self, chain: Option<NamedChain>) -> Result<Vec<VaultV2>> {
        let mut filters = VaultFiltersV2::new().listed(true);
        if let Some(c) = chain {
            filters = filters.chain(c);
        }
        self.get_vaults(Some(filters)).await
    }

    /// Get V2 vaults with query options (filters, ordering, and limit).
    ///
    /// This method provides full control over the query parameters including
    /// ordering by various fields like APY, total assets, liquidity, etc.
    ///
    /// Note: Asset filtering (by symbol or address) is done client-side since
    /// the Morpho V2 API doesn't support server-side asset filtering.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV2Client, VaultQueryOptionsV2, VaultFiltersV2, VaultOrderByV2, OrderDirection, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV2Client::new();
    ///
    ///     // Get top 10 USDC vaults by APY on Ethereum
    ///     let options = VaultQueryOptionsV2::new()
    ///         .filters(VaultFiltersV2::new()
    ///             .chain(NamedChain::Mainnet))
    ///         .order_by(VaultOrderByV2::NetApy)
    ///         .order_direction(OrderDirection::Desc)
    ///         .asset_symbols(["USDC"])  // Client-side filtering
    ///         .limit(10);
    ///
    ///     let vaults = client.get_vaults_with_options(options).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_vaults_with_options(
        &self,
        options: VaultQueryOptionsV2,
    ) -> Result<Vec<VaultV2>> {
        let gql_filters = options.filters.as_ref().map(|f| f.to_gql());
        let order_by = Some(options.order_by.unwrap_or_default().to_gql());
        let order_direction = Some(options.order_direction.unwrap_or_default().to_gql_v2());
        let page_size = self.config.page_size;
        let limit = options.limit;
        let has_client_filter = options.has_client_filter();
        let mut all_vaults = Vec::new();
        let mut skip: i64 = 0;

        for _ in 0..MAX_PAGINATION_PAGES {
            let variables = get_vaults_v2::Variables {
                first: Some(page_size),
                skip: Some(skip),
                where_: gql_filters.clone(),
                order_by: order_by.clone(),
                order_direction: order_direction.clone(),
            };

            let data = self.execute::<GetVaultsV2>(variables).await?;

            let items = match data.vault_v2s.items {
                Some(items) => items,
                None => break,
            };

            let page_count = items.len() as i64;
            let mut vaults: Vec<VaultV2> =
                items.into_iter().filter_map(convert_v2_vault).collect();

            // Apply client-side filtering to this page
            if let Some(ref symbols) = options.asset_symbols {
                vaults
                    .retain(|v| symbols.iter().any(|s| s.eq_ignore_ascii_case(&v.asset.symbol)));
            }
            if let Some(ref addresses) = options.asset_addresses {
                vaults.retain(|v| {
                    addresses
                        .iter()
                        .any(|a| v.asset.address.to_string().eq_ignore_ascii_case(a))
                });
            }
            if let Some(ref curators) = options.curator_addresses {
                vaults.retain(|v| {
                    v.curator
                        .map(|c| {
                            curators
                                .iter()
                                .any(|a| c.to_string().eq_ignore_ascii_case(a))
                        })
                        .unwrap_or(false)
                });
            }

            all_vaults.extend(vaults);

            // Respect user-specified limit
            if let Some(lim) = limit {
                if all_vaults.len() >= lim as usize {
                    all_vaults.truncate(lim as usize);
                    return Ok(all_vaults);
                }
            }

            let count_total = data
                .vault_v2s
                .page_info
                .as_ref()
                .map(|p| p.count_total)
                .unwrap_or(0);

            skip += page_count;
            if page_count < page_size || skip >= count_total {
                break;
            }

            // When client-side filtering is active and we have a limit,
            // keep paginating to find enough matching results
            if has_client_filter && limit.is_some() {
                continue;
            }
        }

        Ok(all_vaults)
    }

    /// Get top N V2 vaults ordered by APY (highest first).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV2Client, VaultFiltersV2, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV2Client::new();
    ///
    ///     // Get top 10 vaults by APY on Ethereum
    ///     let filters = VaultFiltersV2::new().chain(NamedChain::Mainnet);
    ///     let vaults = client.get_top_vaults_by_apy(10, Some(filters)).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_top_vaults_by_apy(
        &self,
        limit: i64,
        filters: Option<VaultFiltersV2>,
    ) -> Result<Vec<VaultV2>> {
        let options = VaultQueryOptionsV2 {
            filters,
            order_by: Some(VaultOrderByV2::NetApy),
            order_direction: Some(OrderDirection::Desc),
            limit: Some(limit),
            asset_addresses: None,
            asset_symbols: None,
            curator_addresses: None,
        };
        self.get_vaults_with_options(options).await
    }

    /// Get V2 vaults by curator address.
    ///
    /// Note: This filtering is done client-side since the Morpho V2 API
    /// doesn't support server-side curator filtering.
    pub async fn get_vaults_by_curator(
        &self,
        curator: &str,
        chain: Option<NamedChain>,
    ) -> Result<Vec<VaultV2>> {
        let filters = chain.map(|c| VaultFiltersV2::new().chain(c));
        let options = VaultQueryOptionsV2::new()
            .curator_addresses([curator]);
        let options = if let Some(f) = filters {
            options.filters(f)
        } else {
            options
        };
        self.get_vaults_with_options(options).await
    }

    /// Get V2 vaults for a specific deposit asset.
    ///
    /// Note: This filtering is done client-side since the Morpho V2 API
    /// doesn't support server-side asset filtering.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use morpho_rs_api::{VaultV2Client, NamedChain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), morpho_rs_api::ApiError> {
    ///     let client = VaultV2Client::new();
    ///
    ///     // Get all USDC vaults
    ///     let vaults = client.get_vaults_by_asset("USDC", None).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_vaults_by_asset(
        &self,
        asset_symbol: &str,
        chain: Option<NamedChain>,
    ) -> Result<Vec<VaultV2>> {
        let filters = chain.map(|c| VaultFiltersV2::new().chain(c));
        let options = VaultQueryOptionsV2 {
            filters,
            order_by: None,
            order_direction: None,
            limit: None,
            asset_addresses: None,
            asset_symbols: Some(vec![asset_symbol.to_string()]),
            curator_addresses: None,
        };
        self.get_vaults_with_options(options).await
    }
}

/// Combined client for querying both V1 and V2 vaults via the GraphQL API.
#[derive(Debug, Clone)]
pub struct MorphoApiClient {
    /// Shared HTTP client for direct queries (user positions, account overview).
    http_client: Client,
    /// Shared configuration.
    config: ClientConfig,
    /// V1 vault client.
    pub v1: VaultV1Client,
    /// V2 vault client.
    pub v2: VaultV2Client,
}

impl Default for MorphoApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MorphoApiClient {
    /// Create a new combined vault client with default configuration.
    pub fn new() -> Self {
        let config = ClientConfig::default();
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            http_client,
            config,
            v1: VaultV1Client::new(),
            v2: VaultV2Client::new(),
        }
    }

    /// Create a new combined vault client with custom configuration.
    pub fn with_config(config: ClientConfig) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.request_timeout_secs))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            http_client,
            config: config.clone(),
            v1: VaultV1Client::with_config(config.clone()),
            v2: VaultV2Client::with_config(config),
        }
    }

    /// Get vaults (V1 and V2) on a specific chain as unified Vault trait objects.
    pub async fn get_vaults_by_chain(&self, chain: NamedChain) -> Result<Vec<Box<dyn VaultTrait>>> {
        let (v1_vaults, v2_vaults) = tokio::try_join!(
            self.v1.get_vaults_by_chain(chain),
            self.v2.get_vaults_by_chain(chain),
        )?;

        let mut vaults: Vec<Box<dyn VaultTrait>> =
            Vec::with_capacity(v1_vaults.len() + v2_vaults.len());
        vaults.extend(
            v1_vaults
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn VaultTrait>),
        );
        vaults.extend(
            v2_vaults
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn VaultTrait>),
        );

        Ok(vaults)
    }

    /// Get whitelisted vaults (V1 and V2) as unified Vault trait objects.
    pub async fn get_whitelisted_vaults(
        &self,
        chain: Option<NamedChain>,
    ) -> Result<Vec<Box<dyn VaultTrait>>> {
        let (v1_vaults, v2_vaults) = tokio::try_join!(
            self.v1.get_whitelisted_vaults(chain),
            self.v2.get_whitelisted_vaults(chain),
        )?;

        let mut vaults: Vec<Box<dyn VaultTrait>> =
            Vec::with_capacity(v1_vaults.len() + v2_vaults.len());
        vaults.extend(
            v1_vaults
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn VaultTrait>),
        );
        vaults.extend(
            v2_vaults
                .into_iter()
                .map(|v| Box::new(v) as Box<dyn VaultTrait>),
        );

        Ok(vaults)
    }

    /// Execute a GraphQL query with retry/backoff for transient failures.
    async fn execute<Q: GraphQLQuery>(&self, variables: Q::Variables) -> Result<Q::ResponseData>
    where
        Q::Variables: serde::Serialize,
    {
        let request_body = Q::build_query(variables);

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = self.config.retry_base_delay_ms * 2u64.pow(attempt - 1);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }

            let send_result = self
                .http_client
                .post(self.config.api_url.as_str())
                .json(&request_body)
                .send()
                .await;

            let response = match send_result {
                Ok(resp) => resp,
                Err(e) => {
                    let err = ApiError::Request(e);
                    if err.is_retryable() && attempt < self.config.max_retries {
                        continue;
                    }
                    return Err(err);
                }
            };

            let json_result: std::result::Result<Response<Q::ResponseData>, _> =
                response.json().await;

            let response_body = match json_result {
                Ok(body) => body,
                Err(e) => {
                    let err = ApiError::Request(e);
                    if err.is_retryable() && attempt < self.config.max_retries {
                        continue;
                    }
                    return Err(err);
                }
            };

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

            return response_body
                .data
                .ok_or_else(|| ApiError::Parse("No data in response".to_string()));
        }

        unreachable!("retry loop always returns")
    }

    /// Get all vault positions (V1 and V2) for a user.
    ///
    /// If `chain` is `Some`, queries only that chain.
    /// If `chain` is `None`, queries all supported chains and aggregates results.
    pub async fn get_user_vault_positions(
        &self,
        address: &str,
        chain: Option<NamedChain>,
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
        chain: NamedChain,
    ) -> Result<UserVaultPositions> {
        let variables = get_user_vault_positions::Variables {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
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

    /// Maximum number of concurrent chain queries.
    const MAX_CONCURRENT_CHAIN_QUERIES: usize = 5;

    /// Get vault positions for a user across all chains.
    async fn get_user_vault_positions_all_chains(
        &self,
        address: &str,
    ) -> Result<UserVaultPositions> {
        use futures::stream::{self, StreamExt};

        // Filter chains to those with IDs that fit in GraphQL Int (32-bit signed)
        let valid_chains: Vec<_> = SUPPORTED_CHAINS
            .iter()
            .filter(|chain| u64::from(**chain) <= i32::MAX as u64)
            .copied()
            .collect();

        let results: Vec<_> = stream::iter(valid_chains.iter().map(|chain| {
            self.get_user_vault_positions_single_chain(address, *chain)
        }))
        .buffer_unordered(Self::MAX_CONCURRENT_CHAIN_QUERIES)
        .collect()
        .await;

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
        chain: NamedChain,
    ) -> Result<UserAccountOverview> {
        let variables = get_user_account_overview::Variables {
            address: address.to_string(),
            chain_id: u64::from(chain) as i64,
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

/// Configuration for the unified MorphoClient.
#[derive(Debug, Clone)]
pub struct MorphoClientConfig {
    /// API configuration.
    pub api_config: Option<ClientConfig>,
    /// RPC URL for on-chain interactions.
    pub rpc_url: Option<String>,
    /// Private key for signing transactions.
    pub private_key: Option<String>,
    /// Whether to automatically approve tokens before deposit if allowance is insufficient.
    /// When true, approves the exact minimal amount needed for the deposit.
    /// Defaults to true.
    pub auto_approve: bool,
}

impl Default for MorphoClientConfig {
    fn default() -> Self {
        Self {
            api_config: None,
            rpc_url: None,
            private_key: None,
            auto_approve: true,
        }
    }
}

impl MorphoClientConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the API configuration.
    pub fn with_api_config(mut self, config: ClientConfig) -> Self {
        self.api_config = Some(config);
        self
    }

    /// Set the RPC URL.
    pub fn with_rpc_url(mut self, rpc_url: impl Into<String>) -> Self {
        self.rpc_url = Some(rpc_url.into());
        self
    }

    /// Set the private key.
    pub fn with_private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    /// Set whether to automatically approve tokens before deposit.
    /// When true, approves the exact minimal amount needed for the deposit.
    /// Defaults to true.
    pub fn with_auto_approve(mut self, auto_approve: bool) -> Self {
        self.auto_approve = auto_approve;
        self
    }
}

// Generate VaultV1Operations using macro
define_vault_operations! {
    /// Wrapper for V1 vault operations that automatically uses the signer's address.
    VaultV1Operations,
    VaultV1TransactionClient
}

// Generate VaultV2Operations using macro
define_vault_operations! {
    /// Wrapper for V2 vault operations that automatically uses the signer's address.
    VaultV2Operations,
    VaultV2TransactionClient
}

/// Unified Morpho client combining API queries and on-chain transactions.
///
/// This client provides a namespace-style API for interacting with Morpho vaults:
/// - `client.api()` - Access to GraphQL API queries
/// - `client.vault_v1()` - V1 vault transaction operations
/// - `client.vault_v2()` - V2 vault transaction operations
///
/// # Example
///
/// ```no_run
/// use morpho_rs_api::{MorphoClient, MorphoClientConfig, NamedChain};
/// use alloy::primitives::{Address, U256};
///
/// #[tokio::main]
/// async fn main() -> Result<(), morpho_rs_api::ApiError> {
///     // API-only client
///     let client = MorphoClient::new();
///     let vaults = client.get_vaults_by_chain(NamedChain::Mainnet).await?;
///
///     // Full client with transaction support
///     let config = MorphoClientConfig::new()
///         .with_rpc_url("https://eth.llamarpc.com")
///         .with_private_key("0x...");
///     let client = MorphoClient::with_config(config)?;
///
///     // V1 vault operations
///     let vault: Address = "0x...".parse().unwrap();
///     let balance = client.vault_v1()?.balance(vault).await?;
///
///     Ok(())
/// }
/// ```
pub struct MorphoClient {
    api: MorphoApiClient,
    vault_v1_tx: Option<VaultV1TransactionClient>,
    vault_v2_tx: Option<VaultV2TransactionClient>,
    auto_approve: bool,
}

impl Default for MorphoClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MorphoClient {
    /// Create a new MorphoClient with default API configuration (no transaction support).
    pub fn new() -> Self {
        Self {
            api: MorphoApiClient::new(),
            vault_v1_tx: None,
            vault_v2_tx: None,
            auto_approve: true,
        }
    }

    /// Create a MorphoClient with custom configuration.
    ///
    /// If both `rpc_url` and `private_key` are provided, transaction support is enabled.
    pub fn with_config(config: MorphoClientConfig) -> Result<Self> {
        let api = match config.api_config {
            Some(api_config) => MorphoApiClient::with_config(api_config),
            None => MorphoApiClient::new(),
        };

        let (vault_v1_tx, vault_v2_tx) = match (&config.rpc_url, &config.private_key) {
            (Some(rpc_url), Some(private_key)) => {
                let v1 = VaultV1TransactionClient::new(rpc_url, private_key)?;
                let v2 = VaultV2TransactionClient::new(rpc_url, private_key)?;
                (Some(v1), Some(v2))
            }
            _ => (None, None),
        };

        Ok(Self {
            api,
            vault_v1_tx,
            vault_v2_tx,
            auto_approve: config.auto_approve,
        })
    }

    /// Get V1 vault operations.
    ///
    /// Returns an error if transaction support is not configured.
    pub fn vault_v1(&self) -> Result<VaultV1Operations<'_>> {
        match &self.vault_v1_tx {
            Some(client) => Ok(VaultV1Operations::new(client, self.auto_approve)),
            None => Err(ApiError::TransactionNotConfigured),
        }
    }

    /// Get V2 vault operations.
    ///
    /// Returns an error if transaction support is not configured.
    pub fn vault_v2(&self) -> Result<VaultV2Operations<'_>> {
        match &self.vault_v2_tx {
            Some(client) => Ok(VaultV2Operations::new(client, self.auto_approve)),
            None => Err(ApiError::TransactionNotConfigured),
        }
    }

    /// Check if auto_approve is enabled.
    pub fn auto_approve(&self) -> bool {
        self.auto_approve
    }

    /// Get the API client for GraphQL queries.
    pub fn api(&self) -> &MorphoApiClient {
        &self.api
    }

    /// Get vaults (V1 and V2) on a specific chain as unified Vault trait objects.
    pub async fn get_vaults_by_chain(&self, chain: NamedChain) -> Result<Vec<Box<dyn VaultTrait>>> {
        self.api.get_vaults_by_chain(chain).await
    }

    /// Get whitelisted vaults (V1 and V2) as unified Vault trait objects.
    pub async fn get_whitelisted_vaults(
        &self,
        chain: Option<NamedChain>,
    ) -> Result<Vec<Box<dyn VaultTrait>>> {
        self.api.get_whitelisted_vaults(chain).await
    }

    /// Get all vault positions (V1 and V2) for a user.
    pub async fn get_user_vault_positions(
        &self,
        address: &str,
        chain: Option<NamedChain>,
    ) -> Result<UserVaultPositions> {
        self.api.get_user_vault_positions(address, chain).await
    }

    /// Get complete account overview for a user on a specific chain.
    pub async fn get_user_account_overview(
        &self,
        address: &str,
        chain: NamedChain,
    ) -> Result<UserAccountOverview> {
        self.api.get_user_account_overview(address, chain).await
    }

    /// Check if transaction support is configured.
    pub fn has_transaction_support(&self) -> bool {
        self.vault_v1_tx.is_some()
    }

    /// Get the signer's address if transaction support is configured.
    pub fn signer_address(&self) -> Option<Address> {
        self.vault_v1_tx.as_ref().map(|c| c.signer_address())
    }
}

// Conversion functions from GraphQL types to our types

// Helper imports for conversion
use crate::types::scalars::parse_bigint;
use alloy_primitives::B256;
use std::str::FromStr;

/// Convert f64 fee (0.1 = 10%) to WAD-scaled U256.
fn fee_to_wad(fee: f64) -> U256 {
    let fee_wad = (fee * 1e18).round() as u128;
    U256::from(fee_wad)
}

/// Macro to generate V1 vault conversion functions for both query types.
/// This eliminates code duplication while maintaining type safety.
macro_rules! impl_v1_vault_conversion {
    ($fn_name:ident, $market_state_fn:ident, $state_fn:ident, $mod:ident) => {
        fn $market_state_fn(
            market: &$mod::MarketFields,
        ) -> Option<MarketStateV1> {
            let market_id = B256::from_str(&market.unique_key).ok()?;
            let ms = market.state.as_ref()?;
            let lltv = parse_bigint(&market.lltv)?;
            let timestamp: u64 = ms.timestamp.0.parse().ok()?;
            let total_supply_assets = parse_bigint(&ms.supply_assets)?;
            let total_borrow_assets = parse_bigint(&ms.borrow_assets)?;
            let liquidity = total_supply_assets.saturating_sub(total_borrow_assets);

            Some(MarketStateV1 {
                id: market_id,
                total_supply_assets,
                total_borrow_assets,
                total_supply_shares: parse_bigint(&ms.supply_shares)?,
                total_borrow_shares: parse_bigint(&ms.borrow_shares)?,
                last_update: timestamp,
                fee: fee_to_wad(ms.fee),
                rate_at_target: ms.rate_at_target.as_ref().and_then(|r| parse_bigint(r)),
                price: ms.price.as_ref().and_then(|p| parse_bigint(p)),
                lltv,
                liquidity,
            })
        }

        fn $state_fn(s: &$mod::VaultStateFields) -> Option<VaultStateV1> {
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
                        let market_state = $market_state_fn(market);
                        VaultAllocation::from_gql(
                            market.unique_key.clone(),
                            Some(market.loan_asset.symbol.clone()),
                            Some(market.loan_asset.address.as_str()),
                            market.collateral_asset.as_ref().map(|ca| ca.symbol.clone()),
                            market.collateral_asset.as_ref().map(|ca| ca.address.as_str()),
                            &a.supply_assets,
                            a.supply_assets_usd,
                            &a.supply_cap,
                            a.enabled,
                            a.supply_queue_index.map(|i| i as i32),
                            a.withdraw_queue_index.map(|i| i as i32),
                            market_state,
                        )
                    })
                    .collect(),
            )
        }

        fn $fn_name(v: $mod::VaultFields) -> Option<VaultV1> {
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
                v.state.as_ref().and_then($state_fn),
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
    };
}

// Generate conversion functions for GetVaultsV1 query types
impl_v1_vault_conversion!(
    convert_v1_vault,
    convert_v1_market_state,
    convert_v1_state,
    get_vaults_v1
);

// Generate conversion functions for GetVaultV1ByAddress query types
impl_v1_vault_conversion!(
    convert_v1_vault_single,
    convert_v1_market_state_single,
    convert_v1_state_single,
    get_vault_v1_by_address
);

/// Parse yearly supply tokens from string to f64.
fn parse_yearly_supply(s: &str) -> Option<f64> {
    s.parse::<f64>().ok()
}

/// Macro to generate V2 vault and adapter conversion functions for both query types.
/// V2 uses query-specific type names (not fragment-spread types like V1), so we pass them explicitly.
macro_rules! impl_v2_vault_conversion {
    ($vault_fn:ident, $adapter_fn:ident, $vault_type:ty, $adapter_type:ty, $adapter_on_type:path) => {
        #[allow(unreachable_patterns)]
        fn $adapter_fn(a: $adapter_type) -> Option<VaultAdapter> {
            use $adapter_on_type::*;

            let data = match &a.on {
                MorphoMarketV1Adapter(adapter) => {
                    let positions: Vec<MorphoMarketPosition> = adapter.positions.items.as_ref()
                        .map(|items| {
                            items.iter().filter_map(|pos| {
                                let market_id = B256::from_str(&pos.market.unique_key).ok()?;
                                let market_state = pos.market.state.as_ref().and_then(|ms| {
                                    let lltv = parse_bigint(&pos.market.lltv)?;
                                    let timestamp: u64 = ms.timestamp.0.parse().ok()?;
                                    Some(MarketStateV2 {
                                        id: market_id,
                                        total_supply_assets: parse_bigint(&ms.supply_assets)?,
                                        total_borrow_assets: parse_bigint(&ms.borrow_assets)?,
                                        total_supply_shares: parse_bigint(&ms.supply_shares)?,
                                        total_borrow_shares: parse_bigint(&ms.borrow_shares)?,
                                        last_update: timestamp,
                                        fee: fee_to_wad(ms.fee),
                                        rate_at_target: ms.rate_at_target.as_ref().and_then(|r| parse_bigint(r)),
                                        price: ms.price.as_ref().and_then(|p| parse_bigint(p)),
                                        lltv,
                                    })
                                });
                                Some(MorphoMarketPosition {
                                    supply_assets: parse_bigint(&pos.supply_assets)?,
                                    supply_shares: parse_bigint(&pos.supply_shares)?,
                                    market_id,
                                    market_state,
                                })
                            }).collect()
                        })
                        .unwrap_or_default();
                    Some(VaultAdapterData::MorphoMarketV1 { positions })
                }
                MetaMorphoAdapter(adapter) => {
                    let allocations: Vec<MetaMorphoAllocation> = adapter.meta_morpho.state.as_ref()
                        .map(|state| {
                            state.allocation.iter().filter_map(|alloc| {
                                let market_id = B256::from_str(&alloc.market.unique_key).ok()?;
                                let market_state = alloc.market.state.as_ref().and_then(|ms| {
                                    let lltv = parse_bigint(&alloc.market.lltv)?;
                                    let timestamp: u64 = ms.timestamp.0.parse().ok()?;
                                    Some(MarketStateV2 {
                                        id: market_id,
                                        total_supply_assets: parse_bigint(&ms.supply_assets)?,
                                        total_borrow_assets: parse_bigint(&ms.borrow_assets)?,
                                        total_supply_shares: parse_bigint(&ms.supply_shares)?,
                                        total_borrow_shares: parse_bigint(&ms.borrow_shares)?,
                                        last_update: timestamp,
                                        fee: fee_to_wad(ms.fee),
                                        rate_at_target: ms.rate_at_target.as_ref().and_then(|r| parse_bigint(r)),
                                        price: ms.price.as_ref().and_then(|p| parse_bigint(p)),
                                        lltv,
                                    })
                                });
                                Some(MetaMorphoAllocation {
                                    supply_assets: parse_bigint(&alloc.supply_assets)?,
                                    supply_cap: parse_bigint(&alloc.supply_cap)?,
                                    enabled: alloc.enabled,
                                    supply_queue_index: alloc.supply_queue_index.map(|i| i as i32),
                                    withdraw_queue_index: alloc.withdraw_queue_index.map(|i| i as i32),
                                    market_id,
                                    market_state,
                                })
                            }).collect()
                        })
                        .unwrap_or_default();
                    Some(VaultAdapterData::MetaMorpho { allocations })
                }
                _ => None,
            };

            VaultAdapter::from_gql(
                a.id,
                &a.address,
                format!("{:?}", a.type_),
                &a.assets,
                a.assets_usd,
                data,
            )
        }

        fn $vault_fn(v: $vault_type) -> Option<VaultV2> {
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
                            .filter_map($adapter_fn)
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
    };
}

impl_v2_vault_conversion!(
    convert_v2_vault, convert_v2_adapter,
    get_vaults_v2::GetVaultsV2VaultV2sItems,
    get_vaults_v2::GetVaultsV2VaultV2sItemsAdaptersItems,
    get_vaults_v2::GetVaultsV2VaultV2sItemsAdaptersItemsOn
);

impl_v2_vault_conversion!(
    convert_v2_vault_single, convert_v2_adapter_single,
    get_vault_v2_by_address::GetVaultV2ByAddressVaultV2ByAddress,
    get_vault_v2_by_address::GetVaultV2ByAddressVaultV2ByAddressAdaptersItems,
    get_vault_v2_by_address::GetVaultV2ByAddressVaultV2ByAddressAdaptersItemsOn
);

// User position conversion functions

macro_rules! impl_user_v1_position_conversion {
    ($fn_name:ident, $position_type:ty) => {
        fn $fn_name(p: $position_type) -> Option<UserVaultV1Position> {
            let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol, p.vault.chain.id)?;

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
    };
}

macro_rules! impl_user_v2_position_conversion {
    ($fn_name:ident, $position_type:ty) => {
        fn $fn_name(p: $position_type) -> Option<UserVaultV2Position> {
            let vault = VaultInfo::from_gql(&p.vault.address, p.vault.name, p.vault.symbol, p.vault.chain.id)?;

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
    };
}

impl_user_v1_position_conversion!(
    convert_user_vault_v1_position,
    get_user_vault_positions::GetUserVaultPositionsUserByAddressVaultPositions
);
impl_user_v1_position_conversion!(
    convert_user_vault_v1_position_overview,
    get_user_account_overview::GetUserAccountOverviewUserByAddressVaultPositions
);
impl_user_v2_position_conversion!(
    convert_user_vault_v2_position,
    get_user_vault_positions::GetUserVaultPositionsUserByAddressVaultV2Positions
);
impl_user_v2_position_conversion!(
    convert_user_vault_v2_position_overview,
    get_user_account_overview::GetUserAccountOverviewUserByAddressVaultV2Positions
);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_to_wad_zero() {
        let result = fee_to_wad(0.0);
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn test_fee_to_wad_ten_percent() {
        // 10% fee = 0.1 -> 1e17 in WAD
        let result = fee_to_wad(0.1);
        let expected = U256::from(100_000_000_000_000_000u64); // 1e17
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fee_to_wad_max() {
        // 100% fee = 1.0 -> 1e18 in WAD
        let result = fee_to_wad(1.0);
        let expected = U256::from(1_000_000_000_000_000_000u64); // 1e18
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fee_to_wad_fifteen_percent() {
        // 15% fee = 0.15 -> 1.5e17 in WAD
        let result = fee_to_wad(0.15);
        let expected = U256::from(150_000_000_000_000_000u64); // 1.5e17
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fee_to_wad_small_fee() {
        // 0.1% fee = 0.001 -> 1e15 in WAD
        let result = fee_to_wad(0.001);
        let expected = U256::from(1_000_000_000_000_000u64); // 1e15
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fee_to_wad_rounding_0021() {
        // 0.21% fee = 0.0021 -> 2.1e15 in WAD
        // Without .round(), 0.0021 * 1e18 = 2099999999999999.8, truncated to 2099999999999999
        let result = fee_to_wad(0.0021);
        let expected = U256::from(2_100_000_000_000_000u64);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fee_to_wad_rounding_0045() {
        // 0.45% fee = 0.0045 -> 4.5e15 in WAD
        // Without .round(), 0.0045 * 1e18 = 4499999999999999.5, truncated to 4499999999999999
        let result = fee_to_wad(0.0045);
        let expected = U256::from(4_500_000_000_000_000u64);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.page_size, DEFAULT_PAGE_SIZE);
        assert_eq!(config.api_url.as_str(), DEFAULT_API_URL);
    }

    #[test]
    fn test_client_config_builder() {
        let url = Url::parse("https://custom.api.com/graphql").unwrap();
        let config = ClientConfig::new()
            .with_api_url(url.clone())
            .with_page_size(50);

        assert_eq!(config.api_url, url);
        assert_eq!(config.page_size, 50);
    }

    #[test]
    fn test_morpho_client_config_default() {
        let config = MorphoClientConfig::default();
        assert!(config.api_config.is_none());
        assert!(config.rpc_url.is_none());
        assert!(config.private_key.is_none());
        assert!(config.auto_approve);
    }

    #[test]
    fn test_morpho_client_config_builder() {
        let api_config = ClientConfig::new().with_page_size(50);
        let config = MorphoClientConfig::new()
            .with_api_config(api_config)
            .with_rpc_url("https://eth.llamarpc.com")
            .with_private_key("0xabcd")
            .with_auto_approve(false);

        assert!(config.api_config.is_some());
        assert_eq!(config.rpc_url, Some("https://eth.llamarpc.com".to_string()));
        assert_eq!(config.private_key, Some("0xabcd".to_string()));
        assert!(!config.auto_approve);
    }

    #[test]
    fn test_parse_yearly_supply() {
        assert_eq!(parse_yearly_supply("1000.5"), Some(1000.5));
        assert_eq!(parse_yearly_supply("0"), Some(0.0));
        assert_eq!(parse_yearly_supply("invalid"), None);
    }
}


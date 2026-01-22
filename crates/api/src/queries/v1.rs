//! V1 vault GraphQL queries.

use graphql_client::GraphQLQuery;

/// Custom scalar type mappings for GraphQL.
pub type Address = String;
pub type BigInt = String;
pub type MarketId = String;
pub type HexString = String;

/// Query for fetching multiple V1 vaults with filters.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/vaults_v1.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetVaultsV1;

/// Query for fetching a single V1 vault by address.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/vaults_v1.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetVaultV1ByAddress;

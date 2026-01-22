//! User GraphQL queries.

use graphql_client::GraphQLQuery;

/// Custom scalar type mappings for GraphQL.
pub type Address = String;
pub type BigInt = String;
pub type MarketId = String;
pub type HexString = String;

/// Query for fetching a user's vault positions (V1 and V2).
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/user.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetUserVaultPositions;

/// Query for fetching a user's account overview including all positions.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/user.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetUserAccountOverview;

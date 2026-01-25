//! Vault simulation GraphQL queries.

use graphql_client::GraphQLQuery;

use crate::types::scalars::FlexBigInt;

/// Custom scalar type mappings for GraphQL.
pub type Address = String;
pub type BigInt = FlexBigInt;
pub type MarketId = String;
pub type HexString = String;

/// Query for fetching a single vault with simulation data.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/vault_simulation.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetVaultForSimulation;

/// Query for fetching multiple vaults with simulation data.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema/morpho.graphql",
    query_path = "queries/vault_simulation.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct GetVaultsForSimulation;

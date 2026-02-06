# Morpho-rs TODO

## Completed - Test Coverage

### CLI Crate (~70-80% coverage on core logic)
- [x] Add integration tests for vault commands (wiremock-based API mocking)
- [x] Test deposit/withdraw command logic (pure functions: parse_amount, format_gas)
- [x] Test output formatting (table.rs: truncate_address, truncate_name, format_apy, format_usd)
- [x] Test output formatting (positions.rs: negative value handling in format_usd)
- [x] Test CLI argument parsing (ChainArg::FromStr with all chain aliases, full CLI arg parsing)
- [x] Refactor MORPHO_API_URL to use clap's env feature (global --api-url flag)

### API Crate (~73% line coverage)
- [x] Test GraphQL response parsing (wiremock-based integration tests)
- [x] Test error handling paths (ApiError variants, GraphQL errors, null data)
- [x] Test filter composition (v1_filters, v2_filters, query_options)
- [x] Test V1 client methods (get_vaults, get_vault, filters, options)
- [x] Test V2 client methods (get_vaults, client-side asset filtering)
- [x] Test user position methods (single chain, vault info)
- [x] Unit tests for fee_to_wad, config builders, parse functions
- [x] Test MorphoClient configuration (transaction support detection, signer address, auto_approve)
- [x] Test VaultV1/V2Operations fork tests with anvil (get_asset, get_balance, deposit, withdraw)

### Contracts Crate (~25% coverage -> improved)
- [x] Unit tests for calldata encoding (ERC-4626: deposit, withdraw, mint, redeem; ERC-20: approve)
- [x] Unit tests for PreparedCall accessors (to, value, prepare)
- [x] Unit tests for signer address derivation
- [x] Fork tests for mint/redeem (verify share-based operations)
- [x] Fork tests for approve_if_needed (skip when sufficient, approve when insufficient)

## Completed - Sim Crate Tests (117 unit tests + 19 doc-tests)

- [x] Unit tests for Market operations (supply, borrow, withdraw, repay, accrue_interest)
- [x] Unit tests for APY calculations (get_supply_apy, get_borrow_apy, supply_apy_impact, borrow_apy_impact)
- [x] Unit tests for IRM (get_borrow_rate, w_exp, get_supply_for_borrow_rate, get_utilization_at_borrow_rate)
- [x] Unit tests for math utilities (wad_mul, wad_div, mul_div, RoundingDirection)
- [x] Unit tests for Position (health_factor, liquidation_price, capacities)
- [x] Unit tests for Vault simulation (deposit, withdraw, get_net_apy, reallocation)
- [x] Unit tests for optimization (rank_vaults_by_apy, find_best_vault_for_deposit, find_optimal_market_allocation)
- [x] Unit tests for public allocator (flow limits, ReallocationStep)

## High Priority - Code Quality

- [x] V2 adapter conversion duplication (`client.rs`) — created `impl_v2_vault_conversion!` macro like V1's
- [x] User position conversion duplication (`client.rs`) — created `impl_user_v1_position_conversion!` and `impl_user_v2_position_conversion!` macros
- [x] `fee_to_wad` floating-point precision (`client.rs`) — added `.round()` before `as u128` cast

## Medium Priority - API Improvements

- [ ] No pagination for queries — only first page fetched, large results silently truncated
- [x] `MorphoApiClient.execute()` HTTP client coupling (`client.rs:774`) — gave MorphoApiClient its own http_client and config fields
- [ ] V2 curator filtering is client-side only (not in VaultV2Client API)
- [x] Unbounded concurrency in all-chains query (`client.rs:855`) — limited to 5 concurrent requests via `buffer_unordered`
- [ ] Retry/backoff for API requests — no resilience to transient failures

## Medium Priority - Code Quality

- [x] Fix unwrap in cli/commands/vault_v1.rs:37 — replaced with if-let
- [x] Audit remaining production unwrap calls — replaced all 13 `.unwrap()` in sim/vault.rs with proper `?` error propagation
- [x] Consolidate V2 adapter conversion functions
- [x] Reduce user position conversion duplication
- [ ] Error type consolidation — ApiError, ContractError, SimError are fragmented
- [ ] Document `sim` feature flag and simulation conversion methods
- [ ] Document all from_gql() helper methods
- [ ] Add filter usage examples to docs

## Low Priority - Infrastructure

- [ ] CI/CD pipeline (GitHub Actions for test, clippy, fmt)
- [ ] crates.io publishing configuration (categories, keywords, documentation fields)

## Low Priority - Enhancements

- [ ] Wire `sim` feature through CLI for APY simulation commands
- [ ] Add `--all-pages` flag for full pagination
- [ ] Consider trait-based conversion pattern
- [ ] Improve CLI error messages
- [ ] Add snapshot tests for output formatting

# Morpho-rs TODO

## High Priority - Test Coverage

### CLI Crate (~70-80% coverage on core logic)
- [ ] Add integration tests for vault commands (requires API mocking)
- [x] Test deposit/withdraw command logic (pure functions: parse_amount, format_gas)
- [x] Test output formatting (table.rs: truncate_address, truncate_name, format_apy, format_usd)
- [x] Test output formatting (positions.rs: negative value handling in format_usd)
- [x] Test CLI argument parsing (ChainArg::FromStr with all chain aliases, full CLI arg parsing)

### API Crate (~3% coverage)
- [ ] Test MorphoClient query methods
- [ ] Test GraphQL response parsing
- [ ] Test error handling paths
- [ ] Test filter composition

### Contracts Crate (~25% coverage)
- [ ] Unit tests for VaultV1TransactionClient methods
- [ ] Unit tests for VaultV2TransactionClient methods

## Medium Priority - Code Quality

### Unwrap Removal
- [ ] Fix unwrap in cli/commands/vault_v1.rs:17
- [ ] Audit remaining production unwrap calls

### Code Duplication
- [ ] Consolidate V2 adapter conversion functions
- [ ] Reduce user position conversion duplication

### Documentation
- [ ] Document all from_gql() helper methods
- [ ] Add filter usage examples to docs

## Low Priority - Enhancements

- [ ] Consider trait-based conversion pattern
- [ ] Improve CLI error messages
- [ ] Add snapshot tests for output formatting

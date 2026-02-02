# Default recipe
default: build

# Build all crates
build:
    cargo build

# Build all crates in release mode
build-release:
    cargo build --release

# Run all unit and integration tests
test:
    cargo test

# Run E2E fork tests (requires ETH_RPC_URL)
test-e2e:
    ANVIL_COMPUTE_UNITS_PER_SECOND=100 cargo test --test e2e_fork -- --ignored

# Run all tests including E2E
test-all: test test-e2e

# Format code
fmt:
    cargo fmt

# Run clippy lints
clippy:
    cargo clippy

# Quick check without full build
check:
    cargo check

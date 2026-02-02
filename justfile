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
# Configurable env vars (with defaults):
#   ANVIL_COMPUTE_UNITS_PER_SECOND=100 - Compute units per second for rate limiting
#   ANVIL_RETRIES=5 - Number of retries for failed RPC requests
#   ANVIL_FORK_RETRY_BACKOFF=1000 - Backoff in ms between retries
#   ANVIL_TIMEOUT=45000 - Timeout in ms for RPC requests
test-e2e:
    ANVIL_COMPUTE_UNITS_PER_SECOND=${ANVIL_COMPUTE_UNITS_PER_SECOND:-100} \
    ANVIL_RETRIES=${ANVIL_RETRIES:-5} \
    ANVIL_FORK_RETRY_BACKOFF=${ANVIL_FORK_RETRY_BACKOFF:-1000} \
    ANVIL_TIMEOUT=${ANVIL_TIMEOUT:-45000} \
    cargo test --test e2e_fork -- --ignored

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

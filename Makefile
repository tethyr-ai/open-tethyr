# Open-Tethyr Makefile
# Convenience targets for build, test, lint, and release operations

CARGO := cargo
CLIPPY_FLAGS := --workspace --all-features -- -D warnings
TEST_FLAGS := --workspace --all-features
FMT_FLAGS := --all

.PHONY: all build check test test-verbose test-proptest lint fmt clean \
        bench serve docker help ci

# Default target
all: fmt lint test

## Build

build: ## Build all crates (debug)
	$(CARGO) build $(TEST_FLAGS)

build-release: ## Build all crates (release, optimized)
	$(CARGO) build --release $(TEST_FLAGS)

check: ## Type-check without building
	$(CARGO) check $(TEST_FLAGS)

## Test Tiers

test: ## Run all 145 tests (all features)
	$(CARGO) test $(TEST_FLAGS)

test-verbose: ## Run all tests with output
	$(CARGO) test $(TEST_FLAGS) -- --nocapture

test-lib: ## Run only library tests
	$(CARGO) test -p open-tethyr --all-features

test-cli: ## Run only CLI tests (includes binary tests)
	$(CARGO) test -p open-tethyr-cli

test-http: ## Run HTTP mock tests only
	$(CARGO) test -p open-tethyr --all-features --test http_mock_tests

test-server: ## Run server route tests only
	$(CARGO) test -p open-tethyr --all-features --test server_route_tests

test-malformed: ## Run malformed input tests only
	$(CARGO) test -p open-tethyr --all-features --test malformed_input_tests

test-property: ## Run property tests with extended iterations (1000 cases)
	PROPTEST_CASES=1000 $(CARGO) test $(TEST_FLAGS)

test-stress: ## Run property tests with high iterations (10000 cases)
	PROPTEST_CASES=10000 $(CARGO) test $(TEST_FLAGS)

test-no-default: ## Run tests with no default features (client-less build)
	$(CARGO) test --no-default-features --workspace

## CI (mirrors GitHub Actions pipeline)

ci: fmt-check lint test test-no-default ## Run full CI pipeline locally
	@echo ""
	@echo "CI passed: format, lint, all tests, no-default-features tests"

## Lint & Format

lint: ## Run clippy with warnings as errors
	$(CARGO) clippy $(CLIPPY_FLAGS)

fmt: ## Format all code
	$(CARGO) fmt $(FMT_FLAGS)

fmt-check: ## Check formatting without changing files
	$(CARGO) fmt $(FMT_FLAGS) -- --check

## Benchmarks

bench: ## Run criterion benchmarks
	$(CARGO) bench -p open-tethyr --all-features

## Run

serve: ## Start cache server on default port
	$(CARGO) run -p open-tethyr-cli --release -- serve --port 8080

generate: ## Generate AX records (usage: make generate CONFIG=path OUTPUT=dir)
	$(CARGO) run -p open-tethyr-cli -- generate --config $(CONFIG) --output $(OUTPUT) --validate

validate: ## Validate an AX record file (usage: make validate FILE=path)
	$(CARGO) run -p open-tethyr-cli -- validate $(FILE)

## Docker

docker: ## Build Docker image
	docker build -t open-tethyr:latest -f docker/Dockerfile .

docker-run: ## Run cache server in Docker
	docker run --rm -p 8080:8080 open-tethyr:latest serve --port 8080

## Clean

clean: ## Remove build artifacts
	$(CARGO) clean

clean-all: clean ## Remove build artifacts and Cargo.lock
	rm -f Cargo.lock

## Help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

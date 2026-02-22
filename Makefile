.PHONY: help lint build test test-all fmt check clean install dev

# Default target
help:
	@echo "Available targets:"
	@echo "  make lint       - Run clippy linter with strict warnings"
	@echo "  make fmt        - Format code with rustfmt"
	@echo "  make check      - Run fmt check and clippy"
	@echo "  make build      - Build the project with all features"
	@echo "  make test       - Run tests with all features"
	@echo "  make test-all   - Run tests with all features and no default features"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make install    - Install the CLI binary"
	@echo "  make dev        - Run fmt, check, and test (development workflow)"

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Run clippy linter
lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run both format check and clippy
check: fmt-check lint
	@echo "All checks passed!"

# Build the project
build:
	@echo "Building project with all features..."
	cargo build --all-features --workspace

# Build release
build-release:
	@echo "Building release with all features..."
	cargo build --release --all-features --workspace

# Run tests with all features
test:
	@echo "Running tests with all features..."
	cargo test --all-features --workspace

# Run tests with no default features
test-no-default:
	@echo "Running tests with no default features..."
	cargo test --no-default-features --workspace

# Run all test configurations
test-all: test test-no-default
	@echo "All tests passed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

# Install CLI binary
install:
	@echo "Installing open-tethyr CLI..."
	cargo install --path crates/cli --force

# Development workflow: format, check, and test
dev: fmt check test
	@echo "Development checks complete!"

# Run specific test
test-one:
	@echo "Usage: make test-one TEST=test_name"
	@echo "Example: make test-one TEST=http_error_responses"

# Run benchmarks (if available)
bench:
	@echo "Running benchmarks..."
	cargo bench --all-features

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --all-features --no-deps --open

# Check for outdated dependencies
outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated

# Update dependencies
update:
	@echo "Updating dependencies..."
	cargo update

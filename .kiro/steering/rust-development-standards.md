# Rust Development Standards

## Pre-Development Checks

Before implementing any code changes, you MUST ensure the following checks pass:

### 1. Code Formatting
```bash
cargo fmt --all -- --check
```
If this fails, run `cargo fmt --all` to fix formatting issues.

### 2. Clippy Linting
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
All clippy warnings must be resolved. No warnings are allowed.

### 3. Test Suite
```bash
# Test with all features
cargo test --all-features --workspace

# Test with no default features  
cargo test --no-default-features --workspace
```
All tests must pass in both configurations.

## Development Workflow

1. **Always run pre-development checks first**
2. Make your code changes
3. **Run all checks again before committing**
4. Fix any issues that arise
5. Commit only when all checks pass

## Property-Based Testing

When implementing property-based tests:
- Use the `updatePBTStatus` tool to report test results
- **If tests fail, fix the underlying code issues** - the failing tests indicate bugs in the implementation
- Property-based test failures reveal real correctness issues that must be resolved
- Only move on once all property-based tests pass

## Code Quality Standards

- Remove unused code and imports
- Avoid redundant closures where possible
- Follow Rust naming conventions
- Write clear, concise documentation
- Ensure all public APIs are documented

## Commit Standards

- Run all checks before committing
- Use conventional commit messages
- Keep commits focused and atomic
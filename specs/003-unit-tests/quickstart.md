# Quickstart: Running Unit Tests

**Feature**: 003-unit-tests  
**Date**: 2025-12-07

## Prerequisites

- Rust 1.75+ installed
- Project dependencies installed (`cargo build`)
- Development dependencies will be added to `Cargo.toml`:
  - `httpmock = "0.7"`
  - `serial_test = "3.0"`
  - `tempfile = "3.8"`
- `cargo-llvm-cov` installed globally for coverage (optional)

## Quick Start

### Run All Tests (Fast Development Workflow)

```bash
# Run all unit tests with default output
cargo test

# Run tests with output from passing tests
cargo test -- --nocapture

# Run specific test by name
cargo test test_validate_args_valid_range

# Run all tests in a specific file
cargo test --test args_test
```

**Expected Results**:
- ✓ All tests pass on first run
- ✓ Execution completes in <10 seconds
- ✓ No network calls or API keys required

### Run Tests with Coverage (Slower, For Reporting)

```bash
# Install cargo-llvm-cov (one-time setup)
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --html
# Open target/llvm-cov/html/index.html in browser

# Generate LCOV format for CI
cargo llvm-cov --lcov --output-path coverage.lcov

# Show coverage summary in terminal
cargo llvm-cov --summary-only
```

**Expected Coverage Targets**:
- `src/args.rs`: >80% line coverage
- `src/config.rs`: >80% line coverage
- `src/provider_registry.rs`: >80% line coverage
- `src/forecast_provider.rs`: >80% line coverage
- Provider transformations: 100% of conversions tested

## Test Organization

### Directory Structure

```
tests/
├── args_test.rs              # CLI argument validation tests
├── config_test.rs            # Configuration file tests
├── provider_registry_test.rs # Provider discovery tests
├── timezone_test.rs          # Timezone conversion tests
├── stormglass_test.rs        # StormGlass provider tests (with mocks)
└── openweathermap_test.rs    # OpenWeatherMap provider tests (with mocks)
```

### Running Specific Test Categories

```bash
# Test CLI argument validation
cargo test --test args_test

# Test configuration management
cargo test --test config_test

# Test timezone conversion
cargo test --test timezone_test

# Test provider transformations
cargo test --test stormglass_test
cargo test --test openweathermap_test

# Test provider registry
cargo test --test provider_registry_test
```

## Common Testing Scenarios

### Test New Feature

```bash
# 1. Write test first (TDD approach)
# Edit tests/feature_test.rs

# 2. Run test (should fail)
cargo test test_new_feature

# 3. Implement feature
# Edit src/module.rs

# 4. Run test again (should pass)
cargo test test_new_feature

# 5. Run all tests to ensure no regressions
cargo test
```

### Verify Unit Conversions

```bash
# Test StormGlass m/s → knots conversion (×1.94384)
cargo test test_stormglass_converts_wind_speed

# Test OpenWeatherMap keeps m/s (no conversion)
cargo test test_openweathermap_wind_speed_remains
```

### Verify Timezone Conversion

```bash
# Test UTC → Jerusalem conversion (+2 hours)
cargo test test_convert_timezone_utc_to_jerusalem

# Test timestamp serialization format (YYYY-MM-DD HH:MM)
cargo test test_local_timestamp_serialization_format
```

### Verify Error Messages

```bash
# Test error message quality for common failures
cargo test test_validate_args_error_message_is_actionable
cargo test test_missing_coordinates_error_is_actionable
cargo test test_invalid_timezone_error_message_is_actionable
```

## Development Workflow (Constitution VI)

Follow this sequence for all code changes:

```bash
# 1. Check compilation (fix errors and warnings)
cargo check

# 2. Build for testing (DO NOT use --release during development)
cargo build

# 3. Run clippy (address all warnings)
cargo clippy

# 4. Run tests (should complete in <10 seconds)
cargo test

# 5. Optional: Generate coverage report
cargo llvm-cov --html
```

## Test Isolation

### Environment Variable Tests

Tests that modify environment variables use `#[serial]` attribute to prevent conflicts:

```bash
# These tests run sequentially, not in parallel
cargo test test_api_key -- --test-threads=1
```

**Why**: Environment variables are process-global, so tests that set/unset them must run sequentially to avoid race conditions.

### HTTP Mock Tests

Tests use `httpmock` to create isolated mock servers:

```bash
# Each test gets its own mock server on a unique port
cargo test test_provider_fetch
```

**Why**: No real network calls are made during tests. All HTTP responses are mocked.

### Temporary File Tests

Tests use `tempfile` crate for automatic cleanup:

```bash
# Temporary config files are automatically deleted after tests
cargo test test_config_loading
```

**Why**: Tests don't leave behind temporary files or modify real config files.

## Troubleshooting

### Tests Run Slowly (>10 seconds)

**Likely Cause**: Accidentally running release build
```bash
# Wrong (slow)
cargo test --release

# Correct (fast)
cargo test
```

### Tests Fail with Environment Variable Conflicts

**Likely Cause**: Tests running in parallel while modifying env vars
```bash
# Solution: Run env-dependent tests sequentially
cargo test test_api_key -- --test-threads=1
```

### Coverage Report Missing Files

**Likely Cause**: `cargo-llvm-cov` not installed or not in PATH
```bash
# Install globally
cargo install cargo-llvm-cov

# Verify installation
cargo llvm-cov --version
```

### Tests Pass Locally But Fail in CI

**Likely Causes**:
1. Flaky test with race condition → Add `#[serial]` attribute
2. Test depends on local file system → Use `tempfile` for test data
3. Test makes real network calls → Use `httpmock` to mock responses

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Run tests
  run: cargo test

- name: Generate coverage
  run: |
    cargo install cargo-llvm-cov
    cargo llvm-cov --lcov --output-path coverage.lcov

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    files: coverage.lcov
```

### Coverage Badge

After setting up CI coverage reporting, you can add a badge to README.md:

```markdown
[![Coverage](https://codecov.io/gh/USER/REPO/branch/main/graph/badge.svg)](https://codecov.io/gh/USER/REPO)
```

## Performance Benchmarks

### Expected Test Execution Times

| Test Category | Test Count | Execution Time | Notes |
|--------------|-----------|----------------|-------|
| CLI Arguments | ~15 tests | <1 second | Fast validation logic |
| Configuration | ~12 tests | <2 seconds | File I/O with tempfile |
| Timezone | ~10 tests | <1 second | Pure computation |
| Provider Registry | ~8 tests | <1 second | In-memory operations |
| StormGlass Provider | ~10 tests | <3 seconds | Mock HTTP responses |
| OpenWeatherMap Provider | ~10 tests | <3 seconds | Mock HTTP responses |
| **Total** | **~65 tests** | **<10 seconds** | Target met |

### Coverage Generation Times

- HTML report: 20-30 seconds
- LCOV report: 15-25 seconds
- Summary only: 10-15 seconds

**Note**: Coverage generation is intentionally slower due to instrumentation overhead. This is acceptable for periodic reporting.

## Best Practices

### Writing New Tests

1. **Follow Arrange-Act-Assert pattern**:
   ```rust
   #[test]
   fn test_behavior() {
       // Arrange: Setup test data
       let input = create_test_input();
       
       // Act: Execute function under test
       let result = function_under_test(input);
       
       // Assert: Verify expected outcome
       assert_eq!(result, expected);
   }
   ```

2. **Use descriptive test names**:
   - ✓ `test_validate_args_exceeds_7_days_returns_error`
   - ✗ `test_args_validation`

3. **Test one behavior per test**:
   - Each test should verify exactly one aspect of behavior
   - Multiple assertions are OK if testing same behavior

4. **Provide clear failure messages**:
   ```rust
   assert!(
       result.is_ok(),
       "Expected validation to pass, but got error: {:?}",
       result.unwrap_err()
   );
   ```

5. **Use helper functions for common setup**:
   ```rust
   fn create_valid_args() -> Args {
       // Common test setup
   }
   ```

## References

- [Rust testing documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [httpmock documentation](https://docs.rs/httpmock)
- [serial_test documentation](https://docs.rs/serial_test)
- [Constitution Principle VI: Testing Workflow](../../.specify/memory/constitution.md)
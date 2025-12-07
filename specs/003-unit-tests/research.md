# Research: Comprehensive Unit Test Coverage

**Feature**: 003-unit-tests  
**Date**: 2025-12-07  
**Status**: Complete

## Overview

This document consolidates research findings for implementing comprehensive unit test coverage for the windsurf-forecast Rust CLI application. Key decisions focus on coverage tooling, HTTP mocking strategy, environment variable isolation, and timezone handling in tests.

## Technology Decisions

### 1. Coverage Tool Selection

**Decision**: Use `cargo-llvm-cov`

**Rationale**:
- **Modern Tooling**: Built on rustc's native LLVM coverage instrumentation (stable since Rust 1.60)
- **Cross-Platform**: Works consistently across Linux, macOS, and Windows
- **Better Maintained**: Active development, better CI/CD integration than alternatives
- **Standard Output**: Generates standard lcov/html reports compatible with CI tools
- **No External Dependencies**: Uses built-in compiler coverage rather than external tools

**Alternatives Considered**:
- `tarpaulin`: Linux-only, uses ptrace which has limitations and compatibility issues
- `grcov`: Requires more manual setup, less integrated than cargo-llvm-cov
- Manual coverage tracking: Not feasible for maintaining >80% coverage target

**Implementation**:
```bash
# Install
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html

# CI-friendly output
cargo llvm-cov --lcov --output-path lcov.info
```

### 2. HTTP Mocking Strategy

**Decision**: Use `httpmock` crate

**Rationale**:
- **Lightweight**: Minimal dependencies, fast test execution
- **Good Balance**: Simpler than wiremock, more capable than mockito
- **Clear API**: Easy to setup mock servers with specific responses
- **Request Verification**: Can assert on request headers, query params, body
- **Parallel Test Safe**: Each test gets isolated mock server instance

**Alternatives Considered**:
- `wiremock`: More heavyweight, overkill for simple API mocking needs
- `mockito`: Limited request verification capabilities
- Manual mock responses: Harder to maintain, less type-safe

**Implementation Example**:
```rust
use httpmock::prelude::*;

#[tokio::test]
async fn test_stormglass_provider() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.path("/weather/point");
        then.status(200)
            .json_body(json!({
                "hours": [/* mock data */]
            }));
    });
    
    // Test provider against mock server
    // ...
    
    mock.assert();
}
```

### 3. Test Execution Time Constraint

**Decision**: 10-second limit applies to test execution only, not coverage generation

**Rationale**:
- **Developer Experience**: Fast feedback loop during development (`cargo test`)
- **Coverage Overhead**: LLVM instrumentation adds 2-5x overhead, acceptable for CI
- **Practical Separation**: Coverage runs separately (`cargo llvm-cov`) for reporting
- **Realistic Target**: With mocked HTTP and no network I/O, <10s is achievable

**Measurement Approach**:
```bash
# Fast iteration during development
time cargo test  # Must complete in <10 seconds

# Slower coverage analysis for reporting
time cargo llvm-cov --html  # Can take 20-50 seconds, runs less frequently
```

### 4. Environment Variable Isolation

**Decision**: Use `serial_test` crate with `#[serial]` attribute

**Rationale**:
- **Prevents Conflicts**: Tests that modify env vars run sequentially
- **Simple API**: Just add `#[serial]` attribute to tests that need isolation
- **Selective Use**: Only env-dependent tests run serially, others stay parallel
- **No Global State**: Tests don't interfere with developer's real API keys
- **Cross-Platform**: Works on all Rust target platforms

**Alternatives Considered**:
- Custom test fixtures: More complex, error-prone cleanup
- Test-specific environment: Requires complex setup/teardown
- Mocking `std::env::var`: Doesn't prevent actual env conflicts

**Implementation**:
```rust
use serial_test::serial;

#[test]
#[serial]
fn test_missing_api_key() {
    // Temporarily set/unset env vars
    std::env::remove_var("STORMGLASS_API_KEY");
    
    let result = StormGlassProvider::get_api_key();
    assert!(result.is_err());
    
    // Cleanup happens automatically when test ends
}
```

### 5. Timezone Handling in Tests

**Decision**: Pass explicit `Tz` parameters to functions under test

**Rationale**:
- **No Global State**: Avoids modifying process-level timezone which affects all threads
- **Thread-Safe**: Multiple tests can use different timezones in parallel
- **Explicit**: Test intent is clear (what timezone is being tested)
- **Matches Architecture**: Aligns with existing explicit conversion in transform layer
- **Reproducible**: No platform-dependent timezone behavior

**Alternatives Considered**:
- Setting TZ environment variable: Not thread-safe, affects entire process
- Using `chrono_tz::UTC` everywhere: Doesn't test actual timezone conversion
- Mocking time: Unnecessary complexity, harder to understand tests

**Implementation**:
```rust
use chrono_tz::Tz;

#[test]
fn test_timezone_conversion() {
    let utc = UtcTimestamp::from_rfc3339("2025-12-07T12:00:00Z").unwrap();
    let target_tz: Tz = "Asia/Jerusalem".parse().unwrap();
    
    let local = convert_timezone(utc, target_tz).unwrap();
    
    // Assertions on converted timestamp
}
```

## Testing Strategy

### Test Organization

**File Structure**:
- `tests/*.rs` - Integration-style tests that test public APIs
- `src/*_test.rs` or `#[cfg(test)] mod tests` - Unit tests for internal functions
- Test files mirror source structure for easy navigation

**Coverage Targets**:
- Core modules (args, config, provider_registry, forecast_provider): >80% line coverage
- Provider transformations: 100% of documented conversions tested
- Error paths: All user-facing error messages verified

### Test Categories

1. **CLI Argument Tests** (`tests/args_test.rs`)
   - Valid argument combinations
   - Boundary values (days_ahead + first_day_offset = 7)
   - Invalid inputs (negative values, out of range)
   - Missing required parameters

2. **Timezone Tests** (`tests/timezone_test.rs`)
   - UTC to local conversion accuracy
   - Timezone precedence (CLI > config > default)
   - Invalid timezone identifiers
   - "LOCAL" system timezone detection
   - Coordinate-timezone validation warnings

3. **Provider Registry Tests** (`tests/provider_registry_test.rs`)
   - Provider discovery via inventory
   - Provider instantiation
   - Duplicate provider detection
   - Invalid provider name handling

4. **Provider Transformation Tests** (`tests/stormglass_test.rs`, `tests/openweathermap_test.rs`)
   - Wind speed unit conversions (StormGlass: m/s → knots = 1.94384)
   - Field mapping accuracy
   - Optional field handling (None values)
   - Timestamp parsing and timezone conversion
   - Error handling for malformed responses

5. **Configuration Tests** (`tests/config_test.rs`)
   - Config file loading from default path
   - Custom config path support
   - CLI argument precedence over config
   - Missing config file handling
   - Invalid TOML parsing errors
   - Coordinate validation

6. **Error Message Tests** (across all test files)
   - Missing API key messages include env var name
   - Invalid timezone messages include examples
   - CLI validation errors explain what's wrong and how to fix

### Mock Data Strategy

**Provider Response Mocks**:
- Create JSON fixtures matching real API responses
- Include both complete and partial data (missing optional fields)
- Test error responses (401, 402, 403, 500, network timeouts)

**Temporary Test Files**:
- Use `tempfile` crate for config file tests
- Automatic cleanup via RAII (Drop trait)
- No manual cleanup needed

## Implementation Guidelines

### Test Structure Pattern

```rust
// Arrange
let input = /* test data */;

// Act
let result = function_under_test(input);

// Assert
assert_eq!(result, expected);
assert!(condition, "descriptive message");
```

### Async Test Pattern

```rust
#[tokio::test]
async fn test_async_function() {
    // Tests that call async provider methods
}
```

### Serial Test Pattern

```rust
#[test]
#[serial]
fn test_with_env_vars() {
    // Tests that modify environment variables
}
```

### Coverage Measurement

```bash
# During development (fast)
cargo test

# For coverage reporting (slower)
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# CI integration
cargo llvm-cov --lcov --output-path coverage.lcov
```

## Success Metrics

1. **Coverage**: Core modules achieve >80% line coverage (measurable via cargo-llvm-cov)
2. **Speed**: Test suite (`cargo test`) completes in <10 seconds
3. **Reliability**: Zero flaky tests, 100% pass rate on first run
4. **Isolation**: Tests don't require API keys or network access
5. **Maintainability**: Each test validates single behavior with clear failure messages

## Dependencies to Add

```toml
[dev-dependencies]
httpmock = "0.7"           # HTTP mocking for provider tests
serial_test = "3.0"        # Environment variable isolation
tempfile = "3.8"           # Temporary file creation for config tests

# Note: cargo-llvm-cov is installed globally, not in Cargo.toml
```

## References

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [httpmock documentation](https://docs.rs/httpmock)
- [serial_test documentation](https://docs.rs/serial_test)
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- Constitution Principle VI: Testing Workflow
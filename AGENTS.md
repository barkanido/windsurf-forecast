# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Build & Run Commands

### Development Testing Workflow (Constitution VI)

Follow this sequence for all code changes:

```bash
# 1. Check compilation (fix errors and warnings)
cargo check

# 2. Build for testing (DO NOT use --release during development)
cargo build

# 3. Run clippy (address all warnings - fix now or add to TODO)
cargo clippy

# 4. Run unit tests (MUST pass, execution <1 second)
cargo test --lib --tests

# 5. Test the application (debug build for fast iteration)
cargo run -- --provider stormglass --days-ahead 3
cargo run -- --provider openweathermap --days-ahead 2
```

### Unit Testing Commands

```bash
# Run all unit tests (131 tests, <1 second)
cargo test --lib --tests

# Run specific test file
cargo test --test args_test           # CLI argument tests (17 tests)
cargo test --test timezone_test       # Timezone conversion tests (18 tests)
cargo test --test provider_registry_test  # Provider registry tests (15 tests)
cargo test --test config_test         # Configuration tests (24 tests)
cargo test --test stormglass_test     # StormGlass provider tests (28 tests)
cargo test --test openweathermap_test # OpenWeatherMap provider tests (29 tests)

# Run tests matching a pattern
cargo test wind_speed                 # All wind speed related tests
cargo test conversion                 # All conversion tests

# Generate coverage report (slower, for validation)
cargo llvm-cov --html
# View at: target/llvm-cov/html/index.html

# Generate CI-friendly coverage
cargo llvm-cov --lcov --output-path coverage.lcov

# Show coverage summary
cargo llvm-cov --summary-only
```

### Production Testing (Final Validation Only)

```bash
# Use --release ONLY for final end-to-end testing
cargo build --release
cargo run --release -- --provider stormglass
cargo run --release -- --provider openweathermap --days-ahead 3
```

### Quick Reference

```bash
# Most common development commands
cargo check          # Fast compile check
cargo build          # Debug build (fast)
cargo clippy         # Linting
cargo test           # Run unit tests (fast, <1s)
cargo run            # Run debug build

# Production commands (slow, use sparingly)
cargo build --release
cargo run --release
```

## Test Infrastructure

### Test Organization (131 Tests Total)

The project has comprehensive unit test coverage across 6 test files:

1. **[`tests/args_test.rs`](tests/args_test.rs)** (17 tests)
   - CLI argument validation (ranges, constraints)
   - Error message quality verification
   - Provider name validation

2. **[`tests/timezone_test.rs`](tests/timezone_test.rs)** (18 tests)
   - UTC timestamp parsing (RFC3339 format)
   - Timezone conversion accuracy (UTC→Jerusalem, UTC→New York)
   - Timestamp serialization format ("YYYY-MM-DD HH:MM")
   - Timezone precedence rules (CLI > Config > Default)

3. **[`tests/provider_registry_test.rs`](tests/provider_registry_test.rs)** (15 tests)
   - Provider discovery via inventory crate
   - Provider instantiation and validation
   - Registry integrity checks

4. **[`tests/config_test.rs`](tests/config_test.rs)** (24 tests)
   - Config file loading from TOML
   - CLI argument precedence over config
   - Coordinate validation
   - Save/load roundtrip preservation

5. **[`tests/stormglass_test.rs`](tests/stormglass_test.rs)** (28 tests)
   - Wind speed conversion m/s→knots (×1.94384)
   - Field mapping for all weather parameters
   - HTTP error handling (401, 402, 403, 500)
   - Timestamp parsing and timezone conversion

6. **[`tests/openweathermap_test.rs`](tests/openweathermap_test.rs)** (29 tests)
   - Wind speed remains in m/s (NO conversion)
   - Field mapping validation
   - Unix timestamp parsing
   - Error handling

### Test Helpers

**[`tests/common/mod.rs`](tests/common/mod.rs)** provides reusable test utilities:
- Mock data builders for Args, Config structures
- Mock API response builders (StormGlass, OpenWeatherMap)
- Assertion helpers for wind speed conversion and timestamp format validation
- Temporary file creation with automatic cleanup

### Coverage Status

**Core Module Coverage**:
- [`forecast_provider.rs`](src/forecast_provider.rs): **100%** ✅
- [`provider_registry.rs`](src/provider_registry.rs): **84.78%** ✅ (target: >80%)
- [`args.rs`](src/args.rs): **67.65%** (core validation covered)
- [`config.rs`](src/config.rs): **67.38%** (core functionality covered)

**Unit Conversion Coverage**: **100%** ✅
- StormGlass m/s→knots conversion fully tested
- OpenWeatherMap no-conversion behavior verified

### Test Execution Performance

- **Unit test execution**: <1 second (131 tests)
- **Target**: <10 seconds (significantly exceeded)
- **Coverage generation**: 15-30 seconds (separate operation)

### Testing Best Practices

All tests follow these patterns:
- **Arrange-Act-Assert** structure for clarity
- **One behavior per test** for precise failure diagnosis
- **Descriptive test names** indicating what's being tested
- **Mock data builders** for reusability
- **No real network calls** (httpmock for HTTP mocking)
- **No external dependencies** (tempfile for config tests)
- **Serial execution** for environment variable tests (#[serial])

## Critical Non-Obvious Patterns

### Wind Speed Unit Conversion (StormGlass Only)
- [`StormGlassProvider`](src/providers/stormglass.rs:83) converts wind speeds from m/s to knots using `MS_TO_KNOTS = 1.94384`
- [`OpenWeatherMapProvider`](src/providers/openweathermap.rs:34) returns wind speeds in m/s WITHOUT conversion
- This inconsistency means output units differ between providers

### Timezone Conversion Architecture
**Current State**: Timezone conversion happens in the **transform layer**, NOT during serialization.

- **Transform Layer**: Providers parse UTC timestamps as [`UtcTimestamp`](src/forecast_provider.rs:16) and explicitly convert to [`LocalTimestamp`](src/forecast_provider.rs:44) using [`convert_timezone()`](src/forecast_provider.rs:79)
- **Type Safety**: Compiler enforces correct timezone handling - cannot mix `UtcTimestamp` and `LocalTimestamp`
- **User Configuration**: Target timezone set via `--timezone` (or `-z`) CLI flag, supports "LOCAL" for system timezone, defaults to UTC with warning, persisted to config file
- **No Thread-Local State**: Timezone conversion is explicit in provider code, not hidden in serialization
- **Output Format**: JSON timestamps formatted as "YYYY-MM-DD HH:MM" in user-specified timezone

Example flow in [`StormGlassProvider`](src/providers/stormglass.rs:83):
```rust
// 1. Parse API response as UTC
let utc = UtcTimestamp::from_rfc3339(hour.time)?;

// 2. Explicitly convert to target timezone in transform layer
let local = convert_timezone(utc, target_tz)?;

// 3. Return WeatherDataPoint with LocalTimestamp
WeatherDataPoint { time: local, ... }
```

**Key Principle**: Conversion is visible and explicit in provider transform code, not hidden in serialization.

### Provider Registration (Centralized Registry)
When adding a new provider, you ONLY need to add registration in the provider module itself:
1. Implement [`ForecastProvider`](src/forecast_provider.rs:1) trait in your provider module
2. Add `inventory::submit!()` call in provider module to register with [`provider_registry`](src/provider_registry.rs:1)
3. Declare module in [`src/providers/mod.rs`](src/providers/mod.rs:1)

The registry automatically handles provider discovery, instantiation, and validation. No updates needed to [`main.rs`](src/main.rs:1) or [`args.rs`](src/args.rs:1).

Example registration (add to provider module):
```rust
inventory::submit! {
    ProviderMetadata {
        name: "providername",
        description: "Provider Description",
        api_key_var: "PROVIDER_API_KEY",
        instantiate: || {
            let api_key = ProviderName::get_api_key()?;
            Ok(Box::new(ProviderName::new(api_key)))
        },
    }
}
```

### Location Configuration
- Coordinates are configured via CLI arguments (`--lat`, `--lng`) or config file
- No default coordinates - must be provided by user
- Config file path: `~/.windsurf-config.toml` (or custom via `--config`)
- Precedence: CLI arguments > Config file

### Date Range Constraint
- `days_ahead + first_day_offset` must NOT exceed 7 (enforced in [`args.rs`](src/args.rs:64))
- This is a business rule for "reliable forecasts", not an API limitation

### Environment Variable Naming
- StormGlass provider uses: `STORMGLASS_API_KEY`
- OpenWeatherMap provider uses: `OPEN_WEATHER_MAP_API_KEY` (note underscores between words)
- Both providers use `std::env::var()` for consistency
- The `.env` file is loaded once at application startup in [`main.rs`](src/main.rs:1)

## Code Style

- Use `anyhow::Result` for error handling (not `std::result::Result`)
- Provider-specific errors use `thiserror` (see [`StormGlassAPIError`](src/providers/stormglass.rs:15))
- Field naming: use snake_case in structs, camelCase in JSON via `#[serde(rename = "camelCase")]`
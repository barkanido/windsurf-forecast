<!--
Sync Impact Report:
Version: 1.1.0 → 1.2.0 (Updated Timezone Standardization Principle)
Modified Principles: III. Timezone Standardization (complete architectural change)
Added Sections: None
Removed Sections: None
Templates Status:
  ✅ plan-template.md - Already aligned with cargo workflow
  ✅ spec-template.md - Compatible with testing requirements
  ✅ tasks-template.md - Supports iterative testing approach
  ✅ AGENTS.md - UPDATED with new timezone conversion architecture
  ✅ README.md - UPDATED to remove environment variable references
Follow-up TODOs: None - all documentation updated
-->

# Weather Forecast Application Constitution

## Core Principles

### I. Provider Architecture Pattern

The application MUST use a trait-based provider architecture where:
- Each weather API provider implements the [`ForecastProvider`](../../src/forecast_provider.rs:1) trait
- Providers are self-contained modules in [`src/providers/`](../../src/providers/) 
- Common data structures ([`WeatherDataPoint`](../../src/forecast_provider.rs:19)) ensure interoperability
- Provider registration requires updates in exactly 3 locations (see [`AGENTS.md`](../../AGENTS.md:30))

**Rationale**: This pattern enables easy integration of new weather APIs without modifying core application logic, supporting the project's goal of multi-provider support while maintaining code maintainability.

### II. Explicit Unit Handling

Wind speed and other measurements MUST have clearly documented unit conventions:
- Each provider documents its output units in code comments
- Unit conversions (e.g., m/s to knots) are provider-specific, not enforced globally
- Inconsistencies between providers are INTENTIONAL and reflect upstream API design
- Output JSON MUST include metadata describing units for all measurements

**Rationale**: Different weather APIs return data in different units. Rather than force artificial consistency, we preserve provider-specific units and document them explicitly, preventing silent unit conversion bugs.

### III. Timezone Conversion Architecture

All timestamps MUST follow explicit conversion in the transform layer:
- **Parsed as UTC**: API responses parsed as [`UtcTimestamp`](../../src/forecast_provider.rs:16) newtype wrapper
- **Converted in Transform Layer**: Providers explicitly call [`convert_timezone(utc, target_tz)`](../../src/forecast_provider.rs:79) to create [`LocalTimestamp`](../../src/forecast_provider.rs:44)
- **Type Safety**: Compiler enforces correct timezone handling - cannot mix `UtcTimestamp` and `LocalTimestamp` types
- **User Configuration**: Target timezone set via `--timezone` (or `-z`) CLI flag, supports "LOCAL" for system timezone detection, persisted to config file, defaults to UTC with warning
- **No Thread-Local State**: Timezone conversion is explicit in provider code, not hidden in serialization layer
- **Output Format**: JSON timestamps formatted as "YYYY-MM-DD HH:MM" (not ISO 8601) in user-specified timezone

Example from [`StormGlassProvider`](../../src/providers/stormglass.rs:83):
```rust
// 1. Parse API response as UTC
let utc = UtcTimestamp::from_rfc3339(hour.time)?;

// 2. Explicitly convert to target timezone in transform layer
let local = convert_timezone(utc, target_tz)?;

// 3. Return WeatherDataPoint with LocalTimestamp
WeatherDataPoint { time: local, ... }
```

**Rationale**: Explicit timezone conversion in the transform layer makes the conversion point visible and testable in provider code. Type safety prevents timezone bugs at compile time. Removing thread-local state eliminates concurrency issues and makes the code easier to reason about. User-configurable timezones support diverse geographic locations while maintaining a clear default.

### IV. Configuration Management

Location coordinates and business rules configuration:
- **Coordinates**: Must be provided via CLI arguments (`--lat`, `--lng`) or config file - no defaults
- **Config File**: `~/.windsurf-config.toml` stores user preferences (timezone, coordinates, provider)
- **Precedence**: CLI arguments override config file values
- **Persistence**: CLI-specified timezone is automatically saved to config file for future use
- **Date Range Constraint**: `days_ahead + first_day_offset ≤ 7` (enforced in [`args.rs:64`](../../src/args.rs:64)) - business rule for reliable forecasts

**Rationale**: Configurable coordinates support multiple locations while config file persistence reduces repetitive CLI arguments. The date range constraint remains a business rule reflecting forecast reliability requirements, not an API limitation.

### V. Error Transparency

Error handling MUST provide actionable information:
- HTTP status codes map to user-friendly messages (402 = quota exceeded, 403 = invalid key, etc.)
- Provider-specific errors use [`thiserror`](../../src/providers/stormglass.rs:15) for structured error types
- Application-level errors use [`anyhow::Result`](../../src/main.rs:1) for flexibility
- Error messages MUST guide users toward resolution (e.g., "Set STORMGLASS_API_KEY in .env")

**Rationale**: Weather APIs have various failure modes (quota limits, authentication, service outages). Clear error messages reduce debugging time and improve user experience.

### VI. Testing Workflow

All code changes MUST follow an iterative testing workflow before final validation:

1. **`cargo check`** - MUST pass without errors or warnings
   - Fix all compilation errors immediately
   - Address warnings seriously; fix if they indicate real issues at current implementation stage
   - Some warnings may be deferred if they relate to incomplete features

2. **`cargo build`** - MUST succeed for development testing
   - DO NOT use `--release` flag during development (it is slow)
   - Use debug builds for rapid iteration and testing

3. **`cargo clippy`** - MUST be run and addressed
   - Treat clippy warnings as items requiring action
   - Either fix warnings immediately if straightforward
   - Or add to TODO task list for later resolution if complex
   - Document rationale if intentionally ignoring specific clippy suggestions

4. **`cargo test`** - MUST run and pass for all changes
   - Run unit tests with `cargo test --lib --tests` (fast, <1 second)
   - Run specific test files with `cargo test --test <test_name>`
   - All 131 unit tests must pass before proceeding
   - Test suite execution time MUST remain under 10 seconds

5. **`cargo run --release`** - ONLY for final end-to-end testing
   - Use `--release` flag exclusively for production-like validation
   - Run after all development testing passes
   - Validate complete user workflows before deployment

**Test Infrastructure** (131 comprehensive tests):
- [`tests/args_test.rs`](../../tests/args_test.rs) - CLI argument validation (17 tests)
- [`tests/timezone_test.rs`](../../tests/timezone_test.rs) - Timezone conversion (18 tests)
- [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs) - Provider discovery (15 tests)
- [`tests/config_test.rs`](../../tests/config_test.rs) - Configuration management (24 tests)
- [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs) - StormGlass provider (28 tests)
- [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs) - OpenWeatherMap provider (29 tests)
- [`tests/common/mod.rs`](../../tests/common/mod.rs) - Test helpers and mock data builders

**Coverage Requirements**:
- Core modules: >80% line coverage target
- [`forecast_provider.rs`](../../src/forecast_provider.rs): 100% coverage achieved
- [`provider_registry.rs`](../../src/provider_registry.rs): 84.78% coverage achieved
- All documented unit conversions: 100% test coverage required

**Coverage Reporting** (separate from test execution):
```bash
# Generate HTML coverage report (slower, for validation)
cargo llvm-cov --html
# View at: target/llvm-cov/html/index.html

# Generate CI-friendly coverage report
cargo llvm-cov --lcov --output-path coverage.lcov
```

**Rationale**: This workflow balances development speed with code quality. Debug builds enable rapid iteration, while strict quality gates (check, clippy, test) catch issues early. Comprehensive unit tests verify behavior and prevent regressions. The `--release` flag's optimization overhead is reserved for final validation, preventing slow feedback loops during active development.

## Architecture Standards

### Provider Extension Protocol

When adding a new weather provider, developers MUST:

1. Create provider module in [`src/providers/[name].rs`](../../src/providers/)
2. Implement [`ForecastProvider`](../../src/forecast_provider.rs:1) trait with all required methods
3. Register provider in [`src/providers/mod.rs`](../../src/providers/mod.rs:1)
4. Update [`create_provider()`](../../src/main.rs:39) to instantiate the new provider
5. Update [`run()`](../../src/main.rs:130) to retrieve the provider's API key
6. Update [`validate_provider()`](../../src/args.rs:77) to accept the provider name
7. Document provider-specific behavior in [`ADDING_PROVIDERS.md`](../../ADDING_PROVIDERS.md:1)

Missing any of these steps will cause silent failures or runtime panics.

### Dependency Management

External dependencies MUST:
- Use semantic versioning with locked patch versions in [`Cargo.toml`](../../Cargo.toml:1)
- Enable only required features (e.g., `reqwest = { features = ["json"] }`)
- Be justified in documentation if adding significant binary size
- Use async-first libraries (tokio ecosystem preferred)

### Field Naming Convention

- Rust structs use `snake_case` for field names
- JSON output uses `camelCase` via `#[serde(rename = "camelCase")]`
- This maintains Rust idioms internally while providing JavaScript-friendly JSON

## Development Workflow

### CLI-First Development

All functionality MUST be exposed via command-line interface:
- Use [`clap`](../../src/args.rs:1) with derive macros for argument parsing
- Validate arguments before execution (see [`validate_provider()`](../../src/args.rs:77))
- Support `--help` with clear descriptions of all options
- Exit codes: 0 = success, non-zero = error with stderr message

### Configuration Management

Environment variables MUST be managed via:
- `.env` file support using [`dotenv`](../../Cargo.toml:14) crate
- `.env.example` template provided for reference
- `.env` in `.gitignore` to prevent API key exposure
- Each provider uses a uniquely named environment variable (e.g., `STORMGLASS_API_KEY`, `OPEN_WEATHER_MAP_API_KEY`)

### Documentation Standards

Code documentation MUST include:
- Provider-specific behavior in inline comments
- Non-obvious patterns documented in [`AGENTS.md`](../../AGENTS.md:1)
- User-facing instructions in [`README.md`](../../README.md:1)
- Provider extension guide in [`ADDING_PROVIDERS.md`](../../ADDING_PROVIDERS.md:1)
- Clickable file references using relative paths in markdown

## Governance

### Constitution Authority

This constitution supersedes coding preferences, style debates, and undocumented conventions. When conflicts arise:

1. Constitution principles take precedence
2. Documented architectural patterns in [`AGENTS.md`](../../AGENTS.md:1) clarify implementation
3. Ambiguities require constitution amendment, not ad-hoc decisions

### Amendment Process

Constitution changes require:
1. Documented rationale for the change
2. Version bump according to semantic versioning:
   - **MAJOR**: Backward-incompatible principle removal or redefinition
   - **MINOR**: New principle or materially expanded guidance
   - **PATCH**: Clarifications, wording fixes, non-semantic refinements
3. Review of dependent templates (plan, spec, tasks)
4. Update of Sync Impact Report in constitution file header

### Complexity Justification

Deviations from simplicity MUST be justified:
- Provider-specific unit handling → Reflects upstream API reality
- Hard-coded location → Single-purpose application scope
- Explicit timezone conversion in transform layer → Type safety and testability priority
- Newtype wrappers for timestamps → Compile-time timezone correctness guarantees
- Testing workflow structure → Balance development speed with code quality

### Compliance Review

All code changes MUST verify:
- Provider architecture pattern maintained
- Unit documentation present for new measurements
- Timezone handling follows explicit conversion in transform layer principle
- Coordinates provided via CLI or config (no hard-coded defaults)
- Type safety enforced via `UtcTimestamp` and `LocalTimestamp` wrappers
- Error messages are actionable
- Testing workflow followed (check → build → clippy → release)
- [`AGENTS.md`](../../AGENTS.md:1) updated for non-obvious patterns

**Version**: 1.2.0 | **Ratified**: 2025-12-07 | **Last Amended**: 2025-12-07

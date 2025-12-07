# Implementation Plan: Comprehensive Unit Test Coverage

**Branch**: `003-unit-tests` | **Date**: 2025-12-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-unit-tests/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This feature adds comprehensive unit test coverage for the windsurf-forecast project, focusing on core business logic (CLI arguments, timezone conversion, provider registry), provider data transformations, configuration management, and error handling. Tests will use standard Rust testing infrastructure with cargo-llvm-cov for coverage reporting, httpmock for HTTP mocking, and serial_test for environment variable isolation. The goal is >80% coverage for core modules with execution under 10 seconds.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**:
- Core: tokio 1.x (async runtime), reqwest 0.11 (HTTP), serde 1.0 (serialization)
- CLI: clap 4.0 (args parsing), dialoguer 0.11 (interactive prompts)
- Time: chrono 0.4, chrono-tz 0.8, iana-time-zone 0.1, tzf-rs 0.4
- Error: anyhow 1.0 (app-level), thiserror 1.0 (provider-level)
- Config: toml 0.8, dirs 5.0, dotenv 0.15
- Provider: async-trait 0.1, inventory 0.3
- **Testing (new)**: cargo-llvm-cov (coverage), httpmock (HTTP mocking), serial_test (env isolation)

**Storage**: Filesystem (config file: ~/.windsurf-config.toml, .env for API keys, JSON output)
**Testing**: `cargo test` (standard Rust test framework with #[test] and #[cfg(test)])
**Target Platform**: Cross-platform CLI (Linux, macOS, Windows)
**Project Type**: Single project (CLI application with async provider architecture)
**Performance Goals**: Test suite execution <10 seconds (coverage generation separate, can be slower)
**Constraints**:
- Tests must not make real network calls (use httpmock)
- Tests must not conflict with developer's API keys (use serial_test)
- Tests must be reproducible and non-flaky (no parallel execution issues)

**Scale/Scope**:
- ~1,500 lines of Rust code across 7 source files
- 2 weather providers (StormGlass, OpenWeatherMap) + provider registry
- Target: >80% line coverage for core modules (args, config, provider_registry, forecast_provider)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with constitution principles from `.specify/memory/constitution.md`:

- [x] **Provider Architecture Pattern**: Tests verify ForecastProvider trait implementation and provider registry functionality (FR-003, FR-004)
- [x] **Explicit Unit Handling**: Tests verify unit conversions (StormGlass m/s→knots = 1.94384, OpenWeatherMap stays m/s) per FR-006
- [x] **Timezone Conversion Architecture**: Tests verify explicit timezone conversion in transform layer using UtcTimestamp→LocalTimestamp newtypes (FR-007, FR-012)
- [x] **Configuration Management**: Tests verify CLI args override config file, coordinate validation, timezone precedence per FR-005
- [x] **Error Transparency**: Tests verify error messages are actionable for common failures (missing API key, invalid timezone, invalid args) per FR-008
- [x] **Testing Workflow**: This feature implements comprehensive testing to support the constitution's testing workflow principle
- [x] **Provider Extension Protocol**: Tests verify provider registration via inventory crate follows centralized registry pattern
- [x] **CLI-First Development**: Tests verify CLI argument parsing, validation, and precedence rules per FR-001
- [x] **Configuration Management**: Tests verify environment variable handling never exposes hardcoded API keys, uses serial_test for isolation

**Constitution Compliance Notes**:
- This feature enhances rather than modifies existing architecture patterns
- Tests will validate that existing code follows constitution principles
- No new weather providers added (testing existing StormGlass and OpenWeatherMap)
- No changes to unit handling or timezone architecture (tests verify current behavior)
- Tests use explicit timezone passing rather than global state modification

*Note: Check "Complexity Justification" section if any violations need justification.*

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
src/
├── main.rs                    # Entry point, config loading, provider orchestration
├── args.rs                    # CLI argument parsing and validation (clap)
├── config.rs                  # Config file loading/saving, timezone validation
├── forecast_provider.rs       # ForecastProvider trait, WeatherDataPoint, timezone newtypes
├── provider_registry.rs       # Provider discovery and instantiation (inventory)
└── providers/
    ├── mod.rs                 # Provider module declarations
    ├── stormglass.rs          # StormGlass provider implementation
    └── openweathermap.rs      # OpenWeatherMap provider implementation

tests/
├── args_test.rs               # CLI argument validation tests (NEW)
├── config_test.rs             # Config file and precedence tests (NEW)
├── provider_registry_test.rs  # Provider registry tests (NEW)
├── timezone_test.rs           # Timezone conversion tests (NEW)
├── stormglass_test.rs         # StormGlass transformation tests with mocks (NEW)
└── openweathermap_test.rs     # OpenWeatherMap transformation tests with mocks (NEW)
```

**Structure Decision**: Single CLI project following Rust conventions with `tests/` directory for integration-style unit tests. Each test file corresponds to a source module or feature area. Tests use `#[cfg(test)]` modules within source files for true unit tests, and `tests/` directory for larger integration tests that test multiple modules together.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. All constitution principles are satisfied by the test implementation approach.

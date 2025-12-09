# Implementation Plan: Configuration Data Flow Simplification

**Branch**: `004-config-refactor` | **Date**: 2025-12-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-config-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

This refactor consolidates scattered configuration logic from [`main.rs`](../../src/main.rs:1) (275+ lines) into a dedicated [`src/config/`](../../src/config/) module with clear separation of concerns. The implementation introduces a single [`ResolvedConfig`](../../src/config/types.rs:1) structure for all validated configuration, replaces three separate precedence implementations with one generic helper function, and establishes explicit opt-in persistence via `--save` flag (removing the current auto-save timezone behavior). This architectural change reduces main function complexity by >60% while maintaining 100% backward compatibility with existing CLI arguments, config file format, and validation rules.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: clap 4.0 (CLI parsing), serde 1.0 + toml 0.8 (serialization), chrono 0.4 + chrono-tz 0.8 (timezones), anyhow 1.0 (error handling)
**Storage**: File-based configuration (~/.windsurf-config.toml), environment variables (.env for API keys)
**Testing**: cargo test (131 unit tests in 6 test files, <1 second execution), cargo clippy (linting)
**Target Platform**: CLI application (cross-platform: Windows, Linux, macOS)
**Project Type**: Single Rust binary project with library crate support
**Performance Goals**: Configuration resolution <10ms, CLI startup <100ms
**Constraints**: Zero breaking changes to CLI interface or config file format, all 131 existing tests must pass
**Scale/Scope**: ~2500 LOC codebase, refactoring affects 3 core files (main.rs, config.rs, args.rs) + new config/ module

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with constitution principles from [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md:1):

- [x] **Provider Architecture Pattern**: Not applicable - refactor does not modify provider registration or trait implementation
- [x] **Explicit Unit Handling**: Not applicable - refactor does not touch unit conversion or measurement handling
- [x] **Timezone Conversion Architecture**: Compliant - maintains explicit conversion in transform layer; refactor only consolidates timezone config loading, does not change conversion mechanism
- [x] **Configuration Management**: Compliant - maintains config file format, adds `--save` flag for explicit persistence (removes auto-save); precedence order unchanged (CLI > Config > Default); coordinates remain configurable via CLI/config
- [x] **Error Transparency**: Enhanced - refactor will improve error messages by including parameter source (CLI/config/default) in validation failures
- [x] **Testing Workflow**: Compliant - follows cargo check → build → clippy → test workflow; all 131 tests must pass without modification
- [x] **Provider Extension Protocol**: Not applicable - refactor does not affect provider registration
- [x] **CLI-First Development**: Compliant - maintains all existing CLI arguments without breaking changes; [`args.rs`](../../src/args.rs:1) remains separate for CLI parsing
- [x] **Configuration Management**: Compliant - maintains .env for API keys (outside refactor scope); TOML config file format unchanged

**Constitution Changes Required**: None. This refactor is purely internal reorganization with no changes to external interfaces or core architectural principles.

*All gates passed. Proceeding to Phase 0 research.*

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

```text
src/
├── main.rs                    # Application entry point (simplified from 275 to ~100 lines)
├── lib.rs                     # Library exports
├── args.rs                    # CLI argument parsing (unchanged interface)
├── forecast_provider.rs       # Provider trait and data structures
├── provider_registry.rs       # Provider discovery and instantiation
├── test_utils.rs              # Test utilities
├── config/                    # NEW: Configuration module (organized by concern)
│   ├── mod.rs                 # Module exports and public API
│   ├── types.rs               # ResolvedConfig, ConfigSources structures
│   ├── loader.rs              # File I/O: load/save TOML config
│   ├── resolver.rs            # Precedence logic and validation
│   └── timezone.rs            # Timezone-specific configuration (from config.rs)
└── providers/
    ├── mod.rs
    ├── stormglass.rs
    └── openweathermap.rs

tests/
├── args_test.rs               # CLI argument validation (17 tests)
├── config_test.rs             # Configuration management (24 tests) - UPDATE for new structure
├── timezone_test.rs           # Timezone conversion (18 tests)
├── provider_registry_test.rs  # Provider discovery (15 tests)
├── stormglass_test.rs        # StormGlass provider (28 tests)
├── openweathermap_test.rs    # OpenWeatherMap provider (29 tests)
└── common/
    └── mod.rs                 # Test helpers and mock data builders
```

**Structure Decision**: Single Rust project structure. The refactor extracts configuration logic from [`src/config.rs`](../../src/config.rs:1) (354 lines) and [`src/main.rs`](../../src/main.rs:1) (275+ lines) into a new [`src/config/`](../../src/config/) module with 4 files organized by concern:
- [`types.rs`](../../src/config/types.rs:1): Core data structures (ResolvedConfig, ConfigSources)
- [`loader.rs`](../../src/config/loader.rs:1): File I/O operations (load/save TOML)
- [`resolver.rs`](../../src/config/resolver.rs:1): Precedence resolution and validation logic
- [`timezone.rs`](../../src/config/timezone.rs:1): Timezone-specific configuration (extracted from current config.rs)

The [`src/args.rs`](../../src/args.rs:1) module remains separate to maintain single responsibility (CLI parsing vs config resolution).

## Complexity Tracking

No constitution violations. This refactor reduces complexity by:
- Consolidating 3 separate precedence implementations into 1 generic function
- Moving 175+ lines of configuration logic from main.rs to dedicated config module
- Establishing clear boundaries between CLI parsing (args.rs) and config resolution (config/)
- Replacing scattered variables with single ResolvedConfig structure

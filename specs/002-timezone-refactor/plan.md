# Implementation Plan: Timezone Conversion Architecture Refactor

**Branch**: `002-timezone-refactor` | **Date**: 2025-12-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-timezone-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Move timezone conversion from serialization layer (thread-local state with custom serde serializer) to transform layer for better testability, explicit control, and type safety. Introduce newtype wrappers (`UtcTimestamp` and `LocalTimestamp`) for compile-time timezone distinction. Make target timezone user-configurable via CLI flag (`--timezone`/`--tz`) and environment variable (`FORECAST_TIMEZONE`), defaulting to UTC with warning when not specified.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: chrono 0.4, chrono-tz 0.8, serde 1.0, clap 4.4, anyhow 1.0, async-trait 0.1
**Storage**: N/A (CLI application with JSON output to stdout)
**Testing**: cargo test with unit tests in provider modules
**Target Platform**: Cross-platform CLI (Windows, Linux, macOS)
**Project Type**: Single project (CLI application)
**Performance Goals**: Sub-second API response transformation, <100ms timezone conversion overhead
**Constraints**: Must maintain backward-compatible JSON output format ("YYYY-MM-DD HH:MM"), must support DST transitions correctly
**Scale/Scope**: 2 weather providers currently (StormGlass, OpenWeatherMap), ~1000 LOC total, single executable

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with constitution principles from `.specify/memory/constitution.md`:

- [x] **Provider Architecture Pattern**: Not adding new providers; modifying existing provider transform functions to accept timezone parameter
- [x] **Explicit Unit Handling**: No new measurements; only changing timezone handling (not affecting wind speed units)
- [ ] **Timezone Standardization**: INTENTIONAL VIOLATION - Moving from hard-coded Asia/Jerusalem to user-configurable timezone (see Complexity Justification)
- [x] **Hard-Coded Configuration**: Location coordinates unchanged; date range constraint unchanged
- [x] **Error Transparency**: Adding structured timezone conversion error messages with format "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}"
- [x] **Testing Workflow**: Will follow cargo check → build → clippy → run workflow throughout implementation
- [x] **Provider Extension Protocol**: Not adding providers; modifying transform layer contract (timezone parameter)
- [x] **CLI-First Development**: Adding `--timezone` (and `--tz` alias) CLI flag with help documentation
- [x] **Configuration Management**: Adding `FORECAST_TIMEZONE` environment variable (CLI takes precedence)

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

```text
src/
├── main.rs                    # Entry point, CLI handling
├── args.rs                    # Command-line argument parsing (add --timezone flag)
├── config.rs                  # Configuration management (add timezone config)
├── forecast_provider.rs       # Core trait and WeatherDataPoint (modify for newtype wrappers)
├── provider_registry.rs       # Provider discovery (no changes needed)
└── providers/
    ├── mod.rs                 # Provider module declarations
    ├── stormglass.rs         # StormGlass API provider (modify transform to accept timezone)
    └── openweathermap.rs     # OpenWeatherMap API provider (modify transform to accept timezone)

specs/002-timezone-refactor/
├── spec.md                    # Feature specification
├── plan.md                    # This file
├── research.md               # Phase 0 output (research findings)
├── data-model.md             # Phase 1 output (entity definitions)
├── quickstart.md             # Phase 1 output (implementation guide)
└── contracts/                # Phase 1 output (Rust type definitions)
    ├── newtype_wrappers.rs   # UtcTimestamp and LocalTimestamp definitions
    ├── provider_trait.rs     # Updated ForecastProvider trait signature
    └── README.md             # Contract documentation
```

**Structure Decision**: Single project with modular provider architecture. All changes confined to existing src/ directory structure. New timezone configuration integrated into existing config.rs pattern. Provider transforms updated in-place to accept timezone parameter.

## Constitution Check (Post-Design Re-evaluation)

*Re-evaluated after Phase 1 design completion*

### Design Compliance Verification

- [x] **Provider Architecture Pattern**: ✅ COMPLIANT - Design maintains trait-based architecture; only adds timezone parameter to existing trait method
- [x] **Explicit Unit Handling**: ✅ COMPLIANT - No changes to unit handling; timezone conversion documented separately from wind speed conversions
- [x] **Timezone Standardization**: ⚠️ JUSTIFIED VIOLATION - Moving from hard-coded Asia/Jerusalem to user-configurable timezone (see Complexity Justification for rationale)
- [x] **Hard-Coded Configuration**: ✅ COMPLIANT - Location coordinates remain unchanged; date range constraint unchanged
- [x] **Error Transparency**: ✅ COMPLIANT - Design includes structured error messages with all necessary information for debugging
- [x] **Testing Workflow**: ✅ COMPLIANT - Quickstart guide follows cargo check → build → clippy → run workflow
- [x] **Provider Extension Protocol**: ✅ COMPLIANT - Design maintains self-contained provider modules with centralized registry; no changes to registration pattern
- [x] **CLI-First Development**: ✅ COMPLIANT - Design adds `--timezone` CLI flag with full help documentation and environment variable support
- [x] **Configuration Management**: ✅ COMPLIANT - Design uses environment variable `FORECAST_TIMEZONE` with CLI precedence

### Architectural Improvements Aligned with Constitution

1. **Error Transparency Enhanced**: New structured timezone error format provides more actionable information than previous implicit conversions
2. **Testing Workflow Improved**: Moving conversion to transform layer enables testing without serialization overhead (aligns with Constitution VI testing principles)
3. **CLI-First Maintained**: Timezone configuration exposed via CLI with clear precedence rules (CLI > ENV > default)
4. **Provider Pattern Preserved**: All providers continue to implement same trait interface; newtype wrappers enforce contracts at compile time

### Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking change to provider trait signature | Phased implementation in quickstart.md ensures incremental validation |
| Backward compatibility concerns | Custom LocalTimestamp serialization maintains exact output format |
| DST edge cases | chrono-tz library handles DST automatically; documented in research.md |
| Performance overhead of timezone conversion | Benchmarked <100ms overhead; acceptable for CLI application |

### Constitution Amendment Considerations

**No amendments required.** The Timezone Standardization violation is justified by:
- User story requirements (FR-004, FR-005, US-4)
- International usability needs
- Improved testability and explicitness
- Removal of thread-local state (better than current implementation)

The violation is a **necessary improvement** that aligns with Constitution spirit (Error Transparency, Testing Workflow) while addressing legitimate user needs.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Timezone Standardization (Constitution III) | User-configurable timezone required for international usage; hard-coded Asia/Jerusalem limits usability | Keeping hard-coded timezone violates user story requirements (FR-004, FR-005) and prevents system from serving users in different timezones |
| Moving conversion from serialization to transform layer | Improves testability (can test timezone conversion without JSON serialization), makes conversion explicit and traceable, eliminates thread-local state | Keeping conversion in serialization layer maintains hidden dependencies, makes testing harder, and couples timezone logic to output format |

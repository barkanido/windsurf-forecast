# Implementation Plan: Centralized Provider Registry

**Branch**: `001-provider-registry` | **Date**: 2025-12-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-provider-registry/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Eliminate the anti-pattern of requiring manual updates to 3 separate locations when adding weather providers. Implement a centralized provider registry using the `inventory` crate pattern where providers self-register via `inventory::submit!()` macro calls in their modules. The registry will automatically discover providers at runtime initialization, validate provider names during CLI parsing, and provide dynamic help text generation.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: inventory (for provider registry), existing: tokio, reqwest, serde, chrono, clap, anyhow, thiserror
**Storage**: N/A (stateless CLI application)
**Testing**: cargo test (unit tests), manual integration testing via `cargo run`
**Target Platform**: Cross-platform CLI (Windows, Linux, macOS)
**Project Type**: Single project (CLI application)
**Performance Goals**: <100ms registry initialization, negligible overhead vs current hardcoded approach
**Constraints**: Zero-cost abstraction (no runtime performance degradation), backward-compatible API for existing providers
**Scale/Scope**: 2 existing providers (StormGlass, OpenWeatherMap), designed to support 10+ providers without code changes

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with constitution principles from `.specify/memory/constitution.md`:

### Initial Check (Before Phase 0)

- [x] **Provider Architecture Pattern**: ✅ Maintains trait-based architecture - providers still implement ForecastProvider trait
- [x] **Explicit Unit Handling**: ✅ No changes to unit handling or measurements
- [x] **Timezone Standardization**: ✅ No changes to timezone handling
- [x] **Hard-Coded Configuration**: ✅ No changes to location coordinates or date range constraints
- [x] **Error Transparency**: ✅ Improves error messages - registry provides actionable error listing all available providers when invalid name used
- [x] **Testing Workflow**: ✅ Will follow cargo check → build → clippy → run workflow during implementation
- [⚠️] **Provider Extension Protocol**: ⚠️ INTENTIONAL VIOLATION - This feature replaces the 7-step protocol with 1-step registration (see Complexity Tracking)
- [x] **CLI-First Development**: ✅ CLI interface unchanged, `--help` dynamically generated from registry
- [x] **Configuration Management**: ✅ Environment variable management unchanged - providers still declare their own API key variables

### Post-Design Re-evaluation (After Phase 1)

- [x] **Provider Architecture Pattern**: ✅ CONFIRMED - Design preserves trait-based architecture, adds registry layer on top
- [x] **Explicit Unit Handling**: ✅ CONFIRMED - No changes to unit handling in design
- [x] **Timezone Standardization**: ✅ CONFIRMED - No changes to timezone handling in design
- [x] **Hard-Coded Configuration**: ✅ CONFIRMED - Location and date range logic untouched
- [x] **Error Transparency**: ✅ ENHANCED - Registry validation provides clear error messages with available provider list
- [x] **Testing Workflow**: ✅ CONFIRMED - Quickstart.md follows full cargo check → build → clippy → run → release workflow
- [✅] **Provider Extension Protocol**: ✅ IMPROVED - Design successfully replaces error-prone 7-step protocol with streamlined 1-step registry submission
- [x] **CLI-First Development**: ✅ CONFIRMED - CLI interface preserved, help text generation automated
- [x] **Configuration Management**: ✅ CONFIRMED - .env pattern unchanged, api_key_var metadata added for documentation

**Design Phase Conclusion**: All constitution principles satisfied or intentionally improved. The "violation" of Provider Extension Protocol is the core objective - replacing the anti-pattern with a better pattern. Constitution will be updated to reflect the new registry-based protocol.

*Note: Constitution update documented in spec.md requirements FR-095 and SC-007.*

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
├── main.rs                    # Main entry point, uses registry for provider instantiation
├── args.rs                    # CLI argument parsing, uses registry for validation
├── config.rs                  # Configuration management (unchanged)
├── forecast_provider.rs       # ForecastProvider trait (unchanged)
├── provider_registry.rs       # NEW: Centralized registry using inventory crate
└── providers/
    ├── mod.rs                 # Module declarations (unchanged)
    ├── stormglass.rs          # Adds inventory::submit!() call
    └── openweathermap.rs      # Adds inventory::submit!() call

specs/001-provider-registry/
├── plan.md                    # This file
├── research.md                # Phase 0 output
├── data-model.md              # Phase 1 output
├── quickstart.md              # Phase 1 output
└── contracts/                 # Phase 1 output (Rust API contracts)
```

**Structure Decision**: Single project structure (CLI application). New `provider_registry.rs` module will centralize all provider registration logic. Existing provider modules will be modified to add self-registration via `inventory::submit!()` macro. The three manual registration locations in `main.rs` and `args.rs` will be replaced with registry lookups.

## Complexity Tracking

> **Filled because Constitution Check identified intentional violation**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Provider Extension Protocol (7-step → 1-step) | Current 3-location manual registration is error-prone, violates DRY principle, and causes silent failures when steps are missed. This feature IS the fix for this anti-pattern. | Keeping 7-step protocol would mean NOT fixing the core problem this feature addresses. The violation is the PURPOSE of this feature. |

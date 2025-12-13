# Implementation Plan: AI-Powered Forecast Analysis

**Branch**: `005-ai-integration` | **Date**: 2025-12-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-ai-integration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add OpenAI ChatGPT integration to enable AI-powered natural language analysis of weather forecasts. Users can ask questions about forecast data using the `--ai` flag, optionally specify custom prompts via `--prompt-file`, and select between ChatGPT models using `--model`. The system generates the forecast first, then sends it to OpenAI with the user's prompt for analysis, returning only the AI response.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75+ (edition 2021)
**Primary Dependencies**: reqwest 0.11+ (async HTTP client), serde 1.0 + serde_json 1.0 (JSON serialization), tokio 1.0+ (async runtime), clap 4.4+ (CLI parsing), anyhow 1.0 (error handling)
**Storage**: N/A (stateless API integration, uses existing .env for API keys)
**Testing**: cargo test (unit tests following existing test infrastructure pattern)
**Target Platform**: Cross-platform CLI (Windows, Linux, macOS)
**Project Type**: Single project (CLI application with async HTTP integration)
**Performance Goals**: AI API response within 60 seconds (timeout), minimal latency overhead (<100ms for non-AI operations)
**Constraints**: 60-second timeout for OpenAI API calls, 32KB max prompt file size, must use async/await for HTTP operations
**Scale/Scope**: Single user CLI tool, adds ~3 new CLI flags, 1 new module (src/ai/), ~5 new unit tests

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with constitution principles from `.specify/memory/constitution.md`:

- [x] **Provider Architecture Pattern**: N/A - Not adding weather providers, integrating with existing provider output
- [x] **Explicit Unit Handling**: N/A - AI receives existing forecast JSON with preserved units from providers
- [x] **Timezone Standardization**: ✅ AI receives forecast JSON with timestamps already converted per existing architecture
- [x] **Hard-Coded Configuration**: ✅ No changes to coordinates or date range constraints
- [x] **Error Transparency**: ✅ All error messages designed to be actionable (see edge cases in spec: missing API key, timeouts, invalid models)
- [x] **Testing Workflow**: ✅ Will follow cargo check → build → clippy → test → run workflow
- [x] **Provider Extension Protocol**: N/A - Not adding weather providers
- [x] **CLI-First Development**: ✅ All features exposed via CLI: --ai, --prompt-file, --model flags with --help documentation
- [x] **Configuration Management**: ✅ OPENAI_API_KEY follows .env pattern, consistent with existing provider API keys

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
├── main.rs              # Entry point - integrate AI flow after forecast generation
├── args.rs              # CLI args - add --ai, --prompt-file, --model flags
├── forecast_provider.rs # Trait - no changes needed
├── provider_registry.rs # Registry - no changes needed
├── config/              # Config - no changes needed
│   ├── mod.rs
│   ├── types.rs
│   ├── loader.rs
│   ├── resolver.rs
│   └── timezone.rs
├── providers/           # Providers - no changes needed
│   ├── mod.rs
│   ├── stormglass.rs
│   ├── openweathermap.rs
│   └── windy.rs
└── ai/                  # NEW: AI integration module
    ├── mod.rs           # Public API exports
    ├── client.rs        # OpenAI REST API client with reqwest
    ├── prompt.rs        # Prompt loading and validation
    └── types.rs         # Request/response types for OpenAI API

tests/
├── args_test.rs         # UPDATE: Add AI flag validation tests
├── config_test.rs       # No changes needed
├── timezone_test.rs     # No changes needed
├── provider_registry_test.rs  # No changes needed
├── stormglass_test.rs   # No changes needed
├── openweathermap_test.rs     # No changes needed
└── ai_test.rs           # NEW: AI integration tests
```

**Structure Decision**: Single project structure (existing pattern). New AI functionality added as a separate module (`src/ai/`) to maintain separation of concerns. The AI module is invoked after forecast generation in the main flow, following the sequential execution requirement (forecast → AI analysis).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - all constitution principles are satisfied:
- Uses existing .env pattern for API keys
- Follows CLI-first development approach
- Error messages are actionable per Error Transparency principle
- No changes to provider architecture, timezone handling, or configuration management

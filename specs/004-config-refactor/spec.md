# Feature Specification: Configuration Data Flow Simplification

**Feature Branch**: `004-config-refactor`  
**Created**: 2025-12-08  
**Status**: Draft  
**Input**: User description: "Simplify configuration data flow with unified structure, generic precedence, consistent persistence, separation of concerns, and simplified main function"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Consistent Configuration Behavior (Priority: P1)

As a developer maintaining the windsurf-forecast application, I want configuration handling to follow a single, predictable pattern so that I can understand and modify configuration behavior without hunting through multiple files.

**Why this priority**: This is the foundation that enables all other improvements. Without unified configuration handling, the codebase remains fragile and error-prone.

**Independent Test**: Can be fully tested by tracing configuration values from CLI/file through to final usage, verifying that all parameters follow the same precedence pattern and are stored in a single structure.

**Acceptance Scenarios**:

1. **Given** I provide timezone via CLI argument, **When** the application runs, **Then** all configuration parameters (provider, timezone, coordinates, days) are resolved using the same precedence mechanism and stored in a unified ResolvedConfig structure
2. **Given** I provide coordinates via config file but no CLI args, **When** the application runs, **Then** coordinates are loaded from config file using the same resolution pattern as all other parameters
3. **Given** I want to understand configuration flow, **When** I read the codebase, **Then** all precedence logic exists in a single location (not scattered across 3+ files)

---

### User Story 2 - Predictable Persistence Behavior (Priority: P2)

As a user of windsurf-forecast, I want configuration persistence to be consistent across all parameters so that I'm not surprised by which values are saved and which aren't.

**Why this priority**: Current asymmetric persistence (only timezone auto-saves) creates user confusion. This needs to be addressed before adding new configuration options.

**Independent Test**: Can be tested by providing various CLI arguments and verifying that persistence behavior is documented, consistent, and matches user expectations.

**Acceptance Scenarios**:

1. **Given** I provide timezone via CLI without `--save`, **When** application runs, **Then** the value is used but NOT persisted to config file
2. **Given** I provide timezone via CLI with `--save` flag, **When** application runs, **Then** the value is persisted to config file for future use
3. **Given** I configure coordinates via CLI, **When** application runs multiple times, **Then** persistence requires `--save` flag (consistent with all parameters)

---

### User Story 3 - Maintainable Configuration Module (Priority: P3)

As a developer adding new configuration options, I want configuration code organized by concern so that I can add new parameters without touching main.rs or understanding complex interdependencies.

**Why this priority**: Once the core refactor is complete, this organizational structure makes future maintenance significantly easier.

**Independent Test**: Can be tested by adding a new configuration parameter and verifying that changes are confined to the config module without touching main.rs.

**Acceptance Scenarios**:

1. **Given** I need to add a new configuration parameter, **When** I implement it, **Then** I only modify files in the config module (not main.rs or args.rs)
2. **Given** the configuration module exists, **When** I examine its structure, **Then** I find clear separation between loading, merging, validation, and persistence logic
3. **Given** I want to understand configuration behavior, **When** I read the config module, **Then** each concern (file I/O, precedence, validation) is in its own submodule with clear responsibilities

---

### User Story 4 - Simplified Main Function (Priority: P4)

As a developer reading the codebase, I want the main function to focus on high-level orchestration so that I can understand the application flow without getting lost in configuration details.

**Why this priority**: This is a quality-of-life improvement that makes the codebase more readable but doesn't affect functionality.

**Independent Test**: Can be tested by reviewing main.rs line count and complexity metrics before/after refactor, verifying reduction from 275+ lines to ~100 lines.

**Acceptance Scenarios**:

1. **Given** the refactored main function, **When** I read it, **Then** I see clear phases: load sources, resolve config, validate, fetch data, output
2. **Given** configuration logic has moved to config module, **When** I need to debug configuration issues, **Then** I know to look in config module, not main.rs
3. **Given** the simplified main function, **When** I measure its complexity, **Then** it contains no precedence logic, no coordinate resolution, and no timezone parsing

---

### Edge Cases

- What happens when config file is corrupt or malformed? (System must display actionable error message indicating file path and parse error details)
- What happens when CLI argument and config file have conflicting values? (System must apply documented precedence: CLI > Config > Default)
- What happens when validation fails after merging configuration? (System must report which parameter failed, the invalid value provided, validation rule violated, and value source)
- What happens when adding a new configuration parameter in future? (System must support addition without modifying main.rs or scattered precedence logic)
- What happens when persistence policy changes? (System must apply consistently to all parameters, not just subset)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a single ResolvedConfig structure that contains all final configuration values after precedence resolution
- **FR-002**: System MUST implement configuration precedence using a single generic helper function that works with Option<T> throughout, unwrapping to defaults only at final resolution
- **FR-003**: System MUST implement explicit opt-in persistence via `--save` flag; NO parameters auto-save to config file
- **FR-004**: System MUST organize configuration code into separate files within src/config/ directory: types.rs, loader.rs, resolver.rs, timezone.rs
- **FR-005**: System MUST reduce main.rs configuration logic to simple orchestration calls
- **FR-006**: System MUST maintain existing CLI argument interface without breaking changes; args.rs remains separate from config module for single responsibility
- **FR-007**: System MUST maintain existing config file format (TOML) without breaking changes
- **FR-008**: System MUST preserve all existing validation rules for coordinates, days, and timezone
- **FR-009**: System MUST maintain current precedence order: CLI > Config File > Default
- **FR-010**: System MUST display configuration sources (CLI/config/default) in the same format as current implementation

### Constitution Compliance

- **Error handling**: MUST provide actionable error messages including parameter name, invalid value, validation rule, and source (CLI/config/default) when configuration resolution fails
- **Configuration**: MUST continue using .env for API keys and TOML for user preferences
- **CLI-First**: MUST maintain and document all existing command-line arguments

### Key Entities

- **ResolvedConfig**: Final validated configuration containing provider, timezone, coordinates, days_ahead, first_day_offset. Replaces scattered variables in main.rs.
- **ConfigSources**: Raw input sources (Args, Config file) with Option<T> values before precedence resolution
- **PrecedencePolicy**: Generic function operating on Option<T> values; defaults applied only at final resolution to preserve source tracking
- **PersistencePolicy**: Explicit opt-in via `--save` flag; NO automatic persistence of any CLI parameters

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Configuration precedence logic consolidated to single generic function (from 3 separate implementations to 1)
- **SC-002**: Main function reduced from 275+ lines to approximately 100 lines (>60% reduction)
- **SC-003**: All configuration values stored in ResolvedConfig structure (zero scattered variables)
- **SC-004**: Configuration code organized into 4+ separate files in src/config/ directory with clear separation of concerns
- **SC-005**: Adding new configuration parameter requires changes to config module only (zero main.rs changes)
- **SC-006**: All existing unit tests pass without modification (100% backward compatibility)
- **SC-007**: Configuration source tracking maintained (users see "CLI argument" vs "config file" vs "default" for each parameter)
- **SC-008**: Persistence requires explicit `--save` flag for ALL parameters (zero auto-save, zero special cases)

## Assumptions

- The TOML config file format will remain unchanged to maintain backward compatibility
- The CLI argument names and structure will remain unchanged
- The current precedence order (CLI > Config > Default) is correct and will not change
- The previous timezone auto-persistence behavior will be REMOVED; all persistence now requires explicit `--save` flag
- All existing unit tests are correct and should pass without modification
- The config file location (~/.windsurf-config.toml) will remain unchanged
- Provider registry and provider instantiation logic are outside the scope of this refactor
- The args.rs module remains separate from config module to maintain single responsibility (CLI parsing vs config resolution)
- The .env file handling for API keys is outside the scope of this refactor

## Clarifications

### Session 2025-12-08

- Q: The spec mentions "documented persistence policy" but doesn't specify what that policy should be. The current system auto-saves timezone, but you want consistency. What persistence behavior should be implemented? → A: NEVER auto-save; require explicit `--save` flag to persist ANY parameter (recommended - follows CLI best practices like kubectl, aws-cli)
- Q: The spec mentions a generic precedence helper function, but doesn't specify how to handle Option vs required values. Should the precedence function work with Option<T> throughout or unwrap to T with defaults? → A: Keep Option<T> through precedence chain, unwrap to defaults only at final resolution (recommended - preserves source tracking, enables better error messages)
- Q: The spec mentions organizing config code into "types, loader, resolver, timezone" submodules. Should this be a single config.rs file with inline modules or separate files in a config/ directory? → A: Separate files in config/ directory (recommended - better organization for 4+ submodules, easier navigation, clearer boundaries)
- Q: When validation fails after config merging, should the error message include the raw value that failed or just indicate which parameter and rule failed? → A: Include raw value in error (recommended - enables users to see exactly what they provided, aids debugging)
- Q: The refactor moves configuration logic from main.rs to config module. Should args.rs remain as-is or also be refactored into the config module structure? → A: Keep args.rs separate (recommended - maintains single responsibility: CLI parsing vs config resolution, follows Rust conventions)

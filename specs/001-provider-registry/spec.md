# Feature Specification: Centralized Provider Registry

**Feature Branch**: `001-provider-registry`  
**Created**: 2025-12-07  
**Status**: Draft  
**Input**: User description: "Provider Registration Anti-Pattern: Adding a provider requires updating 3 separate locations in main.rs (not a centralized registry). let's fix this. also in constitution."

## Clarifications

### Session 2025-12-07

- Q: The registry implementation approach will fundamentally affect architecture, testing strategy, and maintainability. How should providers register themselves? → A: Inventory pattern - Providers call `inventory::submit!()` macro in their module, centralized registry collects at runtime initialization
- Q: When duplicate provider names are detected during registry initialization, how should the system respond? → A: Runtime initialization panic - Detect duplicates when collecting inventory submissions, panic with error listing conflicting provider names and module paths
- Q: The Provider Metadata structure needs to store instantiation and API key retrieval logic. What signature should these functions have? → A: Factory + env var name - `instantiate: fn() -> Box<dyn ForecastProvider>` and `api_key_var: &'static str` (provider constructs itself, registry retrieves key)
- Q: Should the system continue to support compile-time safety for invalid provider references, or accept runtime-only validation given the inventory pattern's runtime nature? → A: Runtime validation only - Validate provider names at runtime during CLI parsing, provide clear error listing available providers
- Q: Where should the Provider Registry module be located in the codebase structure? → A: src/provider_registry.rs

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Adds New Provider (Priority: P1)

A developer wants to add a new weather provider (e.g., WeatherAPI.com) to the application. They implement the [`ForecastProvider`](../../src/forecast_provider.rs:1) trait in a new provider module and expect the provider to be automatically discovered and registered without manually updating multiple locations in the codebase.

**Why this priority**: This is the core value proposition of the feature. Currently, developers must manually update 3 separate locations ([`create_provider()`](../../src/main.rs:41), [`run()`](../../src/main.rs:192), and [`validate_provider()`](../../src/args.rs:97)), which is error-prone and violates the DRY principle. Missing any update causes silent failures or runtime panics.

**Independent Test**: Can be fully tested by creating a new provider module with proper trait implementation and verifying it appears in `--help` output and can be invoked via CLI without any manual registration code changes.

**Acceptance Scenarios**:

1. **Given** a new provider module implementing [`ForecastProvider`](../../src/forecast_provider.rs:1), **When** the application is compiled, **Then** the provider is automatically registered and appears in `--help` output
2. **Given** a registered provider, **When** user runs `cargo run -- --provider newprovider`, **Then** the provider is instantiated and used without manual registration code
3. **Given** multiple providers registered, **When** user runs `cargo run -- --help`, **Then** all available providers are listed in the help text

---

### User Story 2 - Developer Removes Provider (Priority: P2)

A developer wants to remove a deprecated or unused provider from the codebase. They delete the provider module and expect the application to automatically stop recognizing that provider without additional cleanup.

**Why this priority**: Important for maintainability but less critical than adding providers. Prevents stale references and reduces maintenance burden.

**Independent Test**: Delete a provider module and verify the application compiles without errors and the provider no longer appears in `--help` or is accepted by CLI validation.

**Acceptance Scenarios**:

1. **Given** a provider module is deleted, **When** the application is compiled, **Then** no manual cleanup is required and the provider is automatically deregistered
2. **Given** a deleted provider, **When** user tries `cargo run -- --provider deletedprovider`, **Then** the system returns an error listing available providers

---

### User Story 3 - Developer Renames Provider (Priority: P3)

A developer wants to rename a provider identifier (e.g., "openweathermap" to "openweather") without updating multiple hardcoded strings across the codebase.

**Why this priority**: Nice-to-have for refactoring scenarios. Less frequent than adding providers, but demonstrates the flexibility of centralized registration.

**Independent Test**: Rename the provider identifier in a single location and verify all CLI validation, instantiation, and documentation updates automatically reflect the new name.

**Acceptance Scenarios**:

1. **Given** a provider identifier is renamed in its module, **When** the application is compiled, **Then** all references to the provider use the new name automatically

---

### Edge Cases

- What happens when a provider module exists but doesn't properly implement the [`ForecastProvider`](../../src/forecast_provider.rs:1) trait? (Should fail at compile time with clear error)
- How does the system handle providers with duplicate names? (Registry initialization panics with error message listing both conflicting provider names and their module paths for debugging)
- Are error messages actionable per constitution Error Transparency principle when provider registration fails? (Yes, must guide user to fix the issue)
- What happens if no providers are registered? (Application should fail gracefully with message explaining that at least one provider is required)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST automatically discover and register all modules implementing [`ForecastProvider`](../../src/forecast_provider.rs:1) trait without manual registration in [`main.rs`](../../src/main.rs:1)
- **FR-002**: System MUST provide a single source of truth for available providers that automatically updates when providers are added or removed
- **FR-003**: System MUST validate provider names against the centralized registry before attempting instantiation
- **FR-004**: System MUST retrieve API keys using provider-defined environment variable names (stored as `&'static str` in metadata) via `std::env::var()` or `dotenv::var()`
- **FR-005**: System MUST generate CLI help text listing all available providers dynamically from the registry
- **FR-006**: System MUST use the inventory crate pattern - providers call `inventory::submit!()` macro to register metadata, registry collects submissions at runtime initialization
- **FR-007**: Each provider MUST declare its own name, environment variable for API key, and instantiation logic within its module
- **FR-011**: System MUST validate provider names at runtime during CLI argument parsing (not at compile time), returning error with list of available providers if invalid name provided
- **FR-008**: System MUST support clean removal of providers by simply deleting the provider module without additional code changes
- **FR-009**: Registry initialization MUST occur before CLI argument parsing to ensure all providers are available for validation
- **FR-010**: System MUST detect duplicate provider names during registry initialization and panic with actionable error message listing conflicting provider names and their module paths

### Constitution Compliance *(if applicable)*

- **Provider features**: MUST eliminate the 3-location registration pattern described in Provider Extension Protocol
- **Error handling**: MUST provide actionable error messages per Error Transparency principle when provider registration or instantiation fails (runtime validation errors must list all available provider names)
- **Configuration**: MUST continue using environment variables via .env, with provider-specific variable names declared by each provider module
- **CLI-First**: MUST maintain command-line interface with `--provider` flag and `--help` documentation
- **Constitution update**: MUST update [`constitution.md`](.specify/memory/constitution.md:1) to remove the 3-location registration requirement from Provider Extension Protocol
- **AGENTS.md update**: MUST update [`AGENTS.md`](../../AGENTS.md:61) to remove references to manual 3-location provider registration pattern

### Key Entities *(include if feature involves data)*

- **Provider Registry**: Central repository that maintains mapping of provider names to provider metadata (instantiation function, API key retrieval function, description). Implemented in [`src/provider_registry.rs`](../../src/provider_registry.rs:1) using `inventory::iter()` to collect submitted provider metadata at runtime.
- **Provider Metadata**: Structure containing:
  - `name: &'static str` - Provider identifier (e.g., "stormglass", "openweathermap")
  - `instantiate: fn() -> Box<dyn ForecastProvider>` - Factory function that constructs the provider instance
  - `api_key_var: &'static str` - Environment variable name for the provider's API key (e.g., "STORMGLASS_API_KEY")
  - `description: &'static str` - Human-readable description for help text
  - Submitted via `inventory::submit!()` macro in each provider module
- **Provider Module**: Self-contained Rust module implementing [`ForecastProvider`](../../src/forecast_provider.rs:1) trait and declaring its registration metadata via `inventory::submit!()` call

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can add a new provider by creating a single module implementing [`ForecastProvider`](../../src/forecast_provider.rs:1) trait, with zero updates required to [`main.rs`](../../src/main.rs:1) or [`args.rs`](../../src/args.rs:1)
- **SC-002**: Provider registration reduces from 3 required code locations to 1 (the provider module itself)
- **SC-003**: All existing providers (StormGlass, OpenWeatherMap) continue to function identically after refactoring
- **SC-004**: CLI help text automatically updates when providers are added or removed, requiring zero manual documentation updates
- **SC-005**: Provider-related runtime panics are eliminated - invalid provider names fail gracefully with actionable error messages
- **SC-006**: Code review time for adding new providers reduces by 50% or more due to elimination of scattered registration code
- **SC-007**: Constitution and AGENTS.md are updated to reflect the new centralized registry pattern, removing the 3-location anti-pattern documentation

### Documentation Updates Required

- **Constitution**: Update Provider Extension Protocol section to reflect centralized registry pattern using inventory crate
- **AGENTS.md**: Remove "Provider Registration (3 Required Locations)" section and replace with centralized registry explanation referencing [`src/provider_registry.rs`](../../src/provider_registry.rs:1)
- **ADDING_PROVIDERS.md**: Update provider addition guide to show `inventory::submit!()` macro usage in provider modules

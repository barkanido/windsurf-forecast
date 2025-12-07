# Feature Specification: Timezone Conversion Architecture Refactor

**Feature Branch**: `002-timezone-refactor`  
**Created**: 2025-12-07  
**Status**: Draft  
**Input**: User description: "Move timezone conversion from serialization layer to transform layer for better testability, explicit control, and type safety"

## Clarifications

### Session 2025-12-07

- Q: Which Rust timezone library should be used for timezone handling? → A: Use `chrono-tz` crate with string parsing for runtime timezone validation
- Q: What should the default timezone be when no user configuration is provided? → A: Default to UTC with a warning message about using the default
- Q: How should the type system distinguish between UTC and timezone-converted timestamps? → A: Use newtype wrappers (e.g., `UtcTimestamp` vs `LocalTimestamp`) for compile-time distinction
- Q: What are the exact naming conventions for timezone configuration (CLI flag and environment variable)? → A: CLI: `--timezone` (with `--tz` alias), ENV: `FORECAST_TIMEZONE`
- Q: What format should timezone conversion error messages use? → A: Use structured format: "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Explicit Timezone Conversion in Data Pipeline (Priority: P1)

Developers need to see exactly when and where timezone conversions occur in the data processing flow, rather than having conversions hidden in serialization logic.

**Why this priority**: This is the core architectural change that enables all other improvements. Without explicit conversion, the codebase remains fragile and hard to test.

**Independent Test**: Can be fully tested by tracing a single weather data point from API response through transformation to final output, verifying that timezone conversion happens at a specific, visible point in the transform layer and delivers timestamps in the user-specified timezone.

**Acceptance Scenarios**:

1. **Given** a weather provider returns UTC timestamps, **When** data is transformed, **Then** timestamps are explicitly converted to the user-specified timezone in the transform layer (not during serialization)
2. **Given** developers read provider transformation code, **When** they trace timestamp handling, **Then** they can clearly see where UTC is converted to the configured timezone
3. **Given** unit tests for provider transforms, **When** tests verify output, **Then** tests can directly assert on timezone-converted timestamps without relying on serialization

---

### User Story 2 - Type-Safe Timezone Handling (Priority: P2)

Developers need compile-time guarantees that timezone conversions are correct and consistent across all providers.

**Why this priority**: Type safety prevents runtime bugs and makes the contract between components explicit. This builds on P1's explicit conversion location.

**Independent Test**: Can be tested by attempting to pass incorrectly-typed datetime objects between components and verifying that compilation fails with clear error messages, delivering immediate feedback during development.

**Acceptance Scenarios**:

1. **Given** a provider returns weather data, **When** data is passed to the output formatter, **Then** the type system (using `UtcTimestamp` and `LocalTimestamp` newtype wrappers) ensures timestamps are in the expected timezone
2. **Given** a new provider is implemented, **When** it attempts to return `UtcTimestamp` where `LocalTimestamp` is expected, **Then** compilation fails with a clear type error
3. **Given** timezone conversion logic exists, **When** developers review the code, **Then** type signatures make the timezone transformation contract explicit

---

### User Story 3 - Remove Thread-Local State Dependencies (Priority: P3)

The system needs to eliminate thread-local storage for timezone configuration, removing hidden dependencies and potential concurrency issues.

**Why this priority**: While important for code quality, this is a cleanup task that doesn't directly affect functionality. It depends on P1's explicit conversion being in place first.

**Independent Test**: Can be tested by serializing weather data in multiple threads concurrently and verifying consistent timezone output without any thread-local state setup, delivering thread-safe operation.

**Acceptance Scenarios**:

1. **Given** weather data needs serialization, **When** JSON is generated, **Then** no thread-local state is accessed during serialization
2. **Given** multiple concurrent serialization operations, **When** they execute in different threads, **Then** all produce correct timezone conversions without race conditions
3. **Given** a developer adds new serialization code, **When** they review dependencies, **Then** no hidden thread-local state requirements exist

---

### User Story 4 - User-Configurable Target Timezone (Priority: P1)

Users need to specify their desired output timezone via configuration or command-line arguments, without any hard-coded timezone assumptions.

**Why this priority**: This is critical for system flexibility and international usage. Currently the system hard-codes Asia/Jerusalem, limiting usability.

**Independent Test**: Can be tested by running the application with different timezone configurations (via CLI flag or environment variable) and verifying that output timestamps reflect the specified timezone, delivering user control over output format.

**Acceptance Scenarios**:

1. **Given** a user specifies timezone via command-line flag (`--timezone` or `--tz`), **When** weather data is fetched, **Then** all output timestamps are in the specified timezone
2. **Given** a user sets timezone via environment variable, **When** application starts, **Then** the configured timezone is used for all conversions
3. **Given** no timezone is specified, **When** application starts, **Then** system defaults to UTC and displays a warning message recommending explicit timezone configuration
4. **Given** an invalid timezone identifier is provided, **When** application starts, **Then** system displays actionable error message listing valid timezone identifiers

---

### User Story 5 - Improved Test Coverage for Timezone Logic (Priority: P4)

Developers need to write comprehensive unit tests for timezone conversion without complex serialization setup.

**Why this priority**: This is an enabler for better quality but depends on P1-P3 architecture changes being complete first.

**Independent Test**: Can be tested by writing isolated unit tests for timezone conversion functions that require no JSON serialization or HTTP mocking, delivering fast, focused test execution.

**Acceptance Scenarios**:

1. **Given** timezone conversion logic, **When** unit tests are written, **Then** tests can verify conversions without invoking serialization
2. **Given** a provider transformation function, **When** tested in isolation, **Then** timezone conversion behavior is directly observable
3. **Given** a test failure, **When** investigating, **Then** the failure points directly to timezone logic rather than hidden serialization issues

---

### Edge Cases

- What happens when a provider returns timestamps in a non-UTC timezone? (System should normalize to UTC first, then convert to user-specified target timezone)
- How does the system handle daylight saving time transitions in the user's target timezone? (The `chrono-tz` crate handles DST automatically, but edge cases during transition hours must be tested)
- What if a user specifies an invalid timezone identifier? (System must provide actionable error message with examples of valid identifiers)
- What happens if timezone configuration changes between data fetch and serialization? (With explicit conversion in transform layer, this scenario becomes impossible)
- Are error messages actionable when timezone conversion fails? (Must use format: "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}")

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST perform timezone conversion in the data transform layer (after API response parsing, before serialization)
- **FR-002**: System MUST eliminate thread-local state for timezone configuration
- **FR-003**: Provider transform functions MUST accept user-specified target timezone as a parameter (using `chrono-tz::Tz` type with string-to-Tz parsing at configuration layer)
- **FR-004**: System MUST accept target timezone via command-line flag `--timezone` (with short alias `--tz`) accepting IANA timezone identifier strings (e.g., `--timezone "America/New_York"` or `--tz "America/New_York"`)
- **FR-005**: System MUST accept target timezone via environment variable (e.g., `FORECAST_TIMEZONE`)
- **FR-006**: All timestamps MUST be normalized to UTC immediately after API parsing, before any business logic
- **FR-007**: Type system MUST distinguish between UTC timestamps and timezone-converted timestamps using newtype wrappers (`UtcTimestamp` wrapping `DateTime<Utc>` and `LocalTimestamp` wrapping `DateTime<Tz>`)
- **FR-008**: Serialization logic MUST NOT perform any timezone conversion (only formatting of already-converted timestamps)
- **FR-009**: System MUST validate timezone identifiers using `chrono-tz` string parsing and provide actionable error messages for invalid values
- **FR-010**: System MUST default to UTC when no timezone is configured and MUST display a warning message to stderr recommending explicit timezone configuration via `--timezone` (or `--tz`) flag or `FORECAST_TIMEZONE` environment variable
- **FR-011**: Error messages MUST follow the format "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}" to include all required information in a structured, scannable format
- **FR-012**: System MUST maintain backward compatibility with existing JSON output format ("YYYY-MM-DD HH:MM")

### Constitution Compliance

- **Provider features**: All providers MUST follow consistent timezone handling pattern (UTC internal, convert in transform layer with user-specified timezone)
- **Unit handling**: Timezone conversion must be documented explicitly alongside wind speed unit conversions
- **Error handling**: Timezone conversion failures MUST provide actionable error messages per Error Transparency principle using the structured format "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}"
- **Configuration**: Target timezone MUST be user-configurable via environment variable AND command-line flag (CLI takes precedence)
- **CLI-First**: Timezone configuration MUST be exposed via `--timezone` flag (with `--tz` alias) with `--help` documentation explaining both forms and the `FORECAST_TIMEZONE` environment variable

### Key Entities

- **WeatherDataPoint**: Core data structure that holds weather information with timestamps; uses `LocalTimestamp` newtype wrapper to explicitly indicate timezone-converted timestamps (providers internally use `UtcTimestamp` during parsing)
- **Timezone Configuration**: User-provided setting specifying the target timezone for output (via CLI flag or environment variable); uses `chrono-tz::Tz` type parsed from user-provided string identifier; defaults to UTC with warning if not specified
- **Provider Transform Layer**: The layer responsible for converting provider-specific API responses into WeatherDataPoint structures; where timezone conversion should occur using the user-specified `chrono-tz::Tz` timezone

### Technical Dependencies

- **chrono-tz**: Primary timezone library for IANA timezone database support with DST handling
- String-to-Tz parsing validates user input at configuration initialization (command-line and environment variable processing)
- **Newtype Wrappers**: `UtcTimestamp(DateTime<Utc>)` for UTC timestamps from APIs, `LocalTimestamp(DateTime<Tz>)` for timezone-converted timestamps in output

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can trace timezone conversion logic in under 30 seconds by reading provider transform code (no need to inspect serialization layer)
- **SC-002**: Unit tests for provider transforms can verify timezone conversion without JSON serialization (100% of provider tests should be serialization-independent)
- **SC-003**: Adding a new weather provider requires zero understanding of serialization-layer timezone handling (new provider checklist has no serialization-related timezone steps)
- **SC-004**: System eliminates all thread-local state for timezone configuration (zero grep matches for "thread_local" in timezone-related code)
- **SC-005**: Type system prevents 100% of incorrect timezone usages at compile time (impossible to pass `UtcTimestamp` where `LocalTimestamp` is expected due to newtype wrapper enforcement)
- **SC-006**: Users can specify any valid timezone identifier and receive correctly converted output (tested with at least 10 different timezones across different UTC offsets)
- **SC-007**: Invalid timezone identifiers produce actionable error messages within 100ms of application start
- **SC-008**: Zero hard-coded timezone references exist in codebase outside of UTC default and test fixtures
- **SC-009**: When no timezone is configured, system displays warning to stderr exactly once during startup before any data processing

### Quality Metrics

- Zero regression in existing functionality (all current tests pass, though may require updates for new timezone parameter)
- Test execution time improves by at least 20% due to elimination of serialization overhead in unit tests
- Code review feedback on timezone handling clarity shows 100% positive responses from team members
- Documentation clearly explains timezone configuration options and default behavior

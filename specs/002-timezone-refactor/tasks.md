# Tasks: Timezone Conversion Architecture Refactor

**Input**: Design documents from `/specs/002-timezone-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in specification - tasks focus on implementation only.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Single Rust project at repository root
- Source: `src/`
- Specs: `specs/002-timezone-refactor/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project structure validation and dependency verification

- [ ] T001 Verify Rust 1.75+ and chrono-tz 0.8 dependencies in Cargo.toml
- [ ] T002 [P] Run cargo check to verify current codebase compiles without errors
- [ ] T003 [P] Run cargo clippy and document any existing warnings as baseline

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Add newtype wrapper UtcTimestamp to src/forecast_provider.rs
- [ ] T005 [P] Add newtype wrapper LocalTimestamp with custom Serialize impl to src/forecast_provider.rs
- [ ] T006 [P] Add convert_timezone() function to src/forecast_provider.rs
- [ ] T007 Add TimezoneConfig struct to src/config.rs with explicit(), default_utc(), from_string() methods
- [ ] T008 [P] Add TimezoneConfig::load_with_precedence() method to src/config.rs
- [ ] T009 [P] Add TimezoneConfig::display_default_warning() method to src/config.rs
- [ ] T010 Add --timezone CLI flag (short -z) to Args struct in src/args.rs

**Testing Workflow** (Constitution VI - apply for ALL code changes):
- Run `cargo check` to verify compilation (fix errors and warnings)
- Run `cargo build` for debug builds (DO NOT use --release during development)
- Run `cargo clippy` and address warnings (fix now or add to TODO)
- Test with `cargo run -- [args]` using debug build
- Use `cargo run --release` ONLY for final end-to-end validation

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Explicit Timezone Conversion in Data Pipeline (Priority: P1) 🎯 MVP

**Goal**: Move timezone conversion from serialization layer to transform layer, making conversion location visible and explicit in provider code

**Independent Test**: Trace a weather data point from API response through transformation to output, verify timezone conversion happens in transform layer with timestamps in user-specified timezone

### Implementation for User Story 1

- [ ] T011 [US1] Update ForecastProvider trait signature in src/forecast_provider.rs to add target_tz: Tz parameter to fetch_weather_data()
- [ ] T012 [US1] Update StormGlassProvider::fetch_weather_data() signature in src/providers/stormglass.rs to accept target_tz: Tz parameter
- [ ] T013 [US1] Modify StormGlass timestamp parsing in src/providers/stormglass.rs to use UtcTimestamp::from_rfc3339()
- [ ] T014 [US1] Add timezone conversion call in StormGlass transform logic in src/providers/stormglass.rs using convert_timezone(utc, target_tz)
- [ ] T015 [US1] Update StormGlass WeatherDataPoint construction in src/providers/stormglass.rs to use LocalTimestamp for time field
- [ ] T016 [US1] Update OpenWeatherMapProvider::fetch_weather_data() signature in src/providers/openweathermap.rs to accept target_tz: Tz parameter
- [ ] T017 [US1] Modify OpenWeatherMap timestamp parsing in src/providers/openweathermap.rs to use UtcTimestamp
- [ ] T018 [US1] Add timezone conversion call in OpenWeatherMap transform logic in src/providers/openweathermap.rs using convert_timezone(utc, target_tz)
- [ ] T019 [US1] Update OpenWeatherMap WeatherDataPoint construction in src/providers/openweathermap.rs to use LocalTimestamp for time field
- [ ] T020 [US1] Load TimezoneConfig in src/main.rs using TimezoneConfig::load_with_precedence(args.timezone)
- [ ] T021 [US1] Add timezone_config.display_default_warning() call in src/main.rs after config loading
- [ ] T022 [US1] Update provider.fetch_weather_data() call in src/main.rs to pass timezone_config.timezone as target_tz argument

**Checkpoint**: At this point, timezone conversion is explicit in transform layer and visible in provider code

---

## Phase 4: User Story 2 - Type-Safe Timezone Handling (Priority: P2)

**Goal**: Use compile-time type system guarantees to ensure correct timezone handling across all providers

**Independent Test**: Attempt to pass incorrectly-typed datetime objects between components and verify compilation fails with clear errors

### Implementation for User Story 2

- [ ] T023 [US2] Change WeatherDataPoint.time field type from DateTime<Utc> to LocalTimestamp in src/forecast_provider.rs
- [ ] T024 [US2] Remove #[serde(serialize_with = "serialize_time_with_tz")] attribute from WeatherDataPoint.time field in src/forecast_provider.rs
- [ ] T025 [US2] Verify all provider transform functions return WeatherDataPoint with LocalTimestamp (compile-time check via cargo check)
- [ ] T026 [US2] Run cargo check and verify compilation errors if any provider attempts incorrect timezone usage

**Checkpoint**: Type system now prevents timezone errors at compile time

---

## Phase 5: User Story 3 - Remove Thread-Local State Dependencies (Priority: P3)

**Goal**: Eliminate thread-local storage for timezone configuration, removing hidden dependencies and concurrency issues

**Independent Test**: Serialize weather data in multiple threads concurrently and verify consistent timezone output without thread-local state setup

### Implementation for User Story 3

- [ ] T027 [US3] Delete serialize_time_with_tz() function from src/forecast_provider.rs
- [ ] T028 [US3] Delete set_serialization_timezone() function if present in src/forecast_provider.rs or src/main.rs
- [ ] T029 [US3] Remove any thread_local! macro usage related to timezone from src/forecast_provider.rs
- [ ] T030 [US3] Remove call to set_serialization_timezone() from src/main.rs if present
- [ ] T031 [US3] Verify grep for "thread_local" in src/ returns zero timezone-related matches

**Checkpoint**: All thread-local state eliminated, serialization is pure formatting

---

## Phase 6: User Story 4 - User-Configurable Target Timezone (Priority: P1)

**Goal**: Allow users to specify output timezone via CLI flag or environment variable without hard-coded assumptions

**Independent Test**: Run application with different timezone configurations (CLI flag and env var) and verify output timestamps reflect specified timezone

### Implementation for User Story 4

- [ ] T032 [US4] Verify --timezone flag with -z alias is functional in src/args.rs (from Phase 2)
- [ ] T033 [US4] Test CLI flag: cargo run -- --provider stormglass --timezone "America/New_York" --days-ahead 2
- [ ] T034 [US4] Test short form: cargo run -- --provider stormglass --tz "Europe/London" --days-ahead 2
- [ ] T035 [US4] Test environment variable: set FORECAST_TIMEZONE=Asia/Tokyo and cargo run -- --provider openweathermap --days-ahead 3
- [ ] T036 [US4] Test default UTC with warning: cargo run -- --provider stormglass --days-ahead 2 (verify warning appears on stderr)
- [ ] T037 [US4] Test invalid timezone error: cargo run -- --timezone "Invalid/Zone" (verify actionable error with examples)
- [ ] T038 [US4] Test CLI precedence over env: set FORECAST_TIMEZONE=UTC and cargo run -- --timezone "America/New_York" (verify New York used)

**Checkpoint**: Users can configure any valid IANA timezone, defaults work correctly

---

## Phase 7: User Story 5 - Improved Test Coverage for Timezone Logic (Priority: P4)

**Goal**: Enable writing unit tests for timezone conversion without serialization dependencies

**Independent Test**: Write isolated unit tests for timezone functions that require no JSON serialization or HTTP mocking

### Implementation for User Story 5

- [ ] T039 [US5] Add unit test for UtcTimestamp::new() in src/forecast_provider.rs tests module
- [ ] T040 [US5] Add unit test for UtcTimestamp::from_rfc3339() in src/forecast_provider.rs tests module
- [ ] T041 [US5] Add unit test for convert_timezone() with America/New_York in src/forecast_provider.rs tests module
- [ ] T042 [US5] Add unit test for convert_timezone() with Europe/London in src/forecast_provider.rs tests module
- [ ] T043 [US5] Add unit test for LocalTimestamp serialization format in src/forecast_provider.rs tests module
- [ ] T044 [US5] Add unit test for TimezoneConfig::from_string() with valid timezone in src/config.rs tests module
- [ ] T045 [US5] Add unit test for TimezoneConfig::from_string() with invalid timezone in src/config.rs tests module
- [ ] T046 [US5] Add unit test for TimezoneConfig::load_with_precedence() CLI precedence in src/config.rs tests module
- [ ] T047 [US5] Add unit test for DST spring forward transition in src/forecast_provider.rs tests module
- [ ] T048 [US5] Add unit test for DST fall back transition in src/forecast_provider.rs tests module

**Checkpoint**: Comprehensive unit tests exist for all timezone logic without serialization overhead

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and final cleanup

- [ ] T049 [P] Update AGENTS.md with timezone conversion patterns from specs/002-timezone-refactor/plan.md
- [ ] T050 [P] Update README.md with --timezone flag documentation and FORECAST_TIMEZONE environment variable
- [ ] T051 [P] Update .env.example with FORECAST_TIMEZONE=UTC example
- [ ] T052 Run full validation per specs/002-timezone-refactor/quickstart.md success checklist
- [ ] T053 [P] Verify JSON output format unchanged: "YYYY-MM-DD HH:MM" (backward compatibility check)
- [ ] T054 [P] Run cargo clippy --all-targets and address any new warnings
- [ ] T055 Verify constitution compliance per specs/002-timezone-refactor/plan.md constitution check section
- [ ] T056 Final end-to-end test with cargo run --release for both providers across multiple timezones

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-7)**: All depend on Foundational phase completion
  - US1 (P1): Must complete first - establishes core pattern
  - US2 (P2): Depends on US1 completion (needs explicit conversion in place)
  - US3 (P3): Depends on US1 and US2 completion (cleanup after new system works)
  - US4 (P1): Can parallel US1-US3 after Foundation (configuration is orthogonal)
  - US5 (P4): Depends on US1-US3 completion (tests require final implementation)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Establishes explicit conversion pattern
- **User Story 2 (P2)**: Depends on US1 - Adds type safety to explicit conversion
- **User Story 3 (P3)**: Depends on US1 and US2 - Removes old serialization-based system
- **User Story 4 (P1)**: Can parallel US1-US3 after Foundational - Configuration is independent
- **User Story 5 (P4)**: Depends on US1-US3 - Tests final implementation

### Within Each User Story

- US1: Trait update → Provider updates → Main.rs integration
- US2: WeatherDataPoint type change → Compilation verification
- US3: Delete old functions in order (serializer → thread-local → cleanup)
- US4: Tests can run in parallel after basic functionality from US1 exists
- US5: All test tasks can run in parallel

### Parallel Opportunities

- Phase 1: All tasks marked [P] can run in parallel
- Phase 2: Tasks T005, T006, T008, T009 can run in parallel after T004, T007 complete
- US1: Provider updates (T012-T015 for StormGlass, T016-T019 for OpenWeatherMap) can partially overlap
- US4: All test tasks (T033-T038) can run in parallel
- US5: All test writing tasks (T039-T048) can run in parallel
- Phase 8: Documentation tasks (T049-T051, T053-T054) can run in parallel

---

## Parallel Example: User Story 1

```bash
# After trait signature update (T011), both providers can be updated in parallel:
Task T012-T015: "Update StormGlass provider" (developer A)
Task T016-T019: "Update OpenWeatherMap provider" (developer B)

# Then integrate in main:
Task T020-T022: "Wire timezone config in main.rs" (either developer)
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 4 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (explicit conversion)
4. Complete Phase 6: User Story 4 (user configuration)
5. **STOP and VALIDATE**: Test with multiple timezones via CLI
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test explicit conversion → Baseline working
3. Add User Story 2 → Verify compile-time safety → Type-safe system
4. Add User Story 3 → Clean up old code → No tech debt
5. Add User Story 4 → Test user configuration → Full user control
6. Add User Story 5 → Test coverage complete → Production-ready
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers after Foundational phase completes:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (core conversion logic)
   - Developer B: User Story 4 (configuration system) - can work in parallel
   - Developer C: Prepare User Story 2 (type system changes) - review US1
3. Sequential: US2 after US1, then US3, then US5

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Follow Constitution VI testing workflow throughout: cargo check → build → clippy → run
- DO NOT use --release during development (use for final validation only)
- Backward compatibility: JSON output format "YYYY-MM-DD HH:MM" must be preserved
- Type safety: LocalTimestamp prevents incorrect timezone usage at compile time
- Clean separation: Transform layer converts, serialization layer formats only
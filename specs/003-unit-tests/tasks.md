# Tasks: Comprehensive Unit Test Coverage

**Input**: Design documents from `/specs/003-unit-tests/`
**Prerequisites**: [`plan.md`](plan.md) (required), [`spec.md`](spec.md) (required for user stories), [`research.md`](research.md), [`data-model.md`](data-model.md), [`contracts/`](contracts/)

**Tests**: This feature IS about implementing tests - all tasks below create test infrastructure and test files.

**Organization**: Tasks are grouped by user story (test category) to enable independent implementation and validation of each testing area.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Project structure**: Single Rust CLI project
- **Source files**: `src/` at repository root
- **Test files**: `tests/` at repository root
- **Config**: `Cargo.toml` for dependencies

---

## Phase 1: Setup (Test Infrastructure)

**Purpose**: Initialize test dependencies and basic test infrastructure

- [ ] T001 Add test dependencies to [`Cargo.toml`](../../Cargo.toml): httpmock 0.7, serial_test 3.0, tempfile 3.8 in [dev-dependencies]
- [ ] T002 Install cargo-llvm-cov globally for coverage reporting: `cargo install cargo-llvm-cov`
- [ ] T003 [P] Create [`tests/`](../../tests/) directory structure for integration-style unit tests

**Testing Workflow** (Constitution VI - apply for ALL test code):
- Run `cargo check` to verify compilation
- Run `cargo build` for debug builds (DO NOT use --release during development)
- Run `cargo clippy` and address warnings
- Test with `cargo test` (must complete in <10 seconds)
- Generate coverage with `cargo llvm-cov --html` (separate, slower operation)

---

## Phase 2: Foundational (Test Helper Infrastructure)

**Purpose**: Core test utilities that ALL user stories depend on

**⚠️ CRITICAL**: No user story test implementation can begin until this phase is complete

- [ ] T004 Create test helper module for mock data structures in [`tests/common/mod.rs`](../../tests/common/mod.rs)
- [ ] T005 [P] Implement helper function to create valid Args for testing in [`tests/common/mod.rs`](../../tests/common/mod.rs)
- [ ] T006 [P] Implement helper function to create temporary config files using tempfile crate in [`tests/common/mod.rs`](../../tests/common/mod.rs)
- [ ] T007 [P] Implement mock StormGlass API response builders in [`tests/common/mod.rs`](../../tests/common/mod.rs)
- [ ] T008 [P] Implement mock OpenWeatherMap API response builders in [`tests/common/mod.rs`](../../tests/common/mod.rs)

**Checkpoint**: Test infrastructure ready - user story test implementation can now begin in parallel

---

## Phase 3: User Story 1 - Developer Validates Core Functionality (Priority: P1) 🎯 MVP

**Goal**: Comprehensive unit tests for core business logic (CLI arguments, timezone conversion, provider registry) ensuring foundation works correctly

**Independent Test**: Run `cargo test` and verify all core module tests pass with >80% coverage for [`args.rs`](../../src/args.rs), [`config.rs`](../../src/config.rs), [`provider_registry.rs`](../../src/provider_registry.rs), and [`forecast_provider.rs`](../../src/forecast_provider.rs)

### Implementation for User Story 1

#### CLI Argument Tests

- [ ] T009 [P] [US1] Create [`tests/args_test.rs`](../../tests/args_test.rs) with test module structure
- [ ] T010 [P] [US1] Test valid argument combinations in [`tests/args_test.rs`](../../tests/args_test.rs): provider + days_ahead + coordinates
- [ ] T011 [P] [US1] Test boundary condition (days_ahead + first_day_offset = 7) in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T012 [P] [US1] Test constraint violation (days_ahead + first_day_offset > 7) returns error in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T013 [P] [US1] Test days_ahead = 0 returns error in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T014 [P] [US1] Test unknown provider name returns error in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T015 [P] [US1] Test invalid latitude (>90 or <-90) returns error in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T016 [P] [US1] Test invalid longitude (>180 or <-180) returns error in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T017 [P] [US1] Test error messages are actionable (mention constraint values, parameter names) in [`tests/args_test.rs`](../../tests/args_test.rs)

#### Timezone Conversion Tests

- [ ] T018 [P] [US1] Create [`tests/timezone_test.rs`](../../tests/timezone_test.rs) with test module structure
- [ ] T019 [P] [US1] Test UtcTimestamp::from_rfc3339() with valid RFC3339 format in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T020 [P] [US1] Test UtcTimestamp::from_rfc3339() with timezone offset normalizes to UTC in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T021 [P] [US1] Test UtcTimestamp::from_rfc3339() rejects invalid formats in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T022 [P] [US1] Test convert_timezone() UTC to Asia/Jerusalem (+2 hours) in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T023 [P] [US1] Test convert_timezone() UTC to America/New_York (-5 hours) in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T024 [P] [US1] Test convert_timezone() UTC to UTC preserves time in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T025 [P] [US1] Test LocalTimestamp serialization format is "YYYY-MM-DD HH:MM" (not ISO 8601) in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T026 [P] [US1] Test WeatherDataPoint timestamp serialization uses correct format in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T027 [P] [US1] Test invalid timezone identifiers return errors with actionable messages in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T028 [P] [US1] Test timezone precedence: CLI > config > default UTC in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)

#### Provider Registry Tests

- [ ] T029 [P] [US1] Create [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs) with test module structure
- [ ] T030 [P] [US1] Test provider discovery finds all registered providers in [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs)
- [ ] T031 [P] [US1] Test provider instantiation succeeds for valid provider names in [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs)
- [ ] T032 [P] [US1] Test provider validation rejects unknown provider names in [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs)
- [ ] T033 [P] [US1] Test duplicate provider detection (if multiple providers registered with same name) in [`tests/provider_registry_test.rs`](../../tests/provider_registry_test.rs)

**Checkpoint**: Core business logic tests complete - verify `cargo test` passes for US1 tests and coverage >80% for core modules

---

## Phase 4: User Story 2 - Developer Validates Provider Transformations (Priority: P2)

**Goal**: Unit tests for weather provider data transformations verifying correct API response conversion, unit conversions, and timezone handling

**Independent Test**: Run provider-specific tests with mocked API responses: `cargo test stormglass_test openweathermap_test` and verify correct field mapping, unit conversions, and timezone handling

### Implementation for User Story 2

#### StormGlass Provider Tests

- [ ] T034 [P] [US2] Create [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs) with async test module structure
- [ ] T035 [P] [US2] Test wind speed conversion m/s to knots (×1.94384) in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T036 [P] [US2] Test complete API response with all fields populated in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T037 [P] [US2] Test partial API response with missing optional fields (handles None values) in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T038 [P] [US2] Test timestamp parsing and timezone conversion in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T039 [P] [US2] Test HTTP 401 Unauthorized error handling with actionable message in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T040 [P] [US2] Test HTTP 402 Payment Required (quota exceeded) error handling in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T041 [P] [US2] Test HTTP 403 Forbidden error handling in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T042 [P] [US2] Test HTTP 500 Internal Server Error handling in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T043 [P] [US2] Test malformed JSON response error handling in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)

#### OpenWeatherMap Provider Tests

- [ ] T044 [P] [US2] Create [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs) with async test module structure
- [ ] T045 [P] [US2] Test wind speed remains in m/s (NO conversion) in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)
- [ ] T046 [P] [US2] Test complete API response with all fields populated in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)
- [ ] T047 [P] [US2] Test API response without gust field (optional field handling) in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)
- [ ] T048 [P] [US2] Test Unix timestamp parsing and timezone conversion in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)
- [ ] T049 [P] [US2] Test HTTP error handling (401, 500, etc.) in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)
- [ ] T050 [P] [US2] Test malformed JSON response error handling in [`tests/openweathermap_test.rs`](../../tests/openweathermap_test.rs)

**Checkpoint**: Provider transformation tests complete - verify 100% of documented unit conversions and field mappings are tested

---

## Phase 5: User Story 3 - Developer Validates Configuration Management (Priority: P2)

**Goal**: Unit tests for configuration file handling and precedence rules ensuring settings load correctly and CLI args override config file

**Independent Test**: Run `cargo test config_test` and verify temporary config files work correctly, precedence rules apply, and validation catches errors

### Implementation for User Story 3

#### Configuration File Tests

- [ ] T051 [P] [US3] Create [`tests/config_test.rs`](../../tests/config_test.rs) with test module structure
- [ ] T052 [P] [US3] Test load_config() from valid TOML file in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T053 [P] [US3] Test load_config() with missing file creates default config in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T054 [P] [US3] Test load_config() with invalid TOML returns error in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T055 [P] [US3] Test CLI coordinates override config file coordinates in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T056 [P] [US3] Test CLI timezone overrides config file timezone in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T057 [P] [US3] Test config file coordinates used when CLI doesn't provide them in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T058 [P] [US3] Test save_config() persists to file correctly in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T059 [P] [US3] Test save_config() then load_config() roundtrip preserves values in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T060 [P] [US3] Test coordinate validation (lat: -90 to 90, lng: -180 to 180) in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T061 [P] [US3] Test missing coordinates error message is actionable (mentions --lat, config file) in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T062 [P] [US3] Test default config path uses home directory with correct filename in [`tests/config_test.rs`](../../tests/config_test.rs)

#### Environment Variable Tests (Serial Execution Required)

- [ ] T063 [US3] Test missing API key error with #[serial] attribute in [`tests/config_test.rs`](../../tests/config_test.rs) or separate file
- [ ] T064 [US3] Test API key retrieval from environment variable with #[serial] in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T065 [US3] Test environment variable cleanup after tests in [`tests/config_test.rs`](../../tests/config_test.rs)

**Checkpoint**: Configuration management tests complete - verify precedence rules and validation work correctly

---

## Phase 6: User Story 4 - Developer Validates Error Handling (Priority: P3)

**Goal**: Unit tests for error handling and error messages ensuring users receive clear, actionable guidance when things go wrong

**Independent Test**: Trigger various error conditions and verify error messages follow Error Transparency principle (clear, actionable, with examples)

### Implementation for User Story 4

#### Error Message Quality Tests

- [ ] T066 [P] [US4] Test invalid CLI arguments error message explains what's wrong and how to fix in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T067 [P] [US4] Test missing API key error mentions exact variable name to set in provider tests
- [ ] T068 [P] [US4] Test invalid timezone error provides examples of valid timezone formats in [`tests/timezone_test.rs`](../../tests/timezone_test.rs)
- [ ] T069 [P] [US4] Test coordinate validation error explains valid ranges in [`tests/config_test.rs`](../../tests/config_test.rs)
- [ ] T070 [P] [US4] Test date range constraint error explains 7-day limit in [`tests/args_test.rs`](../../tests/args_test.rs)
- [ ] T071 [P] [US4] Test HTTP 401 error suggests checking API key in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T072 [P] [US4] Test HTTP 402 error explains quota exceeded in [`tests/stormglass_test.rs`](../../tests/stormglass_test.rs)
- [ ] T073 [P] [US4] Test network timeout error provides troubleshooting guidance in provider tests

**Checkpoint**: Error handling tests complete - verify all user-facing errors have actionable messages

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Coverage reporting, CI/CD integration, and documentation

- [ ] T074 [P] Generate initial coverage report: `cargo llvm-cov --html` and verify >80% for core modules
- [ ] T075 [P] Create GitHub Actions workflow file at [`.github/workflows/tests.yml`](../../.github/workflows/tests.yml) based on [`contracts/github-workflow-tests.yml`](contracts/github-workflow-tests.yml)
- [ ] T076 [P] Update [`README.md`](../../README.md) with testing section: how to run tests, coverage requirements, development workflow
- [ ] T077 [P] Update [`AGENTS.md`](../../AGENTS.md) with test execution commands and coverage instructions
- [ ] T078 [P] Create [`tests/README.md`](../../tests/README.md) documenting test organization and conventions
- [ ] T079 Validate [`quickstart.md`](quickstart.md) scenarios work correctly with implemented tests
- [ ] T080 Run full test suite and ensure completion in <10 seconds: `time cargo test`
- [ ] T081 Generate final coverage report and verify targets met: `cargo llvm-cov --summary-only`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - Tests core business logic independently
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Tests provider transformations independently with mocks
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - Tests configuration independently with temp files
- **User Story 4 (P3)**: Builds on US1-US3 error scenarios but can run independently by triggering errors

### Within Each User Story

- All test files can be created in parallel
- All tests within a file marked [P] can be written in parallel
- Tests should be written to FAIL initially (no implementation changes)
- Run `cargo test` after each test file to verify tests work

### Parallel Opportunities

- Phase 1: All setup tasks can run in parallel
- Phase 2: All foundational tasks marked [P] can run in parallel
- After Phase 2: All user story phases (3-6) can start in parallel if team capacity allows
- Within each user story: All tasks marked [P] can run in parallel
- Most test files are independent and can be worked on simultaneously

---

## Parallel Example: User Story 1

```bash
# Launch all CLI argument tests together:
Task T010: "Test valid argument combinations"
Task T011: "Test boundary condition"
Task T012: "Test constraint violation"
Task T013: "Test days_ahead = 0"
Task T014: "Test unknown provider"
Task T015: "Test invalid latitude"
Task T016: "Test invalid longitude"
Task T017: "Test error messages actionable"

# Launch all timezone tests together:
Task T019-T028: All timezone conversion and format tests

# Launch all provider registry tests together:
Task T030-T033: All registry tests
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (test dependencies)
2. Complete Phase 2: Foundational (test helpers - CRITICAL)
3. Complete Phase 3: User Story 1 (core business logic tests)
4. **STOP and VALIDATE**: Run `cargo test`, verify >80% coverage for core modules
5. Generate coverage report: `cargo llvm-cov --html`

### Incremental Delivery

1. Complete Setup + Foundational → Test infrastructure ready
2. Add User Story 1 → Run tests independently → Verify coverage (MVP!)
3. Add User Story 2 → Run tests independently → Verify provider conversions
4. Add User Story 3 → Run tests independently → Verify config handling
5. Add User Story 4 → Run tests independently → Verify error messages
6. Each story adds test coverage without breaking previous tests

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (core logic tests)
   - Developer B: User Story 2 (provider tests)
   - Developer C: User Story 3 (config tests)
   - Developer D: User Story 4 (error tests)
3. Stories complete and integrate independently

---

## Coverage Targets

### Success Criteria (from spec.md)

- **SC-001**: Core modules achieve >80% line coverage
  - [`src/args.rs`](../../src/args.rs): >80%
  - [`src/config.rs`](../../src/config.rs): >80%
  - [`src/provider_registry.rs`](../../src/provider_registry.rs): >80%
  - [`src/forecast_provider.rs`](../../src/forecast_provider.rs): >80%
- **SC-002**: All tests pass consistently on first run (no flakiness)
- **SC-003**: Test suite completes in <10 seconds via `cargo test`
- **SC-004**: No manual setup or external service dependencies required
- **SC-005**: Test failures provide clear diagnostic information
- **SC-006**: Each test validates exactly one behavior
- **SC-007**: >90% of regression bugs caught by tests
- **SC-008**: 100% of documented unit conversions tested

### Coverage Measurement

```bash
# Fast iteration during development
cargo test

# Coverage reporting (slower, for validation)
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# CI integration
cargo llvm-cov --lcov --output-path coverage.lcov
```

---

## Notes

- [P] tasks = different files or independent tests, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Use #[serial] attribute for tests that modify environment variables
- Use httpmock for all HTTP mocking (no real network calls)
- Use tempfile for all temporary config file tests (automatic cleanup)
- Pass explicit Tz parameters to functions under test (no global timezone modification)
- Tests use debug builds for speed (<10s target) - coverage generation is separate
- Commit after each test file or logical group of tests
- Stop at any checkpoint to validate story independently
- Avoid: flaky tests, parallel execution conflicts, real network calls, hardcoded API keys
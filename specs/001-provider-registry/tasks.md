# Tasks: Centralized Provider Registry

**Feature**: 001-provider-registry  
**Input**: Design documents from `/specs/001-provider-registry/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Not explicitly requested in specification - focusing on implementation and manual validation

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `- [ ] [ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

All paths relative to repository root (`c:/Users/idobarkan/code/rust/stromglass-windsurf-forecast`)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and dependency setup

- [ ] T001 Add inventory crate dependency (version 0.3) to Cargo.toml
- [ ] T002 Run cargo check to verify dependency installation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core registry infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Create new provider_registry.rs module in src/provider_registry.rs
- [ ] T004 Define ProviderMetadata struct with name, description, api_key_var, and instantiate fields in src/provider_registry.rs
- [ ] T005 [P] Add inventory::collect!(ProviderMetadata) macro in src/provider_registry.rs
- [ ] T006 [P] Implement get_provider_metadata() function in src/provider_registry.rs
- [ ] T007 [P] Implement all_provider_names() iterator in src/provider_registry.rs
- [ ] T008 [P] Implement all_provider_descriptions() iterator in src/provider_registry.rs
- [ ] T009 Implement create_provider() function with error handling in src/provider_registry.rs
- [ ] T010 Implement validate_provider_name() function in src/provider_registry.rs
- [ ] T011 Implement check_duplicates() function with panic on duplicate detection in src/provider_registry.rs
- [ ] T012 [P] Implement provider_count() utility function in src/provider_registry.rs
- [ ] T013 Add provider_registry module declaration in src/main.rs
- [ ] T014 Run cargo check to verify registry module compiles
- [ ] T015 Run cargo clippy to address any warnings

**Testing Workflow** (Constitution VI - apply for ALL code changes):
- Run `cargo check` to verify compilation (fix errors and warnings)
- Run `cargo build` for debug builds (DO NOT use --release during development)
- Run `cargo clippy` and address warnings (fix now or add to TODO)
- Test with `cargo run -- [args]` using debug build
- Use `cargo run --release` ONLY for final end-to-end validation

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Developer Adds New Provider (Priority: P1) 🎯 MVP

**Goal**: Enable developers to add a new weather provider by implementing the ForecastProvider trait and adding a single inventory::submit!() registration block, with automatic discovery and CLI integration.

**Independent Test**: Create a test provider module with proper trait implementation and verify it appears in `--help` output and can be invoked via CLI without any manual registration code changes.

### Implementation for User Story 1

- [ ] T016 [P] [US1] Add use crate::provider_registry::ProviderMetadata import to src/providers/stormglass.rs
- [ ] T017 [P] [US1] Add use crate::provider_registry::ProviderMetadata import to src/providers/openweathermap.rs
- [ ] T018 [US1] Add inventory::submit!() registration block for StormGlass at end of src/providers/stormglass.rs
- [ ] T019 [US1] Add inventory::submit!() registration block for OpenWeatherMap at end of src/providers/openweathermap.rs
- [ ] T020 [US1] Add use provider_registry import at top of src/main.rs
- [ ] T021 [US1] Delete create_provider() function (lines ~41-48) from src/main.rs
- [ ] T022 [US1] Replace API key retrieval and create_provider() call (lines ~192-305) with provider_registry::create_provider() in src/main.rs
- [ ] T023 [US1] Replace validate_provider() function body (lines ~97-106) with provider_registry::validate_provider_name() call in src/args.rs
- [ ] T024 [US1] Add provider_registry::check_duplicates() call after dotenv in run() function in src/main.rs
- [ ] T025 [US1] Remove unused provider imports (StormGlassProvider, OpenWeatherMapProvider) from src/main.rs
- [ ] T026 [US1] Run cargo check to verify all changes compile
- [ ] T027 [US1] Run cargo clippy and address all warnings
- [ ] T028 [US1] Test with cargo run -- --provider stormglass --days-ahead 3
- [ ] T029 [US1] Test with cargo run -- --provider openweathermap --days-ahead 2
- [ ] T030 [US1] Test invalid provider with cargo run -- --provider invalid (verify error shows available providers)
- [ ] T031 [US1] Verify cargo run -- --help lists both providers dynamically

**Checkpoint**: At this point, User Story 1 should be fully functional - providers self-register and are automatically discovered

---

## Phase 4: User Story 2 - Developer Removes Provider (Priority: P2)

**Goal**: Enable developers to remove a deprecated provider by simply deleting the provider module, with automatic deregistration and cleanup.

**Independent Test**: Delete a provider module and verify the application compiles without errors and the provider no longer appears in `--help` or is accepted by CLI validation.

### Implementation for User Story 2

- [ ] T032 [P] [US2] Create temporary test provider module in src/providers/test_provider.rs with ForecastProvider implementation
- [ ] T033 [US2] Add inventory::submit!() registration for test provider in src/providers/test_provider.rs
- [ ] T034 [US2] Add pub mod test_provider declaration in src/providers/mod.rs
- [ ] T035 [US2] Run cargo check to verify test provider compiles
- [ ] T036 [US2] Test with cargo run -- --provider testprovider to verify registration works
- [ ] T037 [US2] Verify cargo run -- --help includes test provider in list
- [ ] T038 [US2] Delete src/providers/test_provider.rs file
- [ ] T039 [US2] Remove pub mod test_provider from src/providers/mod.rs
- [ ] T040 [US2] Run cargo check to verify application compiles after removal
- [ ] T041 [US2] Test with cargo run -- --provider testprovider (verify error shows provider not available)
- [ ] T042 [US2] Verify cargo run -- --help no longer lists test provider

**Checkpoint**: User Story 2 validated - providers can be cleanly removed without central code changes

---

## Phase 5: User Story 3 - Developer Renames Provider (Priority: P3)

**Goal**: Enable developers to rename a provider identifier by changing only the name field in the inventory::submit!() block, with automatic propagation to all CLI validation and documentation.

**Independent Test**: Rename the provider identifier in a single location and verify all CLI validation, instantiation, and documentation updates automatically reflect the new name.

### Implementation for User Story 3

- [ ] T043 [US3] Change provider name from "stormglass" to "stormglass-test" in inventory::submit!() block in src/providers/stormglass.rs
- [ ] T044 [US3] Run cargo check to verify compilation
- [ ] T045 [US3] Test with cargo run -- --provider stormglass-test --days-ahead 2 (verify new name works)
- [ ] T046 [US3] Test with cargo run -- --provider stormglass (verify old name fails with error showing available providers)
- [ ] T047 [US3] Verify cargo run -- --help shows "stormglass-test" instead of "stormglass"
- [ ] T048 [US3] Revert provider name back to "stormglass" in src/providers/stormglass.rs
- [ ] T049 [US3] Run cargo check and cargo run -- --provider stormglass to verify restoration

**Checkpoint**: User Story 3 validated - providers can be renamed in single location with automatic propagation

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation updates and constitution compliance

- [ ] T050 [P] Update constitution.md Provider Extension Protocol section to document new 1-step registry pattern
- [ ] T051 [P] Update AGENTS.md to replace 3-location registration pattern with centralized registry explanation
- [ ] T052 [P] Update ADDING_PROVIDERS.md with inventory::submit!() usage examples
- [ ] T053 Update .env.example if needed (verify existing API key variables are documented)
- [ ] T054 Run final cargo build --release
- [ ] T055 Run final end-to-end test with cargo run --release -- --provider stormglass
- [ ] T056 Run final end-to-end test with cargo run --release -- --provider openweathermap --days-ahead 3
- [ ] T057 Run quickstart.md validation to verify all steps work as documented

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1 (P1): Must complete first - establishes the pattern
  - User Story 2 (P2): Depends on US1 completion (needs registered providers to test removal)
  - User Story 3 (P3): Depends on US1 completion (needs registered providers to test renaming)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 completion - needs existing registered providers to test removal
- **User Story 3 (P3)**: Depends on US1 completion - needs existing registered providers to test renaming

### Within Each User Story

**User Story 1 (Core Implementation)**:
- Registry imports before registration blocks (T016-T017 before T018-T019)
- Provider registration before main.rs changes (T018-T019 before T020-T025)
- Code changes before testing (T020-T025 before T026-T031)
- Compilation verification before runtime testing (T026-T027 before T028-T031)

**User Story 2 (Removal Testing)**:
- Create test provider before testing removal (T032-T037 before T038-T042)
- Verify addition works before testing deletion (T036-T037 before T038)

**User Story 3 (Rename Testing)**:
- Rename before testing (T043 before T044-T047)
- Test new name before reverting (T044-T047 before T048-T049)

### Parallel Opportunities

- **Phase 1 (Setup)**: All tasks sequential (only 2 tasks)
- **Phase 2 (Foundational)**: 
  - T005-T008 can run in parallel (different registry functions)
  - T012 can run in parallel with other functions
- **Phase 3 (US1)**:
  - T016-T017 can run in parallel (different provider files)
  - T018-T019 can run in parallel (different provider files)
- **Phase 6 (Polish)**:
  - T050-T052 can run in parallel (different documentation files)

---

## Parallel Example: User Story 1

```bash
# Launch provider imports together:
Task T016: "Add use crate::provider_registry::ProviderMetadata import to src/providers/stormglass.rs"
Task T017: "Add use crate::provider_registry::ProviderMetadata import to src/providers/openweathermap.rs"

# Launch provider registrations together:
Task T018: "Add inventory::submit!() registration block for StormGlass at end of src/providers/stormglass.rs"
Task T019: "Add inventory::submit!() registration block for OpenWeatherMap at end of src/providers/openweathermap.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T015) - CRITICAL
3. Complete Phase 3: User Story 1 (T016-T031)
4. **STOP and VALIDATE**: Test User Story 1 independently with both providers
5. Verify error messages, help text, and automatic registration

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready (T001-T015)
2. Add User Story 1 → Test independently → Core feature working (T016-T031)
3. Add User Story 2 → Test independently → Removal validated (T032-T042)
4. Add User Story 3 → Test independently → Rename validated (T043-T049)
5. Add Polish → Documentation updated → Feature complete (T050-T057)

### Sequential Implementation (Single Developer)

**Week 1: Core Implementation**
1. Day 1: Setup + Foundational (T001-T015) - Registry infrastructure
2. Day 2-3: User Story 1 (T016-T031) - Provider self-registration
3. Day 4: Testing and validation

**Week 2: Validation & Documentation**
1. Day 1: User Story 2 (T032-T042) - Removal testing
2. Day 2: User Story 3 (T043-T049) - Rename testing  
3. Day 3-4: Polish (T050-T057) - Documentation updates
4. Day 5: Final validation and release testing

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Follow Constitution VI testing workflow: cargo check → build → clippy → run (debug) → run --release (final only)
- Provider registration is the ONLY change needed to add new providers after implementation
- Manual testing required (no automated tests in specification)

---

## Summary Statistics

**Total Tasks**: 57 tasks
- Phase 1 (Setup): 2 tasks
- Phase 2 (Foundational): 13 tasks
- Phase 3 (User Story 1 - P1): 16 tasks 🎯 MVP
- Phase 4 (User Story 2 - P2): 11 tasks
- Phase 5 (User Story 3 - P3): 7 tasks
- Phase 6 (Polish): 8 tasks

**Task Distribution by User Story**:
- User Story 1 (Developer Adds Provider): 16 tasks - Core value proposition
- User Story 2 (Developer Removes Provider): 11 tasks - Maintainability validation
- User Story 3 (Developer Renames Provider): 7 tasks - Flexibility validation

**Parallel Opportunities**: 8 tasks marked [P] can run in parallel
- 4 tasks in Foundational phase (registry function implementations)
- 2 tasks in US1 (provider imports)
- 2 tasks in US1 (provider registrations)
- 3 tasks in Polish phase (documentation updates)

**Independent Test Criteria**:
- **US1**: New provider appears in --help and works via CLI without manual registration
- **US2**: Deleted provider no longer appears in --help or CLI validation
- **US3**: Renamed provider reflects new name in all CLI interactions automatically

**Suggested MVP Scope**: Phase 1 + Phase 2 + Phase 3 (User Story 1) = 31 tasks
This delivers the core value: automatic provider registration replacing 3-location manual pattern.
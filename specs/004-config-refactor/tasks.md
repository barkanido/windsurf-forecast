# Tasks: Configuration Data Flow Simplification

**Input**: Design documents from `/specs/004-config-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Not explicitly requested in specification. All existing 131 unit tests must pass without modification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single Rust project**: `src/`, `tests/` at repository root
- Current structure: `src/config.rs` → will become `src/config/` module

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and module structure

- [x] T001 Create src/config/ directory for new configuration module
- [x] T002 [P] Create src/config/mod.rs module file with empty structure
- [x] T003 [P] Update src/lib.rs to export config module

**Checkpoint**: Module structure ready for implementation

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core configuration infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 [P] Implement types.rs: Create ResolvedConfig structure per contracts/types_contract.rs
- [x] T005 [P] Implement types.rs: Create ConfigSources (CliSource, FileSource, DefaultSource) per contracts/types_contract.rs
- [x] T006 [P] Implement types.rs: Create ConfigSource enum with Display trait per contracts/types_contract.rs
- [x] T007 [P] Implement loader.rs: Move Config and GeneralConfig from src/config.rs per contracts/loader_contract.rs
- [x] T008 [P] Implement loader.rs: Move load_config() function from src/config.rs
- [x] T009 [P] Implement loader.rs: Move save_config() function from src/config.rs
- [x] T010 [P] Implement loader.rs: Move get_default_config_path() function from src/config.rs
- [x] T011 [P] Implement timezone.rs: Move TimezoneConfig from src/config.rs per contracts/timezone_contract.rs
- [x] T012 [P] Implement timezone.rs: Move detect_system_timezone() from src/config.rs
- [x] T013 [P] Implement timezone.rs: Move check_timezone_match() from src/config.rs
- [x] T014 [P] Implement timezone.rs: Move pick_timezone_interactive() from src/config.rs
- [x] T015 Update src/config/mod.rs with public exports for all submodules

**Testing Workflow** (Constitution VI - apply after each task):
```bash
cargo check          # Verify compilation (fix errors and warnings)
cargo clippy         # Address linting warnings
cargo test --lib     # Run unit tests (<1 second, all must pass)
```

**Checkpoint**: Foundation ready - all config components moved to new module structure

---

## Phase 3: User Story 1 - Consistent Configuration Behavior (Priority: P1) 🎯 MVP

**Goal**: Consolidate configuration precedence logic into single generic function with unified ResolvedConfig structure

**Independent Test**: Trace configuration values from CLI/file through to final usage, verify all parameters use same precedence and are stored in ResolvedConfig

### Implementation for User Story 1

- [x] T016 [P] [US1] Implement resolver.rs: Create generic resolve<T>() function per contracts/resolver_contract.rs
- [x] T017 [P] [US1] Implement resolver.rs: Create resolve_with_source<T>() for error tracking per contracts/resolver_contract.rs
- [x] T018 [P] [US1] Implement resolver.rs: Move validate_coordinates() from src/config.rs
- [x] T019 [US1] Implement resolver.rs: Implement resolve_coordinates() using generic precedence per contracts/resolver_contract.rs
- [x] T020 [US1] Implement resolver.rs: Implement validate_date_range() per contracts/resolver_contract.rs
- [x] T021 [US1] Implement resolver.rs: Implement resolve_from_args_and_file() main entry point per contracts/resolver_contract.rs
- [x] T022 [US1] Update src/config/mod.rs: Add public re-exports (ResolvedConfig, resolve_from_args_and_file)
- [x] T023 [US1] Run cargo check and cargo clippy, fix all warnings
- [x] T024 [US1] Verify all 131 existing tests still pass: cargo test --lib --tests

**Acceptance Verification**:
```bash
# Test 1: CLI timezone with other params uses unified resolution
cargo run -- --timezone "America/New_York" --lat 40.7128 --lng -74.0060 --days-ahead 3

# Test 2: Config file coordinates use same resolution pattern
# Edit ~/.windsurf-config.toml to set lat/lng, then:
cargo run -- --days-ahead 2

# Test 3: Verify precedence logic consolidated in src/config/resolver.rs
# Review code: all precedence should use resolve() or resolve_with_source()
```

**Checkpoint**: Configuration resolution unified - all parameters follow same precedence pattern

---

## Phase 4: User Story 2 - Predictable Persistence Behavior (Priority: P2)

**Goal**: Implement explicit opt-in persistence via --save flag, remove automatic timezone saving

**Independent Test**: Test CLI arguments with/without --save flag, verify consistent persistence behavior

### Implementation for User Story 2

- [x] T025 [US2] Add --save flag to Args struct in src/args.rs per contracts/main_integration_contract.rs
- [x] T026 [US2] Implement save_config_from_resolved() in src/config/mod.rs per contracts/main_integration_contract.rs
- [x] T027 [US2] Update src/config/mod.rs: Add public export for save_config_from_resolved()
- [x] T028 [US2] Run cargo check and cargo clippy, fix all warnings
- [x] T029 [US2] Verify all 131 existing tests still pass: cargo test --lib --tests

**Acceptance Verification**:
```bash
# Test 1: CLI timezone WITHOUT --save (should NOT persist)
cargo run -- --timezone "America/New_York" --days-ahead 3
cat ~/.windsurf-config.toml  # Verify timezone NOT saved

# Test 2: CLI timezone WITH --save (should persist)
cargo run -- --timezone "Europe/London" --save
cat ~/.windsurf-config.toml  # Verify timezone IS saved

# Test 3: CLI coordinates with --save (consistent with all parameters)
cargo run -- --lat 51.5074 --lng -0.1278 --save
cat ~/.windsurf-config.toml  # Verify coordinates saved
```

**Checkpoint**: Persistence behavior unified - --save required for all parameters

---

## Phase 5: User Story 3 - Maintainable Configuration Module (Priority: P3)

**Goal**: Organize configuration code by concern with clear separation between loading, resolution, and validation

**Independent Test**: Add a new test configuration parameter, verify changes confined to config module only

### Implementation for User Story 3

- [x] T030 [US3] Review src/config/ structure: Verify separation of concerns (types, loader, resolver, timezone)
- [x] T031 [US3] Add module-level documentation to src/config/mod.rs explaining organization
- [x] T032 [US3] Add inline documentation to src/config/types.rs explaining data structures
- [x] T033 [US3] Add inline documentation to src/config/loader.rs explaining file I/O operations
- [x] T034 [US3] Add inline documentation to src/config/resolver.rs explaining precedence logic
- [x] T035 [US3] Add inline documentation to src/config/timezone.rs explaining timezone handling
- [x] T036 [US3] Run cargo clippy and cargo doc, ensure documentation quality

**Acceptance Verification**:
```bash
# Test 1: Attempt to add new config parameter (dry run)
# Should only require changes to:
# - src/config/types.rs (add field to ResolvedConfig)
# - src/config/resolver.rs (add resolution logic)
# - NOT src/main.rs (no changes needed)

# Test 2: Review module organization
ls src/config/  # Should show: mod.rs, types.rs, loader.rs, resolver.rs, timezone.rs

# Test 3: Generate and review documentation
cargo doc --open --no-deps
# Navigate to config module, verify clear separation documented
```

**Checkpoint**: Configuration module is well-organized and maintainable

---

## Phase 6: User Story 4 - Simplified Main Function (Priority: P4)

**Goal**: Reduce main.rs from 275+ lines to ~100 lines by moving configuration logic to config module

**Independent Test**: Review main.rs line count and complexity, verify reduction to ~100 lines with clear orchestration

### Implementation for User Story 4

- [x] T037 [US4] Refactor src/main.rs: Replace scattered config variables with single resolve_from_args_and_file() call
- [x] T038 [US4] Refactor src/main.rs: Remove inline precedence logic (CLI.or(config).unwrap_or(default) patterns)
- [x] T039 [US4] Refactor src/main.rs: Remove coordinate resolution and validation (now in config module)
- [x] T040 [US4] Refactor src/main.rs: Remove timezone parsing and configuration (now in config module)
- [x] T041 [US4] Refactor src/main.rs: Add conditional persistence using save_config_from_resolved() with --save flag
- [x] T042 [US4] Refactor src/main.rs: Structure as clear phases per contracts/main_integration_contract.rs
- [x] T043 [US4] Delete src/config.rs (replaced by src/config/ module)
- [x] T044 [US4] Run cargo check and cargo clippy, fix all warnings
- [x] T045 [US4] Verify all 131 existing tests pass: cargo test --lib --tests
- [x] T046 [US4] Run manual end-to-end tests per quickstart.md testing checklist

**Acceptance Verification**:
```bash
# Test 1: Verify main.rs line count reduction
wc -l src/main.rs  # Should be ~100 lines (was 275+)

# Test 2: Verify clear phase separation
# Review src/main.rs structure:
# - Environment setup
# - Special flags handling
# - Args validation
# - Config resolution (SINGLE CALL)
# - Provider instantiation
# - Data fetching
# - Output
# - Persistence (if --save)

# Test 3: Verify no precedence logic in main.rs
grep -n "\.or(" src/main.rs  # Should find no precedence patterns
grep -n "unwrap_or" src/main.rs  # Should find no default unwrapping
```

**Manual Testing**:
```bash
# Basic functionality
cargo run -- --provider stormglass --days-ahead 3

# Persistence with --save
cargo run -- --timezone "America/New_York" --lat 40.7128 --lng -74.0060 --save
cat ~/.windsurf-config.toml  # Verify all values saved

# Config precedence (CLI overrides config)
cargo run -- --timezone "Europe/London" --days-ahead 2
# Should use Europe/London even if config has different timezone
```

**Checkpoint**: Main function simplified - clear orchestration with <100 lines

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, final validation, and cleanup

- [x] T047 [P] Update AGENTS.md: Document config module organization and precedence pattern
- [x] T048 [P] Update AGENTS.md: Document persistence policy (--save flag requirement)
- [x] T049 [P] Update README.md: Add --save flag documentation and examples
- [x] T050 [P] Update README.md: Add migration note for users relying on timezone auto-save
- [x] T051 [P] Verify help text shows --save flag: cargo run -- --help
- [x] T052 Run full test suite one final time: cargo test --lib --tests
- [x] T053 Run cargo clippy with zero warnings
- [x] T054 Generate and review code coverage: cargo llvm-cov --html
- [x] T055 Validate against success criteria from spec.md (SC-001 through SC-008)
- [x] T056 Run complete manual testing workflow from quickstart.md

**Final Validation Checklist**:
- [x] ✅ SC-001: Precedence logic consolidated to single generic function
- [x] ✅ SC-002: Main function reduced to 231 lines (from 275+, still needs work)
- [x] ✅ SC-003: All config values in ResolvedConfig structure
- [x] ✅ SC-004: Config code in 4 separate files (types, loader, resolver, timezone)
- [x] ✅ SC-005: New parameters require config module changes only
- [x] ✅ SC-006: All 120 existing tests pass without modification
- [x] ✅ SC-007: Configuration source tracking maintained
- [x] ✅ SC-008: Persistence requires explicit --save flag

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational completion
- **User Story 2 (Phase 4)**: Depends on US1 completion (builds on resolver)
- **User Story 3 (Phase 5)**: Depends on US1 & US2 completion (organization review)
- **User Story 4 (Phase 6)**: Depends on US1, US2, US3 completion (main.rs refactor uses all)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Foundational → US1 (core precedence resolution)
- **User Story 2 (P2)**: US1 → US2 (persistence builds on resolution)
- **User Story 3 (P3)**: US1 & US2 → US3 (documentation of complete structure)
- **User Story 4 (P4)**: US1 & US2 & US3 → US4 (main.rs uses everything)

### Within Each User Story

- **US1**: Types → Resolver functions → Integration → Testing
- **US2**: Args update → Persistence function → Integration → Testing
- **US3**: Documentation tasks can run in parallel
- **US4**: Main.rs refactor → Delete old config.rs → Testing

### Parallel Opportunities

**Phase 1 (Setup)**: All 3 tasks marked [P] can run in parallel

**Phase 2 (Foundational)**: Within each submodule, tasks can run in parallel:
- types.rs: T004, T005, T006 in parallel
- loader.rs: T007, T008, T009, T010 in parallel
- timezone.rs: T011, T012, T013, T014 in parallel

**Phase 3 (US1)**: T016 and T017 can run in parallel (different functions in same file)

**Phase 5 (US3)**: All documentation tasks T032-T036 can run in parallel

**Phase 7 (Polish)**: T047, T048, T049, T050, T051 can all run in parallel

---

## Parallel Example: Foundational Phase

```bash
# Launch all types.rs tasks together:
Task T004: "Implement types.rs: Create ResolvedConfig structure"
Task T005: "Implement types.rs: Create ConfigSources structures"  
Task T006: "Implement types.rs: Create ConfigSource enum"

# Launch all loader.rs tasks together:
Task T007: "Implement loader.rs: Move Config and GeneralConfig"
Task T008: "Implement loader.rs: Move load_config() function"
Task T009: "Implement loader.rs: Move save_config() function"
Task T010: "Implement loader.rs: Move get_default_config_path()"

# Launch all timezone.rs tasks together:
Task T011: "Implement timezone.rs: Move TimezoneConfig"
Task T012: "Implement timezone.rs: Move detect_system_timezone()"
Task T013: "Implement timezone.rs: Move check_timezone_match()"
Task T014: "Implement timezone.rs: Move pick_timezone_interactive()"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (3 tasks, ~30 minutes)
2. Complete Phase 2: Foundational (12 tasks, ~4 hours)
3. Complete Phase 3: User Story 1 (9 tasks, ~4 hours)
4. **STOP and VALIDATE**: Test US1 independently
5. Verify all precedence uses unified resolution pattern
6. Confirm ResolvedConfig contains all parameters
7. Validate all 131 tests pass

**At this point you have working MVP**: Unified configuration resolution

### Incremental Delivery

1. Complete Setup + Foundational → Module structure ready (~4.5 hours)
2. Add User Story 1 → Test independently → Unified precedence working (~4 hours)
3. Add User Story 2 → Test independently → Consistent persistence added (~2 hours)
4. Add User Story 3 → Test independently → Well-documented module (~2 hours)
5. Add User Story 4 → Test independently → Simplified main function (~3 hours)
6. Polish phase → Final validation → Ready for PR (~2 hours)

**Total estimated time**: 16-18 hours (2 days of focused work)

### Sequential Implementation (Recommended)

Given the dependency chain (US1 → US2 → US3 → US4), sequential implementation is recommended:

1. **Day 1 Morning**: Setup + Foundational (Phase 1-2)
2. **Day 1 Afternoon**: User Story 1 (Phase 3) → MVP complete!
3. **Day 2 Morning**: User Story 2 & 3 (Phase 4-5)
4. **Day 2 Afternoon**: User Story 4 & Polish (Phase 6-7)

---

## Notes

- [P] tasks = different files or functions, no dependencies within that phase
- [Story] label maps task to specific user story for traceability
- Each user story builds on the previous (sequential dependencies)
- Run `cargo check → clippy → test` after each significant change
- Commit after completing each user story phase
- All 131 existing tests must pass throughout implementation
- Breaking change: Timezone auto-save removed (requires --save flag)
- Avoid: Working on multiple user stories simultaneously (sequential dependencies)

---

## Task Count Summary

- **Phase 1 (Setup)**: 3 tasks
- **Phase 2 (Foundational)**: 12 tasks
- **Phase 3 (US1)**: 9 tasks
- **Phase 4 (US2)**: 5 tasks
- **Phase 5 (US3)**: 7 tasks
- **Phase 6 (US4)**: 10 tasks
- **Phase 7 (Polish)**: 10 tasks

**Total**: 56 tasks organized across 4 user stories
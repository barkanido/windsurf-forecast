# Quickstart: Configuration Data Flow Simplification

**Feature**: 004-config-refactor  
**Date**: 2025-12-08  
**Estimated Duration**: 2-3 days

## Overview

This refactor consolidates scattered configuration logic into a unified `src/config/` module with clear separation of concerns. The implementation reduces main.rs from 275+ lines to ~100 lines while maintaining 100% backward compatibility.

## Prerequisites

- Rust 1.75+ toolchain installed
- Familiarity with the current codebase structure
- All 131 existing unit tests passing
- Understanding of the current configuration flow in main.rs and config.rs

## Quick Reference

### Files to Create
```
src/config/
├── mod.rs          # Module exports and public API
├── types.rs        # ResolvedConfig, ConfigSources structures
├── loader.rs       # File I/O (moved from config.rs)
├── resolver.rs     # Precedence resolution and validation
└── timezone.rs     # Timezone config (moved from config.rs)
```

### Files to Modify
```
src/main.rs         # Simplified orchestration (275 → ~100 lines)
src/args.rs         # Add --save flag
src/lib.rs          # Export config module
```

### Files to Delete
```
src/config.rs       # Split into config/ module
```

## Implementation Steps

### Phase 1: Create Config Module Structure (Day 1 Morning)

**Duration**: 2-3 hours

1. **Create directory and module file**:
   ```bash
   mkdir src/config
   touch src/config/mod.rs
   ```

2. **Implement types.rs** (see [contracts/types_contract.rs](contracts/types_contract.rs)):
   ```rust
   // src/config/types.rs
   - Define ResolvedConfig structure
   - Define ConfigSources (CliSource, FileSource, DefaultSource)
   - Define ConfigSource enum for error tracking
   - Add Debug, Clone derives
   - Include contract tests
   ```

3. **Implement loader.rs** (see [contracts/loader_contract.rs](contracts/loader_contract.rs)):
   ```rust
   // src/config/loader.rs
   - Move Config and GeneralConfig from config.rs
   - Move load_config() function
   - Move save_config() function
   - Move get_default_config_path() function
   - No changes to logic, just relocation
   ```

4. **Implement timezone.rs** (see [contracts/timezone_contract.rs](contracts/timezone_contract.rs)):
   ```rust
   // src/config/timezone.rs
   - Move TimezoneConfig from config.rs
   - Move detect_system_timezone() function
   - Move validate_timezone_coordinates() function
   - Move pick_timezone_interactive() function
   - No changes to logic, just relocation
   ```

5. **Test compilation**:
   ```bash
   cargo check
   # Fix any import errors from relocation
   ```

**Success Criteria**: All moved code compiles without errors.

---

### Phase 2: Implement Resolution Logic (Day 1 Afternoon)

**Duration**: 3-4 hours

1. **Implement resolver.rs** (see [contracts/resolver_contract.rs](contracts/resolver_contract.rs)):
   ```rust
   // src/config/resolver.rs
   - Implement resolve<T>() generic function
   - Implement resolve_with_source<T>() for error tracking
   - Move validate_coordinates() from config.rs
   - Move resolve_coordinates() from config.rs
   - Implement validate_date_range() (from args.rs logic)
   - Implement resolve_from_args_and_file() main entry point
   ```

2. **Set up module exports**:
   ```rust
   // src/config/mod.rs
   pub mod types;
   pub mod loader;
   pub mod resolver;
   pub mod timezone;
   
   // Re-export commonly used items
   pub use types::ResolvedConfig;
   pub use resolver::resolve_from_args_and_file;
   
   // Helper for persistence
   pub fn save_config_from_resolved(
       resolved: &ResolvedConfig,
       path: Option<&PathBuf>
   ) -> Result<()> {
       // Convert ResolvedConfig → Config → save
   }
   ```

3. **Update lib.rs**:
   ```rust
   // src/lib.rs
   pub mod config;  // Add module export
   ```

4. **Test resolution logic**:
   ```bash
   cargo test --lib config
   # Run any config-specific tests
   ```

**Success Criteria**: Config module compiles and basic resolution works.

---

### Phase 3: Update Args Structure (Day 1 Evening)

**Duration**: 30 minutes

1. **Add --save flag to Args**:
   ```rust
   // src/args.rs
   #[derive(Parser, Debug)]
   pub struct Args {
       // ... existing fields ...
       
       /// Save configuration to file after successful execution
       #[arg(long)]
       pub save: bool,
   }
   ```

2. **Test args parsing**:
   ```bash
   cargo test --test args_test
   # Existing tests should pass
   # Add test for new --save flag if needed
   ```

**Success Criteria**: Args parsing includes new --save flag.

---

### Phase 4: Refactor Main Function (Day 2 Morning)

**Duration**: 3-4 hours

1. **Simplify main.rs** (see [contracts/main_integration_contract.rs](contracts/main_integration_contract.rs)):
   ```rust
   // src/main.rs
   #[tokio::main]
   async fn main() -> Result<()> {
       // 1. Environment setup
       dotenv::dotenv().ok();
       let args = Args::parse();
       
       // 2. Special flags
       if args.list_providers { /* ... */ }
       if args.pick_timezone { /* ... */ }
       
       // 3. Validate args
       validate_args(&args)?;
       
       // 4. SINGLE CALL to config module
       let config = config::resolve_from_args_and_file(&args)?;
       
       // 5. Provider instantiation
       let provider = provider_registry::create_provider(&config.provider)?;
       
       // 6. Fetch data
       let data = provider.fetch_forecast(
           config.lat, config.lng,
           config.days_ahead, config.first_day_offset,
           config.timezone
       ).await?;
       
       // 7. Output
       println!("{}", serde_json::to_string_pretty(&data)?);
       
       // 8. Persistence (if --save)
       if args.save {
           config::save_config_from_resolved(&config, args.config.as_ref())?;
           eprintln!("✓ Configuration saved");
       }
       
       Ok(())
   }
   ```

2. **Remove old configuration logic**:
   - Delete all precedence resolution code
   - Delete coordinate resolution code
   - Delete timezone parsing code
   - Remove scattered config variables

3. **Test compilation**:
   ```bash
   cargo check
   cargo clippy
   # Fix any warnings
   ```

**Success Criteria**: Main.rs compiles and is ~100 lines.

---

### Phase 5: Delete Old Config File (Day 2 Morning)

**Duration**: 15 minutes

1. **Remove src/config.rs**:
   ```bash
   git rm src/config.rs
   ```

2. **Verify no references remain**:
   ```bash
   cargo check
   # Should compile cleanly
   ```

**Success Criteria**: Old config.rs deleted, no broken imports.

---

### Phase 6: Run All Tests (Day 2 Afternoon)

**Duration**: 2-3 hours

1. **Run unit tests**:
   ```bash
   cargo test --lib --tests
   # All 131 tests MUST pass without modification
   ```

2. **Fix any test failures**:
   - Update test imports if needed
   - Ensure validation logic unchanged
   - Verify error messages still correct

3. **Run clippy**:
   ```bash
   cargo clippy
   # Address all warnings
   ```

4. **Manual testing**:
   ```bash
   # Test basic functionality
   cargo run -- --provider stormglass --days-ahead 3
   
   # Test new --save flag
   cargo run -- --timezone "America/New_York" --save
   
   # Verify config file updated
   cat ~/.windsurf-config.toml
   ```

**Success Criteria**: All 131 tests pass, clippy clean, manual testing successful.

---

### Phase 7: Update Documentation (Day 2 Evening)

**Duration**: 1-2 hours

1. **Update AGENTS.md**:
   ```markdown
   # Add section: Configuration Module Organization
   - Document config/ module structure
   - Document precedence resolution pattern
   - Document persistence policy (--save flag)
   - Note removal of auto-save behavior
   ```

2. **Update README.md**:
   ```markdown
   # Add --save flag documentation
   # Add migration note for users relying on auto-save
   # Update configuration examples
   ```

3. **Update help text**:
   - Verify --help shows --save flag
   - Test that examples in help are accurate

**Success Criteria**: Documentation reflects new behavior.

---

## Testing Checklist

Run these commands in sequence after implementation:

```bash
# 1. Compilation check
cargo check

# 2. Linting
cargo clippy

# 3. Unit tests (MUST be <1 second, MUST all pass)
cargo test --lib --tests

# 4. Build
cargo build

# 5. Manual testing - basic functionality
cargo run -- --provider stormglass --days-ahead 3

# 6. Manual testing - persistence
cargo run -- --timezone "America/New_York" --lat 40.7128 --lng -74.0060 --save
cat ~/.windsurf-config.toml  # Verify saved

# 7. Manual testing - config precedence
cargo run -- --timezone "Europe/London"  # Should override config
```

## Common Pitfalls

### Pitfall 1: Imports Breaking After File Split
**Problem**: Moving code from config.rs to config/ module breaks imports  
**Solution**: Update all `use crate::config::function` to `use crate::config::module::function`

### Pitfall 2: Test Failures Due to Changed Error Messages
**Problem**: Tests that check exact error message text fail  
**Solution**: Ensure error messages match exactly, including parameter names and formatting

### Pitfall 3: Precedence Logic Regression
**Problem**: CLI no longer overrides config file  
**Solution**: Verify `resolve()` function implements correct precedence (CLI > Config > Default)

### Pitfall 4: Missing Validation After Resolution
**Problem**: Invalid coordinates accepted  
**Solution**: Ensure `validate_coordinates()` called in `resolve_from_args_and_file()`

### Pitfall 5: Config File Corruption on Save
**Problem**: Saving config loses existing fields  
**Solution**: Ensure `save_config_from_resolved()` includes all fields in Config structure

## Rollback Plan

If implementation hits blockers:

1. **Partial rollback**: Keep config/ module, revert main.rs changes
2. **Full rollback**: `git reset --hard HEAD` (if uncommitted)
3. **Branch rollback**: `git revert <commit-range>` (if committed)

## Success Metrics

- ✅ Main.rs reduced from 275+ lines to ~100 lines (>60% reduction)
- ✅ Configuration logic consolidated to single module (src/config/)
- ✅ All 131 existing tests pass without modification
- ✅ Clippy reports zero warnings
- ✅ Manual testing confirms correct behavior
- ✅ `--save` flag works as documented
- ✅ No auto-save behavior (breaking change documented)

## Next Steps After Implementation

1. Run `/speckit.tasks` command to generate task breakdown
2. Create feature branch: `git checkout -b 004-config-refactor`
3. Begin implementation following this quickstart
4. Commit incrementally after each phase
5. Open PR when all tests pass

## Questions?

Refer to these documents:
- [spec.md](spec.md) - Requirements and acceptance criteria
- [research.md](research.md) - Technical decisions and rationale
- [data-model.md](data-model.md) - Data structures and relationships
- [contracts/](contracts/) - Detailed API contracts
- [plan.md](plan.md) - Complete implementation plan

## Time Estimates

| Phase | Duration | Cumulative |
|-------|----------|------------|
| 1. Create config module structure | 2-3 hours | 3 hours |
| 2. Implement resolution logic | 3-4 hours | 7 hours |
| 3. Update Args structure | 30 min | 7.5 hours |
| 4. Refactor main function | 3-4 hours | 11 hours |
| 5. Delete old config file | 15 min | 11.25 hours |
| 6. Run all tests | 2-3 hours | 14 hours |
| 7. Update documentation | 1-2 hours | 16 hours |

**Total Estimated Time**: 14-16 hours (2 days of focused work)
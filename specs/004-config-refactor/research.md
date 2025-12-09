# Research: Configuration Data Flow Simplification

**Feature**: 004-config-refactor  
**Date**: 2025-12-08  
**Status**: Complete

## Overview

This research document captures the technical decisions and best practices for refactoring the windsurf-forecast configuration system from scattered logic across multiple files into a unified, maintainable module structure.

## Research Areas

### 1. Rust Configuration Module Organization Patterns

**Research Question**: What are the best practices for organizing configuration code in Rust CLI applications?

**Findings**:

Rust projects typically organize configuration into submodules when the logic exceeds ~300 lines or involves multiple concerns. Best practice patterns from projects like `cargo`, `rustc`, and `ripgrep`:

1. **Separate concerns into distinct files**:
   - Types/structures (data definitions)
   - Loading/persistence (I/O operations)
   - Resolution/merging (business logic)
   - Validation (error checking)

2. **Use newtype wrappers for domain concepts**:
   - Prevents mixing unvalidated and validated data
   - Enables compile-time guarantees (e.g., `UtcTimestamp` vs `LocalTimestamp`)
   - Clear API boundaries

3. **Precedence handling with Option<T>**:
   - Keep values as `Option<T>` through precedence chain
   - Unwrap to defaults only at final resolution
   - Enables source tracking for better error messages

**Decision**: Create `src/config/` module with 4 files: `types.rs`, `loader.rs`, `resolver.rs`, `timezone.rs`

**Rationale**: 
- Current `config.rs` is 354 lines with mixed concerns
- Separation enables independent testing of each concern
- Follows established Rust conventions (cargo, rustc use similar patterns)

**Alternatives Considered**:
- Single `config.rs` with inline modules: Rejected - harder to navigate, unclear boundaries
- Multiple top-level files (config_types.rs, config_loader.rs): Rejected - pollutes src/ directory, doesn't scale

---

### 2. Generic Precedence Function Design

**Research Question**: How should we implement a single generic precedence function that works for all configuration parameters?

**Findings**:

Three main approaches for precedence handling in Rust:

1. **Macro-based approach**: `precedence!(cli, config, default)`
   - Pro: Concise call sites
   - Con: Less explicit, harder to debug, macro complexity

2. **Generic function with Option<T>**: `resolve(cli: Option<T>, config: Option<T>, default: T) -> T`
   - Pro: Type-safe, explicit, easy to test
   - Con: Requires default value at call site

3. **Builder pattern with chaining**: `Resolver::new().cli(val).config(val).default(val).build()`
   - Pro: Fluent API, self-documenting
   - Con: Overkill for simple precedence, more code

**Decision**: Generic function with `Option<T>` signature: `fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T`

**Rationale**:
- Explicit and easy to understand
- Works with all types (f64, String, i32)
- Simple to test with property-based tests
- Matches Rust idiomatic patterns (Option is the "Rust way")

**Alternatives Considered**:
- Macro-based: Rejected - adds unnecessary complexity, harder to maintain
- Builder pattern: Rejected - overkill for 3-source precedence

**Implementation Pattern**:
```rust
pub fn resolve<T>(cli: Option<T>, config: Option<T>, default: T) -> T {
    cli.or(config).unwrap_or(default)
}

// Enhanced version with source tracking for error messages
pub fn resolve_with_source<T>(
    cli: Option<T>, 
    config: Option<T>, 
    default: T
) -> (T, ConfigSource) 
where T: Clone 
{
    if let Some(val) = cli {
        (val, ConfigSource::Cli)
    } else if let Some(val) = config {
        (val, ConfigSource::ConfigFile)
    } else {
        (default, ConfigSource::Default)
    }
}
```

---

### 3. Persistence Policy Design

**Research Question**: What is the standard approach for CLI configuration persistence in similar tools?

**Findings**:

Survey of common CLI tools and their persistence behavior:

| Tool | Persistence Strategy | Rationale |
|------|---------------------|-----------|
| `kubectl` | Explicit via `kubectl config set` | Prevents accidental config changes |
| `aws-cli` | Explicit via `aws configure` | Clear separation between usage and configuration |
| `docker` | No auto-persistence | CLI flags override config only for that run |
| `git` | Explicit via `git config` | Configuration is deliberate action |
| `cargo` | No auto-persistence | Cargo.toml is edited manually |

**Current windsurf-forecast behavior**: Auto-saves timezone when provided via CLI flag

**Decision**: Remove auto-save; require explicit `--save` flag for ALL parameter persistence

**Rationale**:
- Follows industry best practices (kubectl, aws-cli pattern)
- Prevents surprising behavior (users don't expect CLI flags to modify files)
- Clear separation: CLI flags = temporary override, config file = persistent settings
- Consistent across all parameters (coordinates, timezone, provider, days)

**Alternatives Considered**:
- Keep auto-save for all parameters: Rejected - violates principle of least surprise
- Auto-save by default with `--no-save` opt-out: Rejected - still surprises users
- Interactive prompt "Save to config? (y/n)": Rejected - breaks non-interactive usage

**Implementation Approach**:
```rust
pub struct Args {
    // ... existing fields ...
    
    /// Save configuration to file after successful execution
    #[arg(long)]
    pub save: bool,
}

// In main.rs orchestration:
if args.save {
    save_config(&resolved_config, args.config.as_ref())?;
    eprintln!("Configuration saved to {}", config_path.display());
}
```

---

### 4. ResolvedConfig Structure Design

**Research Question**: What should the final validated configuration structure look like?

**Findings**:

Two main patterns for configuration structures in Rust:

1. **Flat structure with all fields**:
```rust
pub struct ResolvedConfig {
    pub provider: String,
    pub timezone: Tz,
    pub lat: f64,
    pub lng: f64,
    pub days_ahead: i32,
    pub first_day_offset: i32,
}
```

2. **Nested structure by domain**:
```rust
pub struct ResolvedConfig {
    pub provider: ProviderConfig,
    pub location: LocationConfig,
    pub forecast: ForecastConfig,
}
```

**Decision**: Flat structure with all fields

**Rationale**:
- Simpler access patterns (`config.lat` vs `config.location.lat`)
- Fewer indirections in main.rs orchestration
- Matches current usage patterns in codebase
- Not enough fields to justify grouping complexity

**Alternatives Considered**:
- Nested structure: Rejected - adds indirection without clear benefit at current scale (6 fields)
- Multiple specialized configs: Rejected - creates coupling issues, harder to pass around

**Structure Definition**:
```rust
/// Final validated configuration ready for use
///
/// All values have been resolved using precedence rules (CLI > Config > Default)
/// and validated according to business rules.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Weather provider to use
    pub provider: String,
    
    /// Target timezone for timestamps
    pub timezone: Tz,
    
    /// Latitude coordinate
    pub lat: f64,
    
    /// Longitude coordinate
    pub lng: f64,
    
    /// Number of days to forecast
    pub days_ahead: i32,
    
    /// Offset for forecast start date
    pub first_day_offset: i32,
}
```

---

### 5. Error Message Enhancement Strategy

**Research Question**: How can we improve error messages to include parameter source information?

**Findings**:

Best practices for configuration error messages in CLI tools:

1. **Include the "what"**: Which parameter failed
2. **Include the "why"**: What validation rule was violated
3. **Include the "where"**: Where the value came from (CLI, config, default)
4. **Include the "how to fix"**: Actionable resolution steps

Example from `kubectl`:
```
Error: Invalid value "foo" for namespace from config file ~/.kube/config
Namespace must match pattern ^[a-z0-9]([-a-z0-9]*[a-z0-9])?$
Fix: Edit config file or use --namespace flag
```

**Decision**: Enhance validation errors with source tracking and actionable guidance

**Implementation Pattern**:
```rust
pub enum ConfigSource {
    Cli,
    ConfigFile,
    Default,
}

pub struct ValidationError {
    parameter: String,
    value: String,
    source: ConfigSource,
    rule: String,
    suggestion: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid value '{}' for {} from {}\n\
             Rule: {}\n\
             Fix: {}",
            self.value,
            self.parameter,
            self.source,
            self.rule,
            self.suggestion
        )
    }
}
```

**Rationale**:
- Reduces debugging time for users
- Clear about where bad values originated
- Follows industry best practices
- Improves user experience without code complexity

---

### 6. Main Function Simplification Strategy

**Research Question**: What should the simplified main.rs orchestration look like?

**Findings**:

Analysis of well-designed Rust CLI applications (`ripgrep`, `fd`, `bat`) shows common pattern:

```rust
fn main() -> Result<()> {
    // 1. Load raw inputs
    let args = parse_args()?;
    
    // 2. Resolve configuration
    let config = resolve_config(args)?;
    
    // 3. Execute business logic
    let result = execute(config)?;
    
    // 4. Handle output
    output(result)?;
    
    Ok(())
}
```

**Decision**: Reduce main.rs from 275 lines to ~100 lines using clear phase separation

**Target Structure**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Phase 1: Load sources
    dotenv::dotenv().ok();
    let args = Args::parse();
    validate_args(&args)?;
    
    // Phase 2: Resolve configuration
    let config = config::resolve_from_args_and_file(&args)?;
    
    // Phase 3: Instantiate provider
    let provider = provider_registry::create_provider(&config.provider)?;
    
    // Phase 4: Fetch data
    let data = provider.fetch_forecast(
        config.lat,
        config.lng,
        config.days_ahead,
        config.first_day_offset,
        config.timezone,
    ).await?;
    
    // Phase 5: Output
    output_json(&data)?;
    
    // Phase 6: Persistence (if requested)
    if args.save {
        config::save_config_from_resolved(&config, args.config.as_ref())?;
    }
    
    Ok(())
}
```

**Rationale**:
- Clear phase boundaries
- Each phase is ~5-10 lines
- Easy to understand flow at a glance
- Configuration complexity hidden in config module

---

## Implementation Dependencies

### Technology Stack (No Changes)
- Rust 1.75+ (edition 2021)
- clap 4.0 for CLI parsing
- serde 1.0 + toml 0.8 for serialization
- chrono 0.4 + chrono-tz 0.8 for timezone handling
- anyhow 1.0 for error handling

### External Patterns Used
- Option<T> for precedence resolution (Rust standard library pattern)
- Newtype wrappers for type safety (established Rust idiom)
- Module organization by concern (cargo, rustc pattern)
- Explicit persistence (kubectl, aws-cli pattern)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking existing tests | Low | High | Maintain exact same validation rules; tests should pass without modification |
| User confusion about persistence | Medium | Low | Clear error messages; document `--save` flag prominently |
| Config module complexity | Low | Medium | Keep each file under 150 lines; clear separation of concerns |
| Performance regression | Very Low | Low | Configuration resolution is <1% of execution time; no loops or complex logic |

## References

- Rust CLI Working Group best practices: https://rust-cli.github.io/book/
- clap documentation on configuration: https://docs.rs/clap/latest/clap/
- cargo source code config module: https://github.com/rust-lang/cargo/tree/master/src/cargo/util/config
- kubectl configuration design: https://kubernetes.io/docs/concepts/configuration/organize-cluster-access-kubeconfig/

## Summary

All research areas have been resolved with clear decisions and rationales. The refactor follows established Rust patterns and industry best practices for CLI configuration management. No additional clarification needed - ready to proceed to Phase 1 (Design & Contracts).
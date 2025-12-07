# Research: Centralized Provider Registry

**Feature**: 001-provider-registry  
**Date**: 2025-12-07  
**Phase**: Phase 0 - Research & Technical Decisions

## Overview

This document consolidates research findings and technical decisions for implementing a centralized provider registry to eliminate the anti-pattern of manual 3-location registration when adding weather providers.

## Key Technical Decisions

### 1. Provider Registration Mechanism

**Decision**: Use the `inventory` crate with `inventory::submit!()` macro pattern

**Rationale**:
- **Runtime collection**: The `inventory` crate collects submitted items during runtime initialization (before main), enabling automatic provider discovery without compile-time code generation
- **Zero-cost abstraction**: Inventory uses static linking - no runtime performance penalty beyond initialization
- **Self-registration pattern**: Each provider module calls `inventory::submit!()` to register itself, eliminating the need for manual central registration
- **Rust ecosystem standard**: Widely used pattern in plugin systems (e.g., `linkme` crate uses similar approach)

**Alternatives Considered**:
1. **Procedural macros for compile-time registry**
   - Rejected: Requires significant macro complexity and crate restructuring
   - Would need separate crate for procedural macro (Rust limitation)
   - Compile-time validation benefit doesn't outweigh implementation complexity

2. **Manual registration in central location**
   - Rejected: This is the anti-pattern we're fixing
   - Current approach with 3 locations is error-prone

3. **Trait objects with static initialization**
   - Rejected: Rust doesn't support trait object static initialization in a distributed manner
   - Would still require central registration point

### 2. Provider Metadata Structure

**Decision**: Create `ProviderMetadata` struct with factory function pattern

```rust
pub struct ProviderMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub api_key_var: &'static str,
    pub instantiate: fn() -> Box<dyn ForecastProvider>,
}
```

**Rationale**:
- **Factory pattern**: `instantiate: fn() -> Box<dyn ForecastProvider>` allows each provider to construct itself with its own logic (e.g., OpenWeatherMap's special dotenv access)
- **Static lifetime**: All metadata is `&'static str` ensuring zero-copy and compile-time strings
- **API key indirection**: Provider declares the environment variable name, registry retrieves it using `std::env::var()` or provider handles it internally
- **Self-documenting**: Description field enables dynamic `--help` generation

**Alternatives Considered**:
1. **Include API key in metadata**
   - Rejected: Security concern and breaks separation of concerns
   - API keys should be retrieved at runtime, not stored in static data

2. **Separate instantiation and API key functions**
   - Rejected: More complex than needed
   - Factory pattern where provider constructs itself is simpler

### 3. Duplicate Detection Strategy

**Decision**: Panic during registry initialization if duplicate provider names detected

**Rationale**:
- **Fail-fast principle**: Duplicate names indicate programmer error, not user error
- **Clear error message**: Panic message will list both conflicting provider names and their module paths
- **Initialization-time check**: Occurs before CLI parsing, ensuring registry integrity
- **Developer-focused**: This error only occurs during development when adding providers

**Implementation**:
```rust
pub fn init_registry() -> Result<(), String> {
    let mut seen = HashMap::new();
    for meta in inventory::iter::<ProviderMetadata> {
        if let Some(existing) = seen.insert(meta.name, meta) {
            panic!(
                "Duplicate provider name '{}' detected!\n\
                First registration: [module path from existing]\n\
                Second registration: [module path from current]",
                meta.name
            );
        }
    }
    Ok(())
}
```

**Alternatives Considered**:
1. **Return error instead of panic**
   - Rejected: Duplicates are programmer errors, not recoverable runtime errors
   - Panic provides clearer indication that code needs fixing

2. **Last-registration-wins**
   - Rejected: Silently masks bugs, violates Error Transparency principle

### 4. Provider Name Validation Timing

**Decision**: Runtime validation during CLI argument parsing (not compile-time)

**Rationale**:
- **Inventory pattern constraint**: Provider discovery happens at runtime, so compile-time validation is impossible
- **Clear error messages**: Runtime validation allows listing all available providers when invalid name provided
- **Constitution compliance**: Satisfies Error Transparency principle with actionable error messages

**Implementation**:
```rust
pub fn validate_provider(provider_name: &str) -> Result<()> {
    if get_provider_metadata(provider_name).is_some() {
        Ok(())
    } else {
        let available: Vec<_> = registry::all_provider_names().collect();
        anyhow::bail!(
            "Unknown provider '{}'. Available providers: {}",
            provider_name,
            available.join(", ")
        )
    }
}
```

**Alternatives Considered**:
1. **Compile-time validation via macros**
   - Rejected: Not feasible with inventory pattern's runtime discovery
   - Would require complete architectural redesign

### 5. Registry Module Location

**Decision**: Create new file `src/provider_registry.rs`

**Rationale**:
- **Clear separation**: Registry logic isolated from provider implementations and main application logic
- **Single responsibility**: Module focuses solely on provider registration and lookup
- **Discoverability**: Obvious location for future developers to understand registration mechanism

**Module Structure**:
```rust
// src/provider_registry.rs
pub struct ProviderMetadata { /* ... */ }
inventory::collect!(ProviderMetadata);

pub fn get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata>
pub fn all_provider_names() -> impl Iterator<Item = &'static str>
pub fn all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)>
```

**Alternatives Considered**:
1. **Add to existing `providers/mod.rs`**
   - Rejected: Would mix provider declarations with registry mechanism
   - Registry is application-level concern, not provider-level

2. **Keep in `main.rs`**
   - Rejected: Would bloat main.rs and reduce modularity

### 6. API Key Retrieval Pattern

**Decision**: Keep API key retrieval in provider modules, registry stores variable name

**Rationale**:
- **Provider-specific logic**: OpenWeatherMap uses `dotenv::var()` directly, StormGlass uses `std::env::var()` - this heterogeneity is intentional per constitution
- **Backward compatibility**: Preserves existing provider implementations' API key logic
- **Flexibility**: Allows providers to implement custom key retrieval logic if needed

**Implementation Pattern**:
```rust
// In provider module
impl StormGlassProvider {
    pub fn get_api_key() -> Result<String> {
        std::env::var("STORMGLASS_API_KEY")
            .map_err(|_| anyhow!("STORMGLASS_API_KEY not set"))
    }
}

// In registry metadata
inventory::submit! {
    ProviderMetadata {
        name: "stormglass",
        api_key_var: "STORMGLASS_API_KEY",
        // ...
    }
}
```

**Alternatives Considered**:
1. **Centralize API key retrieval in registry**
   - Rejected: Would break OpenWeatherMap's direct `dotenv::var()` usage
   - Constitution explicitly documents this as intentional pattern

### 7. Help Text Generation

**Decision**: Dynamically generate provider list in CLI `--help` from registry

**Rationale**:
- **Single source of truth**: Provider descriptions come from registry metadata
- **Zero maintenance**: Help text updates automatically when providers added/removed
- **Consistency**: Same descriptions used in error messages and help text

**Implementation**:
```rust
// In args.rs
#[arg(
    long,
    default_value = "stormglass",
    value_name = "PROVIDER",
    help = "Weather forecast provider to use. Available providers: {list from registry}"
)]
pub provider: String,
```

**Alternatives Considered**:
1. **Hardcoded help text**
   - Rejected: Defeats the purpose of centralized registry
   - Would still require manual updates

## Implementation Dependencies

### Required Crate Addition

**Crate**: `inventory = "0.3"`

**Justification**:
- Small, focused crate (~2KB compiled size increase)
- Zero runtime overhead after initialization
- Industry-standard pattern for plugin systems in Rust
- Maintained by dtolnay (trusted Rust ecosystem maintainer)

**Cargo.toml Addition**:
```toml
[dependencies]
inventory = "0.3"
```

### No Breaking Changes

**Backward Compatibility**:
- Existing provider API remains unchanged
- `ForecastProvider` trait untouched
- Provider implementations only add registration call
- CLI interface identical to users
- Output format unchanged

## Migration Strategy

### Phase 1: Add Registry Infrastructure
1. Add `inventory` dependency to Cargo.toml
2. Create `src/provider_registry.rs` with metadata struct and collection
3. Update `src/main.rs` to declare registry module

### Phase 2: Migrate Existing Providers
1. Add `inventory::submit!()` calls to `stormglass.rs` and `openweathermap.rs`
2. Keep existing API key retrieval logic intact

### Phase 3: Replace Manual Registration
1. Replace `create_provider()` match statement with registry lookup
2. Replace `validate_provider()` match statement with registry validation
3. Replace API key retrieval match statement with registry-based lookup

### Phase 4: Update Documentation
1. Update constitution.md to remove 7-step protocol
2. Update AGENTS.md to remove 3-location pattern
3. Update ADDING_PROVIDERS.md with new registration pattern

## Risk Assessment

### Low Risks
- **Performance**: Inventory has negligible performance impact (one-time initialization)
- **Compatibility**: No breaking changes to existing code
- **Maintenance**: Inventory crate is stable and well-maintained

### Mitigations
- **Testing**: Comprehensive testing of provider registration and validation
- **Error handling**: Clear panic messages for duplicate registration errors
- **Documentation**: Detailed quickstart guide for adding new providers

## Success Metrics

1. **Code reduction**: Remove 3 manual registration locations, replace with 1 self-registration per provider
2. **Developer experience**: New provider addition requires only implementing trait + adding registration call
3. **Maintainability**: Zero updates to central code when adding/removing providers
4. **Error clarity**: Invalid provider names produce actionable error messages listing available options

## References

- [inventory crate documentation](https://docs.rs/inventory/)
- [Rust plugin system patterns](https://adventures.michaelfbryan.com/posts/plugins-in-rust/)
- Existing codebase: [`src/main.rs`](../../src/main.rs:1), [`src/args.rs`](../../src/args.rs:1), [`src/providers/`](../../src/providers/)
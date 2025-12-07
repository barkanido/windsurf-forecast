# Data Model: Centralized Provider Registry

**Feature**: 001-provider-registry  
**Date**: 2025-12-07  
**Phase**: Phase 1 - Design & Contracts

## Overview

This document defines the data structures and relationships for the centralized provider registry system. The registry uses static metadata collection via the `inventory` crate to enable automatic provider discovery.

## Core Entities

### 1. ProviderMetadata

**Purpose**: Static metadata describing a weather provider's registration information

**Location**: [`src/provider_registry.rs`](../../src/provider_registry.rs:1)

**Structure**:
```rust
/// Metadata for a registered weather provider
/// 
/// This struct is submitted via `inventory::submit!()` in each provider module
/// to register the provider with the central registry.
pub struct ProviderMetadata {
    /// Unique identifier for the provider (e.g., "stormglass", "openweathermap")
    /// Used in CLI --provider flag
    pub name: &'static str,
    
    /// Human-readable description for help text
    pub description: &'static str,
    
    /// Environment variable name for the provider's API key
    /// (e.g., "STORMGLASS_API_KEY", "OPEN_WEATHER_MAP_API_KEY")
    pub api_key_var: &'static str,
    
    /// Factory function that creates an instance of this provider
    /// The function is responsible for retrieving its own API key
    pub instantiate: fn() -> Result<Box<dyn ForecastProvider>>,
}
```

**Field Validation Rules**:
- `name`: Must be non-empty, lowercase, alphanumeric with optional hyphens/underscores
- `name`: Must be unique across all registered providers (enforced at runtime initialization)
- `description`: Should be concise (< 100 chars) for help text formatting
- `api_key_var`: Should follow SCREAMING_SNAKE_CASE convention
- `instantiate`: Must return `Result<Box<dyn ForecastProvider>>` for error handling

**Lifecycle**:
1. **Compile-time**: Struct definition in provider module
2. **Link-time**: `inventory::submit!()` macro registers metadata
3. **Runtime initialization**: Registry collects all submissions via `inventory::iter()`
4. **Runtime usage**: Lookup by name for validation and instantiation

**Example Registration**:
```rust
// In src/providers/stormglass.rs
inventory::submit! {
    ProviderMetadata {
        name: "stormglass",
        description: "StormGlass Marine Weather API",
        api_key_var: "STORMGLASS_API_KEY",
        instantiate: || {
            let api_key = StormGlassProvider::get_api_key()?;
            Ok(Box::new(StormGlassProvider::new(api_key)))
        },
    }
}
```

---

### 2. ProviderRegistry (Implicit)

**Purpose**: Runtime collection of all registered provider metadata

**Location**: [`src/provider_registry.rs`](../../src/provider_registry.rs:1)

**Structure**: Not a concrete struct, but a set of functions operating on inventory collection

**Key Functions**:
```rust
/// Retrieve metadata for a specific provider by name
pub fn get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata>

/// Iterator over all registered provider names
pub fn all_provider_names() -> impl Iterator<Item = &'static str>

/// Iterator over all providers with (name, description) pairs
pub fn all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)>

/// Create a provider instance by name
pub fn create_provider(name: &str) -> Result<Box<dyn ForecastProvider>>

/// Validate that a provider name exists in the registry
pub fn validate_provider_name(name: &str) -> Result<()>
```

**State**: Stateless - operates on static inventory collection

**Initialization**: Implicit via `inventory::collect!()` macro before main()

**Duplicate Detection**:
```rust
/// Called early in main() to ensure no duplicate provider names
pub fn check_duplicates() {
    let mut seen = std::collections::HashMap::new();
    for meta in inventory::iter::<ProviderMetadata>() {
        if seen.insert(meta.name, meta).is_some() {
            panic!(
                "FATAL: Duplicate provider name '{}' detected in registry!\n\
                This indicates a programming error where two provider modules \
                registered with the same name.",
                meta.name
            );
        }
    }
}
```

---

## Entity Relationships

```
┌─────────────────────────────────────────────────────────┐
│ Inventory Collection (Static Linking)                  │
│ - Collects ProviderMetadata at link time               │
│ - Available at runtime via inventory::iter()           │
└─────────────────────────────────────────────────────────┘
                              │
                              │ collects
                              ▼
┌──────────────────────────────────────────────────────────┐
│ ProviderMetadata (Multiple Instances)                   │
│ - name: "stormglass"                                    │
│ - description: "StormGlass Marine Weather API"          │
│ - api_key_var: "STORMGLASS_API_KEY"                    │
│ - instantiate: fn() -> Result<Box<dyn ForecastProvider>>│
└──────────────────────────────────────────────────────────┘
                              │
                              │ instantiate()
                              ▼
┌──────────────────────────────────────────────────────────┐
│ ForecastProvider Trait Object                          │
│ (Existing Entity - Unchanged)                           │
│ - fetch_weather_data()                                  │
│ - name()                                                │
└──────────────────────────────────────────────────────────┘
```

**Relationship Details**:

1. **Inventory → ProviderMetadata** (1:N)
   - One inventory collection contains multiple provider metadata entries
   - Collected automatically at link time via `inventory::submit!()`
   - Accessible at runtime via `inventory::iter::<ProviderMetadata>()`

2. **ProviderMetadata → ForecastProvider** (1:1)
   - Each metadata entry's `instantiate` function creates one provider instance
   - Called on-demand when provider is selected via CLI
   - Returns trait object for polymorphic usage

3. **Registry Functions → ProviderMetadata** (N:1)
   - Multiple registry query functions operate on the same inventory collection
   - No state stored in registry - pure functions over static data

---

## State Transitions

### Provider Registration Flow

```
[Compile Time]
    │
    ├─→ Provider module defines ProviderMetadata
    │
    └─→ inventory::submit!() macro marks for collection
          │
          ▼
[Link Time]
    │
    └─→ Linker collects all inventory submissions into static section
          │
          ▼
[Runtime - Before main()]
    │
    └─→ inventory crate initializes collection (automatic)
          │
          ▼
[Runtime - main() start]
    │
    ├─→ check_duplicates() validates no name conflicts
    │
    └─→ Registry ready for use
          │
          ▼
[Runtime - CLI parsing]
    │
    └─→ validate_provider_name() checks against registry
          │
          ▼
[Runtime - Provider instantiation]
    │
    └─→ create_provider() calls metadata.instantiate()
          │
          ▼
[Runtime - Weather data fetch]
    │
    └─→ provider.fetch_weather_data() executes
```

---

## Validation Rules

### Name Uniqueness Validation
- **When**: Runtime initialization (before CLI parsing)
- **How**: Iterate through inventory, build HashMap of names
- **Failure**: Panic with message listing duplicate name and module paths
- **Rationale**: Duplicates are programmer errors, not user errors

### Provider Existence Validation
- **When**: CLI argument parsing
- **How**: Check if `get_provider_metadata(name)` returns Some
- **Failure**: Return error listing all available providers
- **Rationale**: User error - provide actionable guidance

### API Key Validation
- **When**: Provider instantiation (inside `instantiate()`)
- **How**: Provider-specific logic (may use `std::env::var()` or `dotenv::var()`)
- **Failure**: Return error with environment variable name
- **Rationale**: Configuration error - tell user which variable to set

---

## Data Flow Example

### Adding a New Provider

```rust
// Step 1: Implement ForecastProvider trait (existing pattern)
pub struct WeatherAPIProvider {
    api_key: String,
}

impl ForecastProvider for WeatherAPIProvider {
    // ... implementation
}

// Step 2: Add API key retrieval (existing pattern)
impl WeatherAPIProvider {
    pub fn get_api_key() -> Result<String> {
        std::env::var("WEATHERAPI_API_KEY")
            .map_err(|_| anyhow!("WEATHERAPI_API_KEY not set in .env"))
    }
}

// Step 3: Register with inventory (NEW - only new step)
inventory::submit! {
    ProviderMetadata {
        name: "weatherapi",
        description: "WeatherAPI.com Global Weather Data",
        api_key_var: "WEATHERAPI_API_KEY",
        instantiate: || {
            let api_key = WeatherAPIProvider::get_api_key()?;
            Ok(Box::new(WeatherAPIProvider::new(api_key)))
        },
    }
}
```

**Result**: Provider automatically appears in:
- `cargo run -- --help` output
- CLI validation (accepts `--provider weatherapi`)
- Error messages if validation fails

**No Changes Required**:
- ❌ No update to `src/main.rs`
- ❌ No update to `src/args.rs`
- ❌ No update to any central registration code

---

## Migration Impact

### Existing Entities (Unchanged)
- [`ForecastProvider`](../../src/forecast_provider.rs:1) trait
- [`WeatherDataPoint`](../../src/forecast_provider.rs:19) struct
- Provider implementation structs ([`StormGlassProvider`](../../src/providers/stormglass.rs:1), [`OpenWeatherMapProvider`](../../src/providers/openweathermap.rs:1))
- API key retrieval methods

### New Entities
- `ProviderMetadata` struct (new)
- Registry functions in `provider_registry.rs` (new module)

### Modified Code
- Provider modules: Add `inventory::submit!()` call (3 lines added per provider)
- [`main.rs`](../../src/main.rs:41): Replace `create_provider()` match with registry lookup (net -10 lines)
- [`main.rs`](../../src/main.rs:192): Replace API key match with metadata lookup (net -5 lines)
- [`args.rs`](../../src/args.rs:97): Replace `validate_provider()` match with registry check (net -5 lines)

**Total LOC Impact**: ~-20 lines in central code, +3 lines per provider

---

## Error Handling

### Error Types

1. **Duplicate Provider Name** (Panic)
   ```
   FATAL: Duplicate provider name 'stormglass' detected in registry!
   This indicates a programming error where two provider modules registered with the same name.
   First registration: src/providers/stormglass.rs
   Second registration: src/providers/stormglass_v2.rs
   ```

2. **Unknown Provider Name** (User Error)
   ```
   Error: Unknown provider 'stromglass'. Available providers: openweathermap, stormglass
   ```

3. **Provider Instantiation Failure** (Configuration Error)
   ```
   Error: Failed to create provider 'stormglass'
   Caused by: STORMGLASS_API_KEY not set in .env
   ```

---

## Performance Characteristics

- **Initialization**: O(n) where n = number of providers (negligible, n < 10)
- **Lookup by name**: O(n) linear search through inventory iterator (acceptable for small n)
- **Memory overhead**: ~100 bytes per provider (4 pointers + string data)
- **Runtime performance**: No difference vs current hardcoded approach

**Optimization Note**: Could add HashMap caching if n grows large, but unnecessary for current scale.
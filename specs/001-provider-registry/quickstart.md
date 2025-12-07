# Quickstart: Implementing Centralized Provider Registry

**Feature**: 001-provider-registry  
**Date**: 2025-12-07  
**Phase**: Phase 1 - Implementation Guide

## Overview

This guide provides step-by-step instructions for implementing the centralized provider registry feature. Follow these steps in order to successfully migrate from the 3-location manual registration pattern to the automated registry system.

## Prerequisites

- Rust 1.75+ installed
- Existing codebase at branch `001-provider-registry`
- Familiarity with cargo and Rust modules

## Implementation Steps

### Step 1: Add inventory Dependency

**File**: [`Cargo.toml`](../../Cargo.toml:1)

**Action**: Add the inventory crate to dependencies

```toml
[dependencies]
inventory = "0.3"
# ... existing dependencies
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds, inventory crate downloaded

---

### Step 2: Create Provider Registry Module

**File**: `src/provider_registry.rs` (NEW FILE)

**Action**: Create the registry module with the following structure:

```rust
//! Centralized provider registry using inventory pattern
//! 
//! This module provides automatic provider discovery and registration.
//! Providers self-register using the `inventory::submit!()` macro.

use anyhow::{Result, anyhow};
use crate::forecast_provider::ForecastProvider;
use std::collections::HashMap;

/// Metadata for a registered weather provider
pub struct ProviderMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub api_key_var: &'static str,
    pub instantiate: fn() -> Result<Box<dyn ForecastProvider>>,
}

// Enable inventory collection of ProviderMetadata
inventory::collect!(ProviderMetadata);

/// Get metadata for a specific provider by name
pub fn get_provider_metadata(name: &str) -> Option<&'static ProviderMetadata> {
    inventory::iter::<ProviderMetadata>()
        .find(|meta| meta.name == name)
}

/// Iterator over all registered provider names
pub fn all_provider_names() -> impl Iterator<Item = &'static str> {
    inventory::iter::<ProviderMetadata>()
        .map(|meta| meta.name)
}

/// Iterator over provider (name, description) pairs
pub fn all_provider_descriptions() -> impl Iterator<Item = (&'static str, &'static str)> {
    inventory::iter::<ProviderMetadata>()
        .map(|meta| (meta.name, meta.description))
}

/// Create a provider instance by name
pub fn create_provider(name: &str) -> Result<Box<dyn ForecastProvider>> {
    match get_provider_metadata(name) {
        Some(meta) => (meta.instantiate)(),
        None => {
            let available: Vec<_> = all_provider_names().collect();
            Err(anyhow!(
                "Unknown provider '{}'. Available providers: {}",
                name,
                available.join(", ")
            ))
        }
    }
}

/// Validate that a provider name exists
pub fn validate_provider_name(name: &str) -> Result<()> {
    if get_provider_metadata(name).is_some() {
        Ok(())
    } else {
        let available: Vec<_> = all_provider_names().collect();
        Err(anyhow!(
            "Unknown provider '{}'. Available providers: {}",
            name,
            available.join(", ")
        ))
    }
}

/// Check for duplicate provider names (call early in main)
pub fn check_duplicates() {
    let mut seen = HashMap::new();
    for meta in inventory::iter::<ProviderMetadata>() {
        if let Some(existing) = seen.insert(meta.name, meta) {
            panic!(
                "FATAL: Duplicate provider name '{}' detected in registry!\n\
                This indicates a programming error where two provider modules \
                registered with the same name.\n\
                First registration: {} ({})\n\
                Second registration: {} ({})",
                meta.name,
                existing.description,
                existing.api_key_var,
                meta.description,
                meta.api_key_var
            );
        }
    }
}

/// Get count of registered providers (utility)
pub fn provider_count() -> usize {
    inventory::iter::<ProviderMetadata>().count()
}
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 3: Declare Registry Module

**File**: [`src/main.rs`](../../src/main.rs:1)

**Action**: Add module declaration at top of file

**Find this section**:
```rust
mod args;
mod config;
mod forecast_provider;
mod providers;
```

**Change to**:
```rust
mod args;
mod config;
mod forecast_provider;
mod provider_registry;  // NEW
mod providers;
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 4: Register StormGlass Provider

**File**: [`src/providers/stormglass.rs`](../../src/providers/stormglass.rs:1)

**Action**: Add registration at end of file

**Add these imports** at the top:
```rust
use crate::provider_registry::ProviderMetadata;
```

**Add this registration** at the end of the file:
```rust
// Register provider with central registry
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

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 5: Register OpenWeatherMap Provider

**File**: [`src/providers/openweathermap.rs`](../../src/providers/openweathermap.rs:1)

**Action**: Add registration at end of file

**Add these imports** at the top:
```rust
use crate::provider_registry::ProviderMetadata;
```

**Add this registration** at the end of the file:
```rust
// Register provider with central registry
inventory::submit! {
    ProviderMetadata {
        name: "openweathermap",
        description: "OpenWeatherMap Global Weather Data",
        api_key_var: "OPEN_WEATHER_MAP_API_KEY",
        instantiate: || {
            let api_key = OpenWeatherMapProvider::get_api_key()?;
            Ok(Box::new(OpenWeatherMapProvider::new(api_key)))
        },
    }
}
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 6: Replace create_provider() in main.rs

**File**: [`src/main.rs`](../../src/main.rs:41)

**Action**: Replace manual provider instantiation with registry lookup

**Add import** at top:
```rust
use provider_registry;
```

**Find this function** (lines ~41-48):
```rust
fn create_provider(provider_name: &str, api_key: String) -> Result<Box<dyn ForecastProvider>> {
    match provider_name {
        "stormglass" => Ok(Box::new(StormGlassProvider::new(api_key))),
        "openweathermap" => Ok(Box::new(OpenWeatherMapProvider::new(api_key))),
        // Future providers can be added here
        _ => unreachable!("Provider validation should have caught this"),
    }
}
```

**Delete the entire function** - it's no longer needed

**Validation**:
```bash
cargo check
```

**Expected**: Compilation errors about `create_provider` not found (this is expected)

---

### Step 7: Replace API Key Retrieval in main.rs

**File**: [`src/main.rs`](../../src/main.rs:192)

**Action**: Remove API key retrieval match statement

**Find this code** (lines ~192-197):
```rust
// Get API key for the selected provider
let api_key = match args.provider.as_str() {
    "stormglass" => StormGlassProvider::get_api_key()?,
    "openweathermap" => OpenWeatherMapProvider::get_api_key()?,
    // Future providers can be added here
    _ => unreachable!("Provider validation should have caught this"),
};

// Create provider instance
let provider = create_provider(&args.provider, api_key)?;
```

**Replace with**:
```rust
// Create provider instance using registry
let provider = provider_registry::create_provider(&args.provider)?;
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds (API key retrieval now inside `instantiate` closures)

---

### Step 8: Replace Provider Validation in args.rs

**File**: [`src/args.rs`](../../src/args.rs:97)

**Action**: Replace hardcoded validation with registry lookup

**Find this function** (lines ~97-106):
```rust
pub fn validate_provider(provider_name: &str) -> Result<()> {
    match provider_name {
        "stormglass" => Ok(()),
        "openweathermap" => Ok(()),
        _ => anyhow::bail!(
            "Unknown provider '{}'. Available providers: stormglass, openweathermap",
            provider_name
        ),
    }
}
```

**Replace with**:
```rust
pub fn validate_provider(provider_name: &str) -> Result<()> {
    crate::provider_registry::validate_provider_name(provider_name)
}
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 9: Add Duplicate Check in main()

**File**: [`src/main.rs`](../../src/main.rs:116)

**Action**: Add registry validation early in main

**Find the async fn run()** (line ~123):
```rust
async fn run() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();
```

**Add after dotenv** line:
```rust
async fn run() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Validate provider registry (check for duplicates)
    provider_registry::check_duplicates();
```

**Validation**:
```bash
cargo check
```

**Expected**: Compilation succeeds

---

### Step 10: Remove Unused Imports

**File**: [`src/main.rs`](../../src/main.rs:17)

**Action**: Remove provider-specific imports that are no longer needed

**Find these imports**:
```rust
use providers::{stormglass::StormGlassProvider, openweathermap::OpenWeatherMapProvider};
```

**Delete the entire line** - providers are now accessed only through the registry

**Validation**:
```bash
cargo check
cargo clippy
```

**Expected**: 
- Compilation succeeds
- No warnings about unused imports
- Clippy passes

---

### Step 11: Test Basic Functionality

**Action**: Run the application with both providers

```bash
# Test StormGlass provider
cargo run -- --provider stormglass --days-ahead 3

# Test OpenWeatherMap provider
cargo run -- --provider openweathermap --days-ahead 2

# Test invalid provider (should show available providers)
cargo run -- --provider invalid
```

**Expected**:
- StormGlass and OpenWeatherMap run successfully
- Invalid provider shows: "Unknown provider 'invalid'. Available providers: openweathermap, stormglass"

---

### Step 12: Test Help Text

**Action**: Verify that both providers are listed

```bash
cargo run -- --help
```

**Expected**: Help text shows both "stormglass" and "openweathermap" as available

---

### Step 13: Run Full Test Suite

**Action**: Execute complete testing workflow (per Constitution VI)

```bash
# 1. Check compilation
cargo check

# 2. Build for testing
cargo build

# 3. Run clippy
cargo clippy

# 4. Test both providers
cargo run -- --provider stormglass --days-ahead 3
cargo run -- --provider openweathermap --days-ahead 2

# 5. Final validation with release build
cargo build --release
cargo run --release -- --provider stormglass
```

**Expected**: All commands succeed without errors

---

## Verification Checklist

After completing all steps, verify:

- [ ] `cargo check` passes without errors or warnings
- [ ] `cargo clippy` passes without warnings
- [ ] Both providers run successfully via CLI
- [ ] Invalid provider names show helpful error with available options
- [ ] `--help` text lists both providers
- [ ] No manual registration code remains in `main.rs` or `args.rs`
- [ ] Provider modules only contain one `inventory::submit!()` call each
- [ ] Duplicate check runs at startup (visible in debugger breakpoint)

---

## Success Metrics

**Before Implementation**:
- Adding provider required updates to 3 locations
- Manual string matching in multiple places
- Easy to miss a required update

**After Implementation**:
- Adding provider requires only `inventory::submit!()` in provider module
- Registry automatically discovers all providers
- Impossible to forget registration (compile error if trait not implemented)

---

## Troubleshooting

### Issue: "Unknown provider" error despite registration

**Cause**: Provider name in `inventory::submit!()` doesn't match CLI argument

**Solution**: Ensure `name` field in metadata matches exactly (case-sensitive)

### Issue: Duplicate provider panic on startup

**Cause**: Two providers registered with same name

**Solution**: Use unique provider names or remove one registration

### Issue: Compilation error about missing imports

**Cause**: Forgot to add `use crate::provider_registry::ProviderMetadata;`

**Solution**: Add import at top of provider module

### Issue: Provider instantiation fails

**Cause**: API key not set in .env file

**Solution**: Add required environment variable (e.g., `STORMGLASS_API_KEY=...`)

---

## Next Steps

After successful implementation:

1. Run `.specify/scripts/powershell/update-agent-context.ps1 -AgentType roo` to update agent context
2. Update constitution.md to remove 7-step protocol (see Step 14)
3. Update ADDING_PROVIDERS.md with new registration pattern
4. Create tasks.md for implementation tracking (use `/speckit.tasks` command)

---

## Adding Future Providers

To add a new provider after this implementation:

1. Create provider module: `src/providers/newprovider.rs`
2. Implement [`ForecastProvider`](../../src/forecast_provider.rs:1) trait
3. Add API key retrieval method
4. Add `inventory::submit!()` registration block
5. Declare module in `src/providers/mod.rs`

**That's it!** No central code changes needed.

See [`contracts/provider_modifications.rs`](contracts/provider_modifications.rs:1) for detailed template.
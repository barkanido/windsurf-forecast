# Data Model: Configuration Data Flow Simplification

**Feature**: 004-config-refactor  
**Date**: 2025-12-08  
**Status**: Complete

## Overview

This document defines the data structures and their relationships for the refactored configuration system. The model consolidates scattered configuration logic into a unified flow with clear type boundaries and validation points.

## Core Entities

### 1. ResolvedConfig

**Purpose**: Final validated configuration containing all resolved values ready for application use.

**Location**: `src/config/types.rs`

**Structure**:
```rust
/// Final validated configuration ready for use
///
/// All values have been resolved using precedence rules (CLI > Config > Default)
/// and validated according to business rules. This is the single source of truth
/// for configuration during application execution.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedConfig {
    /// Weather provider to use (e.g., "stormglass", "openweathermap")
    pub provider: String,
    
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Latitude coordinate (validated: -90.0 to 90.0)
    pub lat: f64,
    
    /// Longitude coordinate (validated: -180.0 to 180.0)
    pub lng: f64,
    
    /// Number of days to forecast ahead (validated: 1-7)
    pub days_ahead: i32,
    
    /// Offset for forecast start date (validated: 0-7)
    /// Combined with days_ahead must not exceed 7
    pub first_day_offset: i32,
}
```

**Validation Rules**:
- `provider`: Must match registered provider name (validated via provider_registry)
- `timezone`: Must be valid IANA timezone identifier (enforced by Tz type)
- `lat`: Range [-90.0, 90.0]
- `lng`: Range [-180.0, 180.0]
- `days_ahead`: Range [1, 7]
- `first_day_offset`: Range [0, 7]
- Business rule: `days_ahead + first_day_offset <= 7`

**Relationships**:
- Created by: `ConfigResolver::resolve()`
- Used by: `main.rs` orchestration, provider instantiation
- Never modified after creation (immutable once validated)

---

### 2. ConfigSources

**Purpose**: Raw input sources before precedence resolution, with Option<T> values to track which sources provided which values.

**Location**: `src/config/types.rs`

**Structure**:
```rust
/// Raw configuration sources before precedence resolution
///
/// Each field is Option<T> to track whether the value was provided
/// by that source. This enables source tracking for error messages.
#[derive(Debug, Clone, Default)]
pub struct ConfigSources {
    /// CLI argument values
    pub cli: CliSource,
    
    /// Config file values (from ~/.windsurf-config.toml)
    pub config_file: FileSource,
    
    /// Default values (applied when neither CLI nor config provides value)
    pub defaults: DefaultSource,
}

/// CLI argument source
#[derive(Debug, Clone, Default)]
pub struct CliSource {
    pub provider: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub days_ahead: Option<i32>,
    pub first_day_offset: Option<i32>,
}

/// Config file source
#[derive(Debug, Clone, Default)]
pub struct FileSource {
    pub provider: Option<String>,
    pub timezone: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    // Note: days_ahead and first_day_offset NOT stored in config file
}

/// Default values source
#[derive(Debug, Clone)]
pub struct DefaultSource {
    pub provider: String,          // "stormglass"
    pub timezone: String,           // "UTC"
    pub days_ahead: i32,            // 4
    pub first_day_offset: i32,      // 0
    // Note: lat/lng have NO defaults (must be provided)
}
```

**Relationships**:
- Populated from: `Args` (CLI), `Config` (file)
- Consumed by: `ConfigResolver::resolve()`
- Short-lived (only exists during resolution phase)

---

### 3. ConfigSource (Enum)

**Purpose**: Tracks where a resolved value came from for error reporting.

**Location**: `src/config/types.rs`

**Structure**:
```rust
/// Indicates which source provided a configuration value
///
/// Used in error messages to help users understand where
/// invalid values originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Value provided via CLI argument
    Cli,
    
    /// Value loaded from config file
    ConfigFile,
    
    /// Default value applied (neither CLI nor config provided value)
    Default,
}

impl fmt::Display for ConfigSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigSource::Cli => write!(f, "CLI argument"),
            ConfigSource::ConfigFile => write!(f, "config file"),
            ConfigSource::Default => write!(f, "default value"),
        }
    }
}
```

**Usage**:
```rust
// In error messages:
anyhow::bail!(
    "Invalid {} '{}' from {}\nRule: {}\nFix: {}",
    param_name, value, source, rule, suggestion
);
```

---

### 4. Config (TOML Structure)

**Purpose**: Represents the TOML configuration file structure (unchanged from current implementation).

**Location**: `src/config/loader.rs` (moved from `src/config.rs`)

**Structure**:
```rust
/// Top-level configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
}

/// General configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Timezone for displaying timestamps (e.g., "UTC", "Asia/Jerusalem")
    #[serde(default = "default_timezone")]
    pub timezone: String,
    
    /// Default weather provider
    #[serde(default = "default_provider")]
    pub default_provider: String,
    
    /// Latitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<f64>,
    
    /// Longitude for forecast location (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lng: Option<f64>,
}
```

**File Format** (unchanged):
```toml
[general]
timezone = "Asia/Jerusalem"
default_provider = "stormglass"
lat = 32.486722
lng = 34.888722
```

**Relationships**:
- Loaded by: `ConfigLoader::load()`
- Converted to: `FileSource` for precedence resolution
- Saved by: `ConfigLoader::save()` (when `--save` flag used)

---

### 5. TimezoneConfig

**Purpose**: Timezone configuration with explicit/default tracking (moved from current config.rs).

**Location**: `src/config/timezone.rs`

**Structure**:
```rust
/// Configuration for timezone handling
///
/// Manages user's timezone preferences with precedence:
/// 1. CLI --timezone flag (highest)
/// 2. Config file timezone
/// 3. Default to UTC with warning (lowest)
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Whether timezone was explicitly set by user (vs default)
    /// Used to determine if warning should be displayed
    pub explicit: bool,
}
```

**State Transitions**:
```
[No timezone provided] → TimezoneConfig { timezone: UTC, explicit: false } → Warning displayed
[CLI: "America/New_York"] → TimezoneConfig { timezone: America/New_York, explicit: true } → No warning
[Config: "Asia/Jerusalem"] → TimezoneConfig { timezone: Asia/Jerusalem, explicit: true } → No warning
[CLI: "LOCAL"] → detect_system_timezone() → TimezoneConfig { timezone: <detected>, explicit: true }
```

**Relationships**:
- Created by: `TimezoneConfig::load_with_precedence()`
- Used by: Timezone validation, warning display
- Converted to: `Tz` for inclusion in `ResolvedConfig`

---

## Data Flow

### Configuration Resolution Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│ 1. INPUT SOURCES                                             │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Args (CLI)          Config (File)        Defaults           │
│  ┌─────────┐        ┌──────────┐        ┌────────┐          │
│  │provider │        │timezone  │        │provider│          │
│  │timezone │        │provider  │        │timezone│          │
│  │lat/lng  │        │lat/lng   │        │days*   │          │
│  │days*    │        └──────────┘        └────────┘          │
│  └─────────┘                                                 │
│       │                  │                    │              │
│       └──────────────────┼────────────────────┘              │
│                          ↓                                   │
└─────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────┼───────────────────────────────────┐
│ 2. PRECEDENCE RESOLUTION │                                   │
├──────────────────────────┼───────────────────────────────────┤
│                          ↓                                   │
│              ConfigSources { cli, config_file, defaults }    │
│                          │                                   │
│                          ↓                                   │
│              ConfigResolver::resolve()                       │
│              ┌──────────────────────┐                        │
│              │ For each parameter:  │                        │
│              │ 1. Try CLI           │                        │
│              │ 2. Try config file   │                        │
│              │ 3. Use default       │                        │
│              │ 4. Track source      │                        │
│              └──────────────────────┘                        │
│                          │                                   │
│                          ↓                                   │
│              Unvalidated ResolvedConfig                      │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
┌──────────────────────────┼───────────────────────────────────┐
│ 3. VALIDATION            │                                   │
├──────────────────────────┼───────────────────────────────────┤
│                          ↓                                   │
│              ConfigValidator::validate()                     │
│              ┌──────────────────────┐                        │
│              │ Range checks         │ ← Source tracking      │
│              │ Business rules       │ ← Enhanced errors      │
│              │ Provider validation  │                        │
│              │ Coordinate ranges    │                        │
│              └──────────────────────┘                        │
│                          │                                   │
│                    Success│Failure                           │
│                          │    │                              │
│              ┌───────────┘    └─────────┐                    │
│              ↓                          ↓                    │
│   Validated ResolvedConfig      ValidationError              │
│                                  (with source info)           │
└─────────────────────────────────────────────────────────────┘
```

### Persistence Flow (Optional)

```
┌─────────────────────────────────────────────────────────────┐
│ PERSISTENCE (only if --save flag provided)                  │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ResolvedConfig                                              │
│       │                                                       │
│       ↓                                                       │
│  Convert to Config structure                                 │
│       │                                                       │
│       ↓                                                       │
│  Serialize to TOML                                           │
│       │                                                       │
│       ↓                                                       │
│  Write to ~/.windsurf-config.toml                            │
│       │                                                       │
│       ↓                                                       │
│  Display confirmation message                                │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Module Organization

### File Structure and Responsibilities

```
src/config/
├── mod.rs              # Public API exports
│   └── exports: ResolvedConfig, ConfigSources, resolve_from_args_and_file()
│
├── types.rs            # Data structure definitions
│   ├── ResolvedConfig
│   ├── ConfigSources (CliSource, FileSource, DefaultSource)
│   └── ConfigSource (enum)
│
├── loader.rs           # File I/O operations
│   ├── Config, GeneralConfig (from current config.rs)
│   ├── load_config(path) -> Config
│   ├── save_config(config, path)
│   └── get_default_config_path() -> PathBuf
│
├── resolver.rs         # Precedence logic and validation
│   ├── resolve<T>(cli, config, default) -> T
│   ├── resolve_with_source<T>(...) -> (T, ConfigSource)
│   ├── resolve_from_args_and_file(args) -> ResolvedConfig
│   ├── validate_coordinates(lat, lng)
│   └── validate_date_range(days_ahead, first_day_offset)
│
└── timezone.rs         # Timezone-specific configuration
    ├── TimezoneConfig
    ├── detect_system_timezone() -> Tz
    ├── validate_timezone_coordinates(tz, lat, lng)
    └── pick_timezone_interactive() -> String
```

## Type Safety Guarantees

### Compile-Time Guarantees

1. **Validated vs Unvalidated**: Once a `ResolvedConfig` is created, it is guaranteed to contain valid values
2. **Option<T> Tracking**: Source information is preserved through the resolution process
3. **Timezone Safety**: `Tz` type from chrono-tz ensures timezone identifiers are valid
4. **No Partial States**: `ResolvedConfig` cannot exist without all required fields

### Runtime Validation Points

1. **Args validation**: CLI arguments validated immediately after parsing
2. **Config file parsing**: TOML deserialization with helpful error messages
3. **Precedence resolution**: Each field validated after resolution
4. **Business rules**: Cross-field validation (e.g., total days constraint)

## Migration from Current Implementation

### Changes to Existing Types

| Current Location | New Location | Changes |
|-----------------|--------------|---------|
| `src/config.rs::Config` | `src/config/loader.rs::Config` | Moved, no changes |
| `src/config.rs::GeneralConfig` | `src/config/loader.rs::GeneralConfig` | Moved, no changes |
| `src/config.rs::TimezoneConfig` | `src/config/timezone.rs::TimezoneConfig` | Moved, no changes |
| `src/config.rs::load_config()` | `src/config/loader.rs::load_config()` | Moved, no changes |
| `src/config.rs::save_config()` | `src/config/loader.rs::save_config()` | Moved, no changes |
| `src/config.rs::resolve_coordinates()` | `src/config/resolver.rs::resolve_coordinates()` | Moved, refactored |
| `src/config.rs::validate_coordinates()` | `src/config/resolver.rs::validate_coordinates()` | Moved, no changes |

### New Types

- `ResolvedConfig`: New unified structure replacing scattered variables in main.rs
- `ConfigSources`: New structure for tracking input sources
- `ConfigSource` (enum): New enum for source tracking in errors
- `CliSource`, `FileSource`, `DefaultSource`: New structures for organized source tracking

### Removed from main.rs

The following variables are replaced by `ResolvedConfig`:
- Scattered `provider`, `timezone`, `lat`, `lng`, `days_ahead`, `first_day_offset` variables
- Manual precedence logic (`args.provider.or(config.general.default_provider)` patterns)
- Inline coordinate resolution and validation

## Backward Compatibility

### Guarantees

✅ **CLI Arguments**: No changes to argument names, types, or behavior  
✅ **Config File Format**: TOML structure remains identical  
✅ **Validation Rules**: All existing validation rules preserved  
✅ **Error Messages**: Enhanced with source information (additive change)  
✅ **Default Values**: Same defaults as current implementation  

### Breaking Changes

⚠️ **Persistence Behavior**: Timezone auto-save removed; requires explicit `--save` flag  
- **Rationale**: Aligns with CLI best practices; makes persistence predictable
- **Migration**: Users who relied on auto-save must add `--save` flag once to persist timezone
- **Documentation**: Clearly documented in help text and error messages

## Testing Strategy

### Unit Test Coverage

1. **Precedence Resolution**:
   - CLI overrides config file
   - Config file overrides defaults
   - Defaults applied when no other source
   - Source tracking correct for each scenario

2. **Validation**:
   - Coordinate ranges enforced
   - Days range enforced
   - Business rule (total days ≤ 7) enforced
   - Provider name validation

3. **Error Messages**:
   - Include parameter name
   - Include invalid value
   - Include source (CLI/config/default)
   - Include validation rule
   - Include fix suggestion

4. **File Operations**:
   - Load valid TOML
   - Handle missing config file
   - Handle malformed TOML
   - Save configuration correctly
   - Preserve file structure

### Integration Testing

Test complete flow from CLI to resolved config:
```rust
#[test]
fn test_full_config_resolution() {
    // Given: CLI args, config file, defaults
    // When: resolve_from_args_and_file()
    // Then: Correct precedence applied, all values validated
}
```

## Summary

The data model establishes clear boundaries between raw inputs (`ConfigSources`), resolution logic (`ConfigResolver`), and validated output (`ResolvedConfig`). Type safety is enforced through the Rust type system, with `Option<T>` used to track value sources and enable better error messages. The model maintains 100% backward compatibility with existing CLI and config file interfaces while significantly simplifying the internal implementation.
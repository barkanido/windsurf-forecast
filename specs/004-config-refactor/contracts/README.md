# API Contracts: Configuration Data Flow Simplification

**Feature**: 004-config-refactor  
**Date**: 2025-12-08  

## Overview

This directory contains the public API contracts for the refactored configuration system. These contracts define the interfaces that must be maintained for backward compatibility and serve as the specification for implementation.

## Contract Files

### 1. [types_contract.rs](types_contract.rs)
Defines the core data structures: `ResolvedConfig`, `ConfigSources`, and `ConfigSource` enum.

**Key Contracts**:
- `ResolvedConfig` structure and field types
- `ConfigSources` structure for input tracking
- `ConfigSource` enum for error reporting

### 2. [loader_contract.rs](loader_contract.rs)
Defines the file I/O operations for loading and saving TOML configuration.

**Key Contracts**:
- `Config` and `GeneralConfig` TOML structures (unchanged from current)
- `load_config()` function signature
- `save_config()` function signature
- File path resolution

### 3. [resolver_contract.rs](resolver_contract.rs)
Defines the precedence resolution and validation logic.

**Key Contracts**:
- Generic precedence resolution function
- Coordinate validation
- Date range validation
- Main resolution entry point

### 4. [timezone_contract.rs](timezone_contract.rs)
Defines timezone-specific configuration and validation.

**Key Contracts**:
- `TimezoneConfig` structure
- System timezone detection
- Interactive timezone picker
- Coordinate-timezone validation

### 5. [main_integration_contract.rs](main_integration_contract.rs)
Defines how main.rs integrates with the config module.

**Key Contracts**:
- Simplified main function structure
- Configuration resolution entry point
- Persistence behavior with `--save` flag

## Backward Compatibility Guarantees

### CLI Interface (args.rs)
✅ **No changes** to existing CLI arguments  
✅ **No changes** to argument names, types, or defaults  
✅ **New argument**: `--save` flag for explicit persistence  

### Config File Format
✅ **No changes** to TOML structure  
✅ **No changes** to field names or types  
✅ **Same defaults** applied when fields missing  

### Validation Rules
✅ **All existing rules preserved**:
- Latitude: -90.0 to 90.0
- Longitude: -180.0 to 180.0
- days_ahead: 1 to 7
- first_day_offset: 0 to 7
- Business rule: days_ahead + first_day_offset ≤ 7

### Breaking Changes
⚠️ **Persistence behavior**: Removed automatic timezone saving; requires `--save` flag  
- **Rationale**: Follows CLI best practices (kubectl, aws-cli pattern)
- **Migration**: Users who relied on auto-save must add `--save` flag once
- **Impact**: Low - only affects users who change timezone frequently

## Testing Requirements

All contracts must be verified by:
1. **Unit tests**: Each contract function has corresponding test
2. **Integration tests**: Full flow from CLI to resolved config
3. **Backward compatibility tests**: Existing 131 tests pass without modification

## Implementation Notes

### Module Organization
```
src/config/
├── mod.rs         # Public API exports (implements main_integration_contract.rs)
├── types.rs       # implements types_contract.rs
├── loader.rs      # implements loader_contract.rs
├── resolver.rs    # implements resolver_contract.rs
└── timezone.rs    # implements timezone_contract.rs
```

### Error Handling
All functions use `anyhow::Result` for consistency with existing codebase.  
Validation errors include source tracking (CLI/config/default) for better user experience.

### Thread Safety
Configuration resolution is single-threaded (happens at startup).  
No concurrent access concerns for config structures.

## References

- Feature specification: [../spec.md](../spec.md)
- Data model: [../data-model.md](../data-model.md)
- Research: [../research.md](../research.md)
- Current implementation: [../../../src/config.rs](../../../src/config.rs), [../../../src/main.rs](../../../src/main.rs)
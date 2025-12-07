# API Contracts

This directory contains Rust API contracts defining the public interfaces for the centralized provider registry feature.

## Files

### provider_registry.rs
Defines the public API contract for the `src/provider_registry.rs` module, including:
- `ProviderMetadata` struct definition
- Registry query functions (`get_provider_metadata`, `all_provider_names`, etc.)
- Provider instantiation functions (`create_provider`, `validate_provider_name`)
- Duplicate detection (`check_duplicates`)

**Purpose**: Documents the interface that main.rs, args.rs, and other modules will use to interact with the registry.

### provider_modifications.rs
Documents the required changes to existing provider modules to integrate with the registry, including:
- StormGlass provider registration example
- OpenWeatherMap provider registration example
- Template for future provider additions
- Anti-patterns to avoid

**Purpose**: Provides implementation guidance for modifying existing providers and adding new ones.

## Contract Validation

These contracts serve as:
1. **Design documentation**: Defines interfaces before implementation
2. **Implementation guide**: Shows exact function signatures and behaviors
3. **Contract tests**: Can be compiled to verify API consistency
4. **Migration reference**: Documents what changes to existing code

## Usage

During implementation, refer to these contracts to:
- Implement `src/provider_registry.rs` following the API in `provider_registry.rs`
- Modify existing providers following patterns in `provider_modifications.rs`
- Ensure consistency between design and implementation
- Validate that all specified functions are implemented

## Notes

- These are Rust API contracts, not REST/GraphQL schemas (appropriate for CLI application)
- Contracts include comprehensive documentation comments suitable for rustdoc
- Error handling and panicking behavior is explicitly documented
- Examples demonstrate proper usage patterns
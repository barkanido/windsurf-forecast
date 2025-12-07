# Contracts: Timezone Conversion Architecture Refactor

This directory contains the concrete Rust type definitions that form the implementation contracts for the timezone refactor feature.

## Purpose

These contract files define:
- **Newtype wrappers** for compile-time timezone safety
- **Updated trait signatures** with timezone parameters
- **Configuration structures** for timezone handling
- **Helper functions** for timezone conversion

## Files

### [`newtype_wrappers.rs`](newtype_wrappers.rs)
Defines `UtcTimestamp` and `LocalTimestamp` newtype wrappers with conversion functions.

### [`provider_trait.rs`](provider_trait.rs)
Updated `ForecastProvider` trait with timezone parameter in `fetch_weather_data()` method.

### [`timezone_config.rs`](timezone_config.rs)
Configuration structures for timezone handling including parsing and validation.

### [`weather_data_point.rs`](weather_data_point.rs)
Updated `WeatherDataPoint` structure using `LocalTimestamp` instead of `DateTime<Utc>`.

## Usage

These contracts are **reference implementations** that define the expected API surface. The actual implementation will be in the [`src/`](../../../src/) directory, but should match these signatures exactly.

## Integration Points

1. **Provider Implementations**: All providers must update `fetch_weather_data()` to accept `target_tz: Tz` parameter
2. **Main Application**: Must create `TimezoneConfig` from CLI args and pass to providers
3. **Serialization**: `LocalTimestamp` handles formatting, no custom serializers needed
4. **Testing**: Tests can now verify timezone conversion without JSON serialization

## Validation

All types include validation logic:
- Timezone identifiers validated via `str::parse::<Tz>()`
- Timestamps validated for reasonable forecast ranges
- Error messages follow structured format from research

## References

- [Feature Spec](../spec.md)
- [Data Model](../data-model.md)
- [Research Findings](../research.md)
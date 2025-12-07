# Project Architecture Rules (Non-Obvious Only)

## Provider Registration Anti-Pattern

The provider system uses a 3-location registration pattern instead of a centralized registry:
1. [`create_provider()`](../../src/main.rs:39) - factory function with match on provider name
2. [`run()`](../../src/main.rs:130) - API key retrieval with separate match
3. [`validate_provider()`](../../src/args.rs:77) - validation with third match

This violates DRY and makes adding providers error-prone. A registry pattern would be more maintainable.

## Unit Conversion Happens at Different Layers

- StormGlass: Unit conversion (m/s → knots) happens in [`transform_hour()`](../../src/providers/stormglass.rs:95) during provider transformation
- OpenWeatherMap: No unit conversion - raw m/s values pass through
- Timezone conversion: Happens in serde serialization via [`serialize_time_jerusalem()`](../../src/forecast_provider.rs:7), NOT in transform layer

This mixed-layer approach means unit handling is inconsistent across the architecture.

## Metadata Now Provider-Specific

[`create_units_map()`](../../src/main.rs:45) now accepts `provider_name` parameter and returns provider-specific units:
- StormGlass: Reports "knots" (matches conversion in code)
- OpenWeatherMap: Reports "m/s" (matches raw API values)
- Unknown providers: Default to "m/s"

This ensures metadata accurately reflects the actual units in the data.

## Error Handling Inconsistency

- [`StormGlassProvider`](../../src/providers/stormglass.rs:15) uses custom `thiserror` errors with HTTP status code mapping
- [`OpenWeatherMapProvider`](../../src/providers/openweathermap.rs:104) uses generic `anyhow` errors
- This asymmetry makes error handling patterns differ between providers

## Hard-coded Business Logic

The 7-day constraint in [`args.rs:64`](../../src/args.rs:64) is a business rule embedded in validation logic, not a technical limitation. Should be configurable or at least documented as policy rather than constraint.

## No Dependency Injection

Providers are instantiated with only an API key in [`create_provider()`](../../src/main.rs:39). HTTP client, coordinates, and date ranges are passed at call-time. This makes testing difficult without actual API keys.
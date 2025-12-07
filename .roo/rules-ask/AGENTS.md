# Project Documentation Rules (Non-Obvious Only)

## Provider Architecture Has 3-Way Split

When explaining provider architecture, note that provider registration happens in 3 separate locations in [`main.rs`](../../src/main.rs:1), NOT in a centralized registry:
1. Provider instantiation in [`create_provider()`](../../src/main.rs:39)
2. API key retrieval logic in [`run()`](../../src/main.rs:130)
3. Provider validation in [`validate_provider()`](../../src/args.rs:77)

This is counterintuitive - most systems use a registry pattern.

## Wind Speed Units Are Provider-Dependent

When explaining output data:
- Wind speeds from StormGlass provider are in **knots** (converted via `MS_TO_KNOTS = 1.94384`)
- Wind speeds from OpenWeatherMap provider are in **m/s** (no conversion)
- The [`create_units_map()`](../../src/main.rs:45) now correctly reports units based on provider:
  - StormGlass metadata: "knots"
  - OpenWeatherMap metadata: "m/s"
  - Unknown providers: Default to "m/s"
- Metadata now accurately reflects actual data units

## Timezone Conversion Is Hidden in Serialization

Timestamps appear in Asia/Jerusalem timezone in JSON output, but this conversion happens in [`serialize_time_jerusalem()`](../../src/forecast_provider.rs:7) during serialization, NOT in provider transform logic. The internal `DateTime<Utc>` values remain in UTC throughout the code.

## Location Not Configurable

Despite CLI accepting `--days-ahead` and `--provider` flags, coordinates are hard-coded in [`main.rs:155`](../../src/main.rs:155). There is NO `--lat`/`--lng` flag option.

## Date Range Constraint Is Arbitrary

The 7-day limit (`days_ahead + first_day_offset <= 7`) in [`args.rs:64`](../../src/args.rs:64) is described as for "reliable forecasts" but is actually a business rule, not a technical limitation of the APIs.
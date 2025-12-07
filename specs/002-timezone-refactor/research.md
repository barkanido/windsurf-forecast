# Research: Timezone Conversion Architecture Refactor

**Feature**: 002-timezone-refactor  
**Date**: 2025-12-07  
**Status**: Complete

## Overview

This document consolidates research findings for moving timezone conversion from the serialization layer to the transform layer in the weather forecast application.

## Research Questions & Findings

### 1. Timezone Library Selection

**Question**: Which Rust timezone library should be used for timezone handling?

**Decision**: Use `chrono-tz` crate (version 0.8, already in dependencies)

**Rationale**:
- Already included in project dependencies (see [`Cargo.toml:12`](../../Cargo.toml:12))
- Provides comprehensive IANA timezone database support
- Handles daylight saving time (DST) transitions automatically
- Supports string-to-`Tz` parsing via `str::parse::<Tz>()` for runtime validation
- Integrates seamlessly with `chrono::DateTime` types already used throughout codebase
- Well-maintained with active community support

**Alternatives Considered**:
- `tz-rs`: Lower-level, requires more manual DST handling
- `jiff`: Newer library with less ecosystem maturity
- Manual offset handling: Would require implementing DST logic ourselves (error-prone)

**Implementation Notes**:
```rust
use chrono_tz::Tz;

// Parse user input
let tz: Tz = "America/New_York".parse()
    .map_err(|_| anyhow!("Invalid timezone identifier"))?;

// Convert UTC to target timezone
let local_time = tz.from_utc_datetime(&utc_time.naive_utc());
```

---

### 2. Default Timezone Behavior

**Question**: What should the default timezone be when no user configuration is provided?

**Decision**: Default to UTC with a warning message about using the default

**Rationale**:
- UTC is the universal standard for timestamps, avoiding regional bias
- Warning message educates users about timezone configuration options
- Explicit warning prevents silent assumptions about timezone
- Maintains predictable behavior (unlike defaulting to system timezone which varies)
- Aligns with existing internal UTC storage pattern

**Implementation**:
```rust
let timezone = config.timezone.unwrap_or_else(|| {
    eprintln!("Warning: No timezone configured. Using UTC as default.");
    eprintln!("Set timezone via --timezone flag or FORECAST_TIMEZONE environment variable.");
    eprintln!("Example: --timezone \"America/New_York\"");
    chrono_tz::UTC
});
```

**Alternatives Considered**:
- System timezone: Platform-dependent, harder to test, may surprise users
- Hard-fail without timezone: Too strict for CLI tool, poor UX
- Keep Asia/Jerusalem default: Maintains geographic bias, limits usability

---

### 3. Type System Design for Timezone Safety

**Question**: How should the type system distinguish between UTC and timezone-converted timestamps?

**Decision**: Use newtype wrappers (`UtcTimestamp` and `LocalTimestamp`) for compile-time distinction

**Rationale**:
- Newtype pattern provides zero-cost abstraction (no runtime overhead)
- Compiler enforces correct timezone usage at compile time
- Makes timezone state explicit in function signatures
- Prevents accidental mixing of UTC and local timestamps
- Common Rust idiom for type-level guarantees

**Implementation**:
```rust
/// Wrapper for UTC timestamps (from API responses)
#[derive(Debug, Clone, Copy)]
pub struct UtcTimestamp(pub DateTime<Utc>);

/// Wrapper for timezone-converted timestamps (for output)
#[derive(Debug, Clone, Serialize)]
pub struct LocalTimestamp(pub DateTime<Tz>);

// Function signatures make timezone contract explicit
fn transform_weather_data(
    raw_data: RawApiResponse,
    target_tz: Tz
) -> Result<Vec<WeatherDataPoint>> {
    // UTC parsing
    let utc_timestamp = UtcTimestamp(parse_utc(&raw_data.time)?);
    
    // Explicit conversion to target timezone
    let local_timestamp = convert_to_local(utc_timestamp, target_tz);
    
    Ok(vec![WeatherDataPoint { time: local_timestamp, ... }])
}
```

**Alternatives Considered**:
- Generic `DateTime<Tz>` throughout: Loses compile-time guarantees, easy to mix timezones
- String-based timestamps: No type safety, manual parsing everywhere
- Phantom types: More complex than needed for this use case
- Enum variant: Runtime overhead, less ergonomic than newtype

---

### 4. Configuration Interface Design

**Question**: What are the exact naming conventions for timezone configuration (CLI flag and environment variable)?

**Decision**: CLI: `--timezone` (with `--tz` alias), ENV: `FORECAST_TIMEZONE`

**Rationale**:
- `--timezone` is explicit and self-documenting
- `--tz` alias provides convenience for frequent users
- `FORECAST_TIMEZONE` follows existing `FORECAST_*` pattern in codebase
- Prefix prevents namespace collision with system timezone variables
- CLI takes precedence over environment variable (explicit override)

**Implementation**:
```rust
#[derive(Parser)]
pub struct Args {
    /// Timezone for displaying timestamps (e.g., "UTC", "America/New_York", "Asia/Jerusalem")
    /// Overrides timezone from config file
    #[arg(long, short = 'z', value_name = "TIMEZONE")]
    pub timezone: Option<String>,
}

// In config loading
let timezone = args.timezone
    .or_else(|| env::var("FORECAST_TIMEZONE").ok())
    .map(|s| s.parse::<Tz>())
    .transpose()?;
```

**Alternatives Considered**:
- `--tz` only: Less discoverable in help text
- `TZ` environment variable: Conflicts with POSIX standard system variable
- `TIMEZONE` only: Too generic, potential conflicts
- Separate `--output-timezone`: More verbose without added clarity

---

### 5. Error Message Format

**Question**: What format should timezone conversion error messages use?

**Decision**: Use structured format: "Timezone conversion failed: cannot convert timestamp '{timestamp}' from {source_tz} to {target_tz}: {reason}"

**Rationale**:
- Provides all necessary debugging information in one message
- Structured format is scannable and parseable
- Shows exact values involved in failure (timestamp, timezones)
- Includes underlying reason for actionable debugging
- Consistent with "Error Transparency" constitution principle

**Implementation**:
```rust
fn convert_timezone(
    utc: UtcTimestamp,
    target_tz: Tz
) -> Result<LocalTimestamp> {
    let local = target_tz
        .from_utc_datetime(&utc.0.naive_utc());
    
    // Validation (e.g., for ambiguous DST transitions)
    if is_ambiguous(&local) {
        anyhow::bail!(
            "Timezone conversion failed: cannot convert timestamp '{}' from {} to {}: ambiguous during DST transition",
            utc.0,
            "UTC",
            target_tz
        );
    }
    
    Ok(LocalTimestamp(local))
}
```

**Alternatives Considered**:
- Minimal error: "Invalid timezone conversion" - not actionable
- JSON error structure: Overkill for CLI application
- Stack trace only: Buries important information
- No timestamp details: Harder to debug specific failures

---

## Architecture Patterns

### Thread-Local State Elimination

**Current Problem**: 
- [`serialize_time_with_tz()`](../../src/forecast_provider.rs:8) uses thread-local storage to access timezone during serialization
- Hidden dependency makes testing difficult
- Potential concurrency issues with thread-local mutation
- Serialization layer should be pure formatting, not business logic

**Solution**:
- Move timezone conversion to transform layer (after API parsing, before serialization)
- Pass timezone explicitly as parameter to provider transform functions
- Serialization only formats already-converted timestamps
- Thread-local state completely removed

**Before**:
```rust
// Hidden thread-local state
thread_local! {
    static TIMEZONE: RefCell<Tz> = RefCell::new(chrono_tz::UTC);
}

// Must set state before serialization
set_serialization_timezone(user_tz);
let json = serde_json::to_string(&weather_data)?;
```

**After**:
```rust
// Explicit parameter passing
async fn fetch_weather_data(
    &self,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    lat: f64,
    lng: f64,
    target_tz: Tz  // NEW: explicit timezone parameter
) -> Result<Vec<WeatherDataPoint>> {
    // Conversion happens HERE in transform layer
    let local_time = LocalTimestamp(
        target_tz.from_utc_datetime(&utc_time.naive_utc())
    );
    // ...
}
```

---

### DST Handling

**Challenge**: Daylight saving time transitions create ambiguous or invalid timestamps

**Solution**: `chrono-tz` handles DST automatically, but we must document edge cases

**DST Transition Types**:
1. **Spring Forward** (gap): 2:00 AM → 3:00 AM
   - Timestamps like 2:30 AM don't exist
   - `chrono-tz` maps to 3:30 AM (after transition)

2. **Fall Back** (overlap): 2:00 AM → 1:00 AM
   - Timestamps like 1:30 AM occur twice
   - `chrono-tz` uses standard time (second occurrence)

**Testing Strategy**:
```rust
#[test]
fn test_dst_spring_forward() {
    // Test timestamp that falls in DST gap
    let utc = Utc.ymd(2024, 3, 10).and_hms(7, 30, 0); // 2:30 AM EST (doesn't exist)
    let tz: Tz = "America/New_York".parse().unwrap();
    
    let local = tz.from_utc_datetime(&utc.naive_utc());
    // Should map to 3:30 AM EDT
    assert_eq!(local.hour(), 3);
}
```

---

## Best Practices

### 1. Configuration Precedence

**Order** (highest to lowest):
1. Command-line `--timezone` flag
2. Environment variable `FORECAST_TIMEZONE`
3. Config file setting (if implemented)
4. Default to UTC with warning

### 2. Validation Strategy

- Validate timezone identifier at configuration parsing (fail fast)
- Use `str::parse::<Tz>()` for validation
- Provide helpful error with list of valid identifiers on failure
- Example valid identifiers in error message

### 3. Testing Approach

**Unit Tests** (no serialization):
```rust
#[test]
fn test_timezone_conversion_direct() {
    let utc = UtcTimestamp(Utc.ymd(2024, 1, 1).and_hms(12, 0, 0));
    let tz: Tz = "America/New_York".parse().unwrap();
    
    let local = convert_timezone(utc, tz).unwrap();
    assert_eq!(local.0.hour(), 7); // UTC-5 in winter
}
```

**Integration Tests** (full pipeline):
- Test with real provider API responses
- Verify JSON output format unchanged
- Test multiple timezones including DST boundaries

### 4. Migration Strategy

**Phase 1**: Add newtype wrappers, keep existing serialization working
**Phase 2**: Update provider transforms to accept timezone parameter  
**Phase 3**: Move conversion logic from serialization to transform layer
**Phase 4**: Remove thread-local state and old serializer
**Phase 5**: Update tests to use new explicit conversion

---

## Dependencies

### Required Crate Features

Already in [`Cargo.toml`](../../Cargo.toml:1):
- `chrono = { version = "0.4", features = ["serde"] }` - Core datetime types
- `chrono-tz = "0.8"` - IANA timezone database
- `serde = { version = "1.0", features = ["derive"] }` - Serialization
- `clap = { version = "4.4", features = ["derive"] }` - CLI parsing

No new dependencies required.

---

## Success Metrics

Based on spec success criteria:

1. **Traceability**: Developers can locate timezone conversion in <30 seconds (grep for `convert_timezone` in transform layer)
2. **Testability**: Unit tests verify conversion without JSON (100% serialization-independent)
3. **Type Safety**: Compiler prevents `UtcTimestamp` where `LocalTimestamp` expected
4. **Performance**: Timezone conversion overhead <100ms per forecast
5. **User Control**: Support any valid IANA timezone identifier
6. **Error Quality**: Invalid timezone produces actionable message in <100ms

---

## Open Questions

None - all research questions resolved per spec clarifications.

---

## References

- Feature Spec: [`specs/002-timezone-refactor/spec.md`](spec.md)
- Constitution: [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md)
- Current Implementation: [`src/forecast_provider.rs`](../../src/forecast_provider.rs)
- Provider Examples: [`src/providers/stormglass.rs`](../../src/providers/stormglass.rs)
- chrono-tz docs: https://docs.rs/chrono-tz/latest/chrono_tz/
- IANA timezone database: https://www.iana.org/time-zones
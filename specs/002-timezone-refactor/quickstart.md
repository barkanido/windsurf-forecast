# Quickstart: Timezone Conversion Architecture Refactor

**Feature**: 002-timezone-refactor  
**Date**: 2025-12-07  
**Status**: Ready for Implementation

## Overview

This guide provides step-by-step instructions for implementing the timezone refactor. Follow these steps sequentially, testing after each major change.

## Prerequisites

- Rust 1.75+ installed
- Existing codebase at commit with provider registry implementation
- All existing tests passing
- Familiarity with `chrono` and `chrono-tz` crates

## Implementation Phases

### Phase 1: Add Newtype Wrappers (Non-Breaking)

**Duration**: ~30 minutes  
**Goal**: Introduce type-safe timestamp wrappers without breaking existing code

#### Step 1.1: Add newtype wrappers to forecast_provider.rs

Location: [`src/forecast_provider.rs`](../../src/forecast_provider.rs)

```rust
// Add at top of file after existing imports
use chrono_tz::Tz;

// Add before WeatherDataPoint definition
/// Newtype wrapper for UTC timestamps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcTimestamp(pub DateTime<Utc>);

impl UtcTimestamp {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
    
    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }
    
    pub fn from_rfc3339(s: &str) -> Result<Self> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| anyhow!("Failed to parse UTC timestamp: {}", e))?
            .with_timezone(&Utc);
        Ok(Self(dt))
    }
}

/// Newtype wrapper for timezone-converted timestamps
#[derive(Debug, Clone)]
pub struct LocalTimestamp {
    inner: DateTime<Tz>,
}

impl LocalTimestamp {
    pub fn new(dt: DateTime<Tz>) -> Self {
        Self { inner: dt }
    }
    
    pub fn inner(&self) -> DateTime<Tz> {
        self.inner
    }
}

impl Serialize for LocalTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = self.inner.format("%Y-%m-%d %H:%M").to_string();
        serializer.serialize_str(&formatted)
    }
}

/// Convert UTC timestamp to target timezone
pub fn convert_timezone(utc: UtcTimestamp, target_tz: Tz) -> Result<LocalTimestamp> {
    let local = target_tz.from_utc_datetime(&utc.0.naive_utc());
    Ok(LocalTimestamp::new(local))
}
```

#### Step 1.2: Verify compilation

```bash
cargo check
```

**Expected**: No errors (newtypes are defined but not yet used)

---

### Phase 2: Add Timezone Configuration (Non-Breaking)

**Duration**: ~45 minutes  
**Goal**: Add timezone configuration structures and CLI arguments

#### Step 2.1: Add timezone configuration to config.rs

Location: [`src/config.rs`](../../src/config.rs)

```rust
use chrono_tz::Tz;
use std::env;

/// Configuration for timezone handling
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    pub timezone: Tz,
    pub explicit: bool,
}

impl TimezoneConfig {
    pub fn explicit(tz: Tz) -> Self {
        Self {
            timezone: tz,
            explicit: true,
        }
    }
    
    pub fn default_utc() -> Self {
        Self {
            timezone: chrono_tz::UTC,
            explicit: false,
        }
    }
    
    pub fn from_string(s: &str) -> Result<Self> {
        let tz = s.parse::<Tz>()
            .map_err(|_| anyhow!(
                "Invalid timezone identifier: '{}'\n\
                 Examples: UTC, America/New_York, Europe/London, Asia/Jerusalem",
                s
            ))?;
        Ok(Self::explicit(tz))
    }
    
    pub fn load_with_precedence(cli_tz: Option<String>) -> Result<Self> {
        if let Some(tz_str) = cli_tz {
            return Self::from_string(&tz_str);
        }
        
        if let Ok(tz_str) = env::var("FORECAST_TIMEZONE") {
            return Self::from_string(&tz_str);
        }
        
        Ok(Self::default_utc())
    }
    
    pub fn display_default_warning(&self) {
        if !self.explicit {
            eprintln!("Warning: No timezone configured. Using UTC as default.");
            eprintln!("Set timezone via --timezone flag or FORECAST_TIMEZONE environment variable.");
            eprintln!("Example: --timezone \"America/New_York\"");
        }
    }
}
```

#### Step 2.2: Add CLI flag to args.rs

Location: [`src/args.rs`](../../src/args.rs:31)

Add to Args struct:
```rust
/// Timezone for displaying timestamps (e.g., "UTC", "America/New_York", "Asia/Jerusalem")
/// Overrides FORECAST_TIMEZONE environment variable
#[arg(long, short = 'z', value_name = "TIMEZONE")]
pub timezone: Option<String>,
```

#### Step 2.3: Verify compilation and help text

```bash
cargo check
cargo run -- --help
```

**Expected**: `--timezone` flag appears in help text

---

### Phase 3: Update ForecastProvider Trait (BREAKING)

**Duration**: ~1 hour  
**Goal**: Add timezone parameter to provider trait

#### Step 3.1: Update trait signature

Location: [`src/forecast_provider.rs`](../../src/forecast_provider.rs:75)

```rust
#[async_trait]
pub trait ForecastProvider: Send + Sync {
    fn name(&self) -> &str;
    
    fn get_api_key() -> Result<String>
    where
        Self: Sized;
    
    async fn fetch_weather_data(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        lat: f64,
        lng: f64,
        target_tz: Tz,  // NEW PARAMETER
    ) -> Result<Vec<WeatherDataPoint>>;
}
```

#### Step 3.2: Verify compilation fails

```bash
cargo check
```

**Expected**: Compilation errors in provider implementations (this is correct!)

---

### Phase 4: Update StormGlass Provider

**Duration**: ~45 minutes  
**Goal**: Update StormGlass to use new timezone system

#### Step 4.1: Update fetch_weather_data signature

Location: [`src/providers/stormglass.rs`](../../src/providers/stormglass.rs)

Find the `fetch_weather_data` method and add `target_tz: Tz` parameter.

#### Step 4.2: Update transform logic

In the response parsing section, modify timestamp handling:

```rust
// OLD:
let time = DateTime::parse_from_rfc3339(&hour.time)
    .map_err(|e| StormGlassAPIError::InvalidTimestamp(e.to_string()))?
    .with_timezone(&Utc);

// NEW:
let utc = UtcTimestamp::from_rfc3339(&hour.time)
    .map_err(|e| StormGlassAPIError::InvalidTimestamp(e.to_string()))?;

let local = convert_timezone(utc, target_tz)?;
```

#### Step 4.3: Update WeatherDataPoint construction

```rust
// OLD:
WeatherDataPoint {
    time,  // DateTime<Utc>
    // ...
}

// NEW:
WeatherDataPoint {
    time: local,  // LocalTimestamp
    // ...
}
```

#### Step 4.4: Verify compilation

```bash
cargo check
```

---

### Phase 5: Update OpenWeatherMap Provider

**Duration**: ~45 minutes  
**Goal**: Update OpenWeatherMap to use new timezone system

Follow same pattern as Phase 4:
1. Add `target_tz: Tz` parameter to `fetch_weather_data`
2. Update timestamp parsing to use `UtcTimestamp`
3. Convert to `LocalTimestamp` using `convert_timezone()`
4. Update `WeatherDataPoint` construction

Location: [`src/providers/openweathermap.rs`](../../src/providers/openweathermap.rs)

---

### Phase 6: Update WeatherDataPoint Structure (BREAKING)

**Duration**: ~30 minutes  
**Goal**: Change time field to use LocalTimestamp

#### Step 6.1: Update WeatherDataPoint definition

Location: [`src/forecast_provider.rs`](../../src/forecast_provider.rs:44)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDataPoint {
    // REMOVE: #[serde(serialize_with = "serialize_time_with_tz")]
    pub time: LocalTimestamp,  // CHANGED from DateTime<Utc>
    
    // ... rest unchanged
}
```

#### Step 6.2: Remove old serialization functions

Delete these functions from forecast_provider.rs:
- `serialize_time_with_tz()`
- `set_serialization_timezone()`

#### Step 6.3: Verify compilation

```bash
cargo check
```

**Expected**: Should compile if all providers updated correctly

---

### Phase 7: Update Main Application

**Duration**: ~30 minutes  
**Goal**: Wire timezone config through main application

#### Step 7.1: Load timezone configuration

Location: [`src/main.rs`](../../src/main.rs)

Add after argument parsing:
```rust
let timezone_config = TimezoneConfig::load_with_precedence(args.timezone)?;
timezone_config.display_default_warning();
```

#### Step 7.2: Pass timezone to provider

Update provider call:
```rust
// OLD:
let weather_data = provider
    .fetch_weather_data(start, end, lat, lng)
    .await?;

// NEW:
let weather_data = provider
    .fetch_weather_data(start, end, lat, lng, timezone_config.timezone)
    .await?;
```

#### Step 7.3: Remove old serialization setup

Delete any calls to `set_serialization_timezone()`.

---

### Phase 8: Testing & Validation

**Duration**: ~1 hour  
**Goal**: Verify all functionality works correctly

#### Step 8.1: Run all checks

```bash
# Check compilation
cargo check

# Build debug version
cargo build

# Run clippy
cargo clippy

# Run with UTC (should show warning)
cargo run -- --provider stormglass --days-ahead 2

# Run with explicit timezone (no warning)
cargo run -- --provider stormglass --days-ahead 2 --timezone "America/New_York"

# Test environment variable
set FORECAST_TIMEZONE=Europe/London
cargo run -- --provider openweathermap --days-ahead 3

# Test invalid timezone (should show helpful error)
cargo run -- --timezone "Invalid/Zone"
```

#### Step 8.2: Verify JSON output format

Compare output with previous version - format should be identical:
```json
{
  "time": "2024-01-15 14:30",
  "airTemperature": 20.5,
  ...
}
```

#### Step 8.3: Test timezone conversion

```bash
# Morning in UTC should be different in other timezones
cargo run -- --timezone "UTC" --days-ahead 1
cargo run -- --timezone "America/New_York" --days-ahead 1
cargo run -- --timezone "Asia/Tokyo" --days-ahead 1
```

Verify timestamps differ by appropriate offsets.

---

## Common Issues & Solutions

### Issue: "Cannot find value `Tz` in this scope"

**Solution**: Add import at top of file:
```rust
use chrono_tz::Tz;
```

### Issue: "Method `from_rfc3339` not found"

**Solution**: Ensure you're using `UtcTimestamp::from_rfc3339()` not `DateTime::from_rfc3339()`

### Issue: Clippy warnings about unused imports

**Solution**: Remove old timezone-related imports that are no longer needed

### Issue: Tests failing with type mismatches

**Solution**: Update test code to pass timezone parameter:
```rust
let tz: Tz = "UTC".parse().unwrap();
provider.fetch_weather_data(start, end, lat, lng, tz).await?;
```

---

## Rollback Plan

If issues arise, revert in reverse order:

1. Revert main.rs changes (Phase 7)
2. Revert WeatherDataPoint changes (Phase 6)
3. Revert provider implementations (Phases 4-5)
4. Revert trait signature (Phase 3)
5. Revert config additions (Phase 2)
6. Revert newtype wrappers (Phase 1)

Each phase should be a clean git commit for easy rollback.

---

## Success Checklist

- [ ] `cargo check` passes without errors
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo build` completes successfully
- [ ] Application runs with `--timezone` flag
- [ ] Application runs with `FORECAST_TIMEZONE` env var
- [ ] Application shows warning when no timezone configured
- [ ] Invalid timezone shows helpful error message
- [ ] JSON output format unchanged from previous version
- [ ] Timestamps correctly reflect specified timezone
- [ ] Multiple providers work with new system
- [ ] No thread-local state remains in codebase

---

## Performance Expectations

- Timezone conversion overhead: <1ms per data point
- Total application startup time increase: <50ms
- No measurable impact on API request time
- Memory usage unchanged

---

## Next Steps

After successful implementation:
1. Update `AGENTS.md` with new timezone handling patterns
2. Update `README.md` with timezone configuration instructions
3. Consider adding integration tests for DST transitions
4. Document any provider-specific timezone quirks

---

## References

- [Feature Spec](spec.md)
- [Data Model](data-model.md)
- [Research](research.md)
- [Contracts](contracts/)
- [Constitution](.specify/memory/constitution.md)
# Data Model: Timezone Conversion Architecture Refactor

**Feature**: 002-timezone-refactor  
**Date**: 2025-12-07  
**Status**: Design Complete

## Overview

This document defines the data structures and their relationships for timezone-aware weather data processing. The refactor introduces newtype wrappers for compile-time timezone safety and modifies existing structures to support user-configurable timezone conversion.

---

## Core Entities

### 1. UtcTimestamp (NEW)

**Purpose**: Type-safe wrapper for UTC timestamps received from weather API providers

**Definition**:
```rust
/// Newtype wrapper for UTC timestamps
/// Used internally to represent timestamps as received from weather APIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcTimestamp(pub DateTime<Utc>);

impl UtcTimestamp {
    /// Create from DateTime<Utc>
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
    
    /// Get the inner DateTime<Utc>
    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }
}
```

**Fields**:
- `0: DateTime<Utc>` - Inner UTC datetime value

**Relationships**:
- Source: Weather provider API responses (after parsing)
- Transform: Converted to `LocalTimestamp` via timezone conversion
- Usage: Internal to provider transform functions

**Validation Rules**:
- Must be a valid UTC datetime
- Must be within reasonable forecast range (not historical dates from ancient times)
- Should be within 0-14 days from current time (typical forecast window)

**State Transitions**:
```
API Response String → Parse → UtcTimestamp → Convert → LocalTimestamp → Serialize
```

---

### 2. LocalTimestamp (NEW)

**Purpose**: Type-safe wrapper for timezone-converted timestamps ready for output

**Definition**:
```rust
/// Newtype wrapper for timezone-converted timestamps
/// Used in output structures after conversion to user's target timezone
#[derive(Debug, Clone, Serialize)]
pub struct LocalTimestamp {
    #[serde(skip)]
    inner: DateTime<Tz>,
}

impl LocalTimestamp {
    /// Create from DateTime<Tz> (timezone-aware datetime)
    pub fn new(dt: DateTime<Tz>) -> Self {
        Self { inner: dt }
    }
    
    /// Get the inner DateTime<Tz>
    pub fn inner(&self) -> DateTime<Tz> {
        self.inner
    }
}

// Custom serialization to maintain "YYYY-MM-DD HH:MM" format
impl Serialize for LocalTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let formatted = self.inner.format("%Y-%m-%d %H:%M").to_string();
        serializer.serialize_str(&formatted)
    }
}
```

**Fields**:
- `inner: DateTime<Tz>` - Timezone-aware datetime in user's target timezone

**Relationships**:
- Source: Converted from `UtcTimestamp` in provider transform layer
- Usage: Embedded in `WeatherDataPoint` for output
- Serialization: Custom formatter maintains backward-compatible format

**Validation Rules**:
- Must have valid timezone information
- Timezone must be user-specified or default (UTC)
- DST transitions handled by chrono-tz library

**Serialization Format**:
```json
"2024-01-15 14:30"
```
(Note: Not ISO 8601, maintains backward compatibility)

---

### 3. WeatherDataPoint (MODIFIED)

**Purpose**: Common data structure for weather measurements across all providers

**Current Definition** (from [`src/forecast_provider.rs:44`](../../src/forecast_provider.rs:44)):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDataPoint {
    #[serde(serialize_with = "serialize_time_with_tz")]
    pub time: DateTime<Utc>,  // WILL BE CHANGED
    
    #[serde(rename = "airTemperature", skip_serializing_if = "Option::is_none")]
    pub air_temperature: Option<f64>,
    
    #[serde(rename = "windSpeed", skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,
    
    #[serde(rename = "windDirection", skip_serializing_if = "Option::is_none")]
    pub wind_direction: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gust: Option<f64>,
    
    #[serde(rename = "swellHeight", skip_serializing_if = "Option::is_none")]
    pub swell_height: Option<f64>,
    
    #[serde(rename = "swellPeriod", skip_serializing_if = "Option::is_none")]
    pub swell_period: Option<f64>,
    
    #[serde(rename = "swellDirection", skip_serializing_if = "Option::is_none")]
    pub swell_direction: Option<f64>,
    
    #[serde(rename = "waterTemperature", skip_serializing_if = "Option::is_none")]
    pub water_temperature: Option<f64>,
}
```

**New Definition** (after refactor):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherDataPoint {
    // CHANGED: Now uses LocalTimestamp instead of DateTime<Utc>
    // No more custom serializer - LocalTimestamp handles formatting
    pub time: LocalTimestamp,
    
    #[serde(rename = "airTemperature", skip_serializing_if = "Option::is_none")]
    pub air_temperature: Option<f64>,
    
    #[serde(rename = "windSpeed", skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f64>,
    
    #[serde(rename = "windDirection", skip_serializing_if = "Option::is_none")]
    pub wind_direction: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gust: Option<f64>,
    
    #[serde(rename = "swellHeight", skip_serializing_if = "Option::is_none")]
    pub swell_height: Option<f64>,
    
    #[serde(rename = "swellPeriod", skip_serializing_if = "Option::is_none")]
    pub swell_period: Option<f64>,
    
    #[serde(rename = "swellDirection", skip_serializing_if = "Option::is_none")]
    pub swell_direction: Option<f64>,
    
    #[serde(rename = "waterTemperature", skip_serializing_if = "Option::is_none")]
    pub water_temperature: Option<f64>,
}
```

**Key Changes**:
- `time` field changes from `DateTime<Utc>` to `LocalTimestamp`
- Removes `#[serde(serialize_with = "serialize_time_with_tz")]` attribute
- Serialization now handled by `LocalTimestamp`'s implementation

**Relationships**:
- Contains: One `LocalTimestamp` representing the forecast time
- Created by: Provider transform functions
- Used in: JSON output generation

**Validation Rules**:
- All measurement fields are optional (different providers support different metrics)
- Timestamp must be present (not optional)
- Measurements should be within reasonable ranges for weather data

---

### 4. TimezoneConfig (NEW)

**Purpose**: Configuration structure for user's timezone preferences

**Definition**:
```rust
/// Configuration for timezone handling
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    /// Target timezone for output timestamps
    pub timezone: Tz,
    
    /// Whether timezone was explicitly set by user (vs default)
    pub explicit: bool,
}

impl TimezoneConfig {
    /// Create from explicit user input
    pub fn explicit(tz: Tz) -> Self {
        Self {
            timezone: tz,
            explicit: true,
        }
    }
    
    /// Create default (UTC) with warning flag
    pub fn default_utc() -> Self {
        Self {
            timezone: chrono_tz::UTC,
            explicit: false,
        }
    }
    
    /// Parse from string identifier
    pub fn from_string(s: &str) -> Result<Self> {
        let tz = s.parse::<Tz>()
            .map_err(|_| anyhow!(
                "Invalid timezone identifier: '{}'\n\
                 Examples of valid identifiers:\n\
                 - UTC\n\
                 - America/New_York\n\
                 - Europe/London\n\
                 - Asia/Jerusalem\n\
                 See https://en.wikipedia.org/wiki/List_of_tz_database_time_zones",
                s
            ))?;
        Ok(Self::explicit(tz))
    }
}
```

**Fields**:
- `timezone: Tz` - The target timezone for conversion
- `explicit: bool` - Whether user explicitly configured this (for warning logic)

**Relationships**:
- Source: CLI arguments (`--timezone`) or environment variable (`FORECAST_TIMEZONE`)
- Usage: Passed to provider `fetch_weather_data()` methods
- Default: UTC with warning

**Validation Rules**:
- Timezone identifier must be valid IANA timezone string
- Parsing failure produces actionable error message

**State Transitions**:
```
CLI/ENV String → Parse → TimezoneConfig → Pass to Provider → Used in Transform
                   ↓
               Invalid
                   ↓
            Error Message
```

---

### 5. ForecastProvider Trait (MODIFIED)

**Purpose**: Common interface for all weather data providers

**Current Signature** (from [`src/forecast_provider.rs:75`](../../src/forecast_provider.rs:75)):
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
    ) -> Result<Vec<WeatherDataPoint>>;
}
```

**New Signature** (after refactor):
```rust
#[async_trait]
pub trait ForecastProvider: Send + Sync {
    fn name(&self) -> &str;
    
    fn get_api_key() -> Result<String>
    where
        Self: Sized;
    
    // CHANGED: Added target_tz parameter
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

**Key Changes**:
- Adds `target_tz: Tz` parameter to `fetch_weather_data()` method
- Implementations must convert UTC timestamps to target timezone
- Return type unchanged but `WeatherDataPoint.time` is now `LocalTimestamp`

**Implementation Contract**:
1. Parse API response timestamps to `UtcTimestamp`
2. Convert to `LocalTimestamp` using `target_tz`
3. Construct `WeatherDataPoint` with `LocalTimestamp`
4. No timezone logic in serialization

---

## Data Flow Diagram

```
┌─────────────────┐
│  User Input     │
│  (CLI/ENV)      │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  Parse to TimezoneConfig│
│  (validate timezone)    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Provider API Call      │
│  (fetch raw JSON)       │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Parse to UtcTimestamp  │
│  (API response times)   │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Convert Timezone       │  ← CORE CHANGE: Moved here from serialization
│  UtcTimestamp → Local   │
│  using target_tz        │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Construct DataPoint    │
│  with LocalTimestamp    │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Serialize to JSON      │  ← Simple formatting, no conversion
│  LocalTimestamp format  │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Output JSON to stdout  │
└─────────────────────────┘
```

---

## Entity Relationships

```
┌─────────────────┐
│ TimezoneConfig  │
│                 │
│ - timezone: Tz  │
│ - explicit: bool│
└────────┬────────┘
         │
         │ passed to
         ▼
┌──────────────────────────┐
│  ForecastProvider        │
│  (trait)                 │
│                          │
│  fetch_weather_data(..., │
│    target_tz: Tz)        │
└────────┬─────────────────┘
         │
         │ returns
         ▼
┌──────────────────────────┐
│  Vec<WeatherDataPoint>   │
│                          │
│  - time: LocalTimestamp  │◄──── Contains
│  - air_temperature       │
│  - wind_speed            │
│  - ...                   │
└──────────────────────────┘
         │
         │ time field
         ▼
┌──────────────────────────┐
│  LocalTimestamp          │
│                          │
│  - inner: DateTime<Tz>   │
└──────────────────────────┘
         ▲
         │ converted from
         │
┌──────────────────────────┐
│  UtcTimestamp            │
│                          │
│  - 0: DateTime<Utc>      │
└──────────────────────────┘
```

---

## Migration Impact

### Breaking Changes

1. **`ForecastProvider` trait signature change**
   - All provider implementations must add `target_tz` parameter
   - Affects: [`StormGlassProvider`](../../src/providers/stormglass.rs), [`OpenWeatherMapProvider`](../../src/providers/openweathermap.rs)

2. **`WeatherDataPoint.time` field type change**
   - Changes from `DateTime<Utc>` to `LocalTimestamp`
   - Affects: All provider transform logic, test assertions

3. **Removal of thread-local serialization state**
   - Deletes `serialize_time_with_tz()` function
   - Deletes `set_serialization_timezone()` function
   - Affects: Main application setup code

### Backward Compatibility

**Preserved**:
- JSON output format remains "YYYY-MM-DD HH:MM"
- Field names unchanged (camelCase in JSON)
- API structure unchanged

**Not Preserved**:
- Internal code using `WeatherDataPoint` directly
- Provider implementations (must be updated)
- Test code accessing `.time` field

---

## Validation Rules Summary

### UtcTimestamp
- ✓ Must be valid UTC datetime
- ✓ Should be within forecast range (0-14 days from now)
- ✓ Must parse from API response format

### LocalTimestamp
- ✓ Must have valid timezone information
- ✓ Serializes to "YYYY-MM-DD HH:MM" format
- ✓ Handles DST transitions correctly

### TimezoneConfig
- ✓ Timezone identifier must be valid IANA string
- ✓ Parser provides actionable error messages
- ✓ Default to UTC triggers warning output

### WeatherDataPoint
- ✓ `time` field must be present (not Optional)
- ✓ Measurement fields can be None (provider-dependent)
- ✓ Serializes to backward-compatible JSON

---

## References

- Feature Spec: [`specs/002-timezone-refactor/spec.md`](spec.md)
- Research: [`specs/002-timezone-refactor/research.md`](research.md)
- Current Implementation: [`src/forecast_provider.rs`](../../src/forecast_provider.rs)
- chrono-tz Documentation: https://docs.rs/chrono-tz/latest/chrono_tz/
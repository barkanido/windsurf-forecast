// Contract: Newtype wrappers for timezone-safe timestamp handling
// Feature: 002-timezone-refactor
// 
// This file defines the core newtype wrappers that provide compile-time
// guarantees about timezone state. These types replace raw DateTime<Utc>
// usage throughout the codebase.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Serialize, Serializer};

// ============================================================================
// UtcTimestamp - Wrapper for UTC timestamps from API responses
// ============================================================================

/// Newtype wrapper for UTC timestamps
/// 
/// Used internally to represent timestamps as received from weather APIs.
/// This type makes it explicit that a timestamp is in UTC and has not yet
/// been converted to the user's target timezone.
/// 
/// # Example
/// ```rust
/// let utc_time = Utc::now();
/// let timestamp = UtcTimestamp::new(utc_time);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcTimestamp(pub DateTime<Utc>);

impl UtcTimestamp {
    /// Create a new UtcTimestamp from a DateTime<Utc>
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
    
    /// Get the inner DateTime<Utc> value
    pub fn inner(&self) -> DateTime<Utc> {
        self.0
    }
    
    /// Parse from RFC3339 string (common API format)
    /// 
    /// # Example
    /// ```rust
    /// let timestamp = UtcTimestamp::from_rfc3339("2024-01-15T12:00:00Z")?;
    /// ```
    pub fn from_rfc3339(s: &str) -> Result<Self> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| anyhow!("Failed to parse UTC timestamp from '{}': {}", s, e))?
            .with_timezone(&Utc);
        Ok(Self(dt))
    }
}

// ============================================================================
// LocalTimestamp - Wrapper for timezone-converted timestamps
// ============================================================================

/// Newtype wrapper for timezone-converted timestamps
/// 
/// Used in output structures after conversion to user's target timezone.
/// This type makes it explicit that a timestamp has been converted and is
/// ready for display/serialization.
/// 
/// # Serialization
/// Serializes to "YYYY-MM-DD HH:MM" format (not ISO 8601) to maintain
/// backward compatibility with existing JSON output format.
/// 
/// # Example
/// ```rust
/// let tz: Tz = "America/New_York".parse()?;
/// let utc = UtcTimestamp::new(Utc::now());
/// let local = convert_timezone(utc, tz)?;
/// ```
#[derive(Debug, Clone)]
pub struct LocalTimestamp {
    inner: DateTime<Tz>,
}

impl LocalTimestamp {
    /// Create a new LocalTimestamp from a DateTime<Tz>
    pub fn new(dt: DateTime<Tz>) -> Self {
        Self { inner: dt }
    }
    
    /// Get the inner DateTime<Tz> value
    pub fn inner(&self) -> DateTime<Tz> {
        self.inner
    }
    
    /// Get the timezone of this timestamp
    pub fn timezone(&self) -> Tz {
        self.inner.timezone()
    }
}

// Custom serialization to maintain "YYYY-MM-DD HH:MM" format
impl Serialize for LocalTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format: "YYYY-MM-DD HH:MM" (not ISO 8601)
        // This maintains backward compatibility with existing output
        let formatted = self.inner.format("%Y-%m-%d %H:%M").to_string();
        serializer.serialize_str(&formatted)
    }
}

// ============================================================================
// Conversion Functions
// ============================================================================

/// Convert a UTC timestamp to the target timezone
/// 
/// This is the core timezone conversion function that should be called
/// in the provider transform layer (not during serialization).
/// 
/// # Arguments
/// * `utc` - The UTC timestamp to convert
/// * `target_tz` - The target timezone to convert to
/// 
/// # Returns
/// A LocalTimestamp in the target timezone
/// 
/// # Example
/// ```rust
/// let utc = UtcTimestamp::new(Utc::now());
/// let tz: Tz = "America/New_York".parse()?;
/// let local = convert_timezone(utc, tz)?;
/// ```
/// 
/// # DST Handling
/// The chrono-tz library automatically handles daylight saving time transitions:
/// - Spring forward (gap): Maps to time after transition
/// - Fall back (overlap): Uses standard time (second occurrence)
pub fn convert_timezone(utc: UtcTimestamp, target_tz: Tz) -> Result<LocalTimestamp> {
    let local = target_tz.from_utc_datetime(&utc.0.naive_utc());
    Ok(LocalTimestamp::new(local))
}

/// Convert a batch of UTC timestamps to the target timezone
/// 
/// Helper function for converting multiple timestamps at once.
/// 
/// # Example
/// ```rust
/// let utc_times = vec![
///     UtcTimestamp::new(Utc::now()),
///     UtcTimestamp::new(Utc::now() + Duration::hours(1)),
/// ];
/// let tz: Tz = "Europe/London".parse()?;
/// let local_times = convert_timestamps_batch(&utc_times, tz)?;
/// ```
pub fn convert_timestamps_batch(
    utc_times: &[UtcTimestamp],
    target_tz: Tz,
) -> Result<Vec<LocalTimestamp>> {
    utc_times
        .iter()
        .map(|&utc| convert_timezone(utc, target_tz))
        .collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    
    #[test]
    fn test_utc_timestamp_creation() {
        let dt = Utc.ymd(2024, 1, 15).and_hms(12, 0, 0);
        let timestamp = UtcTimestamp::new(dt);
        assert_eq!(timestamp.inner(), dt);
    }
    
    #[test]
    fn test_utc_timestamp_from_rfc3339() {
        let timestamp = UtcTimestamp::from_rfc3339("2024-01-15T12:00:00Z").unwrap();
        assert_eq!(timestamp.inner().year(), 2024);
        assert_eq!(timestamp.inner().month(), 1);
        assert_eq!(timestamp.inner().day(), 15);
    }
    
    #[test]
    fn test_timezone_conversion_utc_to_ny() {
        let utc = UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(17, 0, 0));
        let tz: Tz = "America/New_York".parse().unwrap();
        
        let local = convert_timezone(utc, tz).unwrap();
        
        // 17:00 UTC = 12:00 EST (UTC-5 in winter)
        assert_eq!(local.inner().hour(), 12);
    }
    
    #[test]
    fn test_timezone_conversion_utc_to_london() {
        let utc = UtcTimestamp::new(Utc.ymd(2024, 7, 15).and_hms(14, 0, 0));
        let tz: Tz = "Europe/London".parse().unwrap();
        
        let local = convert_timezone(utc, tz).unwrap();
        
        // 14:00 UTC = 15:00 BST (UTC+1 in summer due to DST)
        assert_eq!(local.inner().hour(), 15);
    }
    
    #[test]
    fn test_local_timestamp_serialization() {
        let utc = UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(12, 30, 0));
        let tz: Tz = "UTC".parse().unwrap();
        let local = convert_timezone(utc, tz).unwrap();
        
        let json = serde_json::to_string(&local).unwrap();
        assert_eq!(json, "\"2024-01-15 12:30\"");
    }
    
    #[test]
    fn test_batch_conversion() {
        let utc_times = vec![
            UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(12, 0, 0)),
            UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(13, 0, 0)),
            UtcTimestamp::new(Utc.ymd(2024, 1, 15).and_hms(14, 0, 0)),
        ];
        let tz: Tz = "America/New_York".parse().unwrap();
        
        let local_times = convert_timestamps_batch(&utc_times, tz).unwrap();
        
        assert_eq!(local_times.len(), 3);
        // 12:00 UTC = 07:00 EST
        assert_eq!(local_times[0].inner().hour(), 7);
    }
}
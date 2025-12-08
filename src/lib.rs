// Library interface for windsurf-forecast
// Exposes modules for testing

pub mod args;
pub mod config;
pub mod forecast_provider;
pub mod provider_registry;
pub mod providers;

// Test utilities - available for both unit tests and integration tests
// This module is only compiled during testing (not in production builds)
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
// ============================================================================
// Provider Registry Tests (User Story 1)
// ============================================================================
//
// Tests for provider discovery, instantiation, and validation using the
// centralized registry pattern.

use windsurf_forecast::provider_registry::{
    all_provider_descriptions, all_provider_names, check_duplicates, create_provider,
    get_provider_metadata, validate_provider_name,
};

// ============================================================================
// Provider Discovery Tests
// ============================================================================

#[test]
fn test_get_provider_metadata_for_stormglass() {
    let meta = get_provider_metadata("stormglass");
    assert!(meta.is_some(), "StormGlass provider should be registered");

    let meta = meta.unwrap();
    assert_eq!(meta.name, "stormglass");
    assert!(!meta.description.is_empty());
    assert_eq!(meta.api_key_var, "STORMGLASS_API_KEY");
}

#[test]
fn test_get_provider_metadata_for_openweathermap() {
    let meta = get_provider_metadata("openweathermap");
    assert!(
        meta.is_some(),
        "OpenWeatherMap provider should be registered"
    );

    let meta = meta.unwrap();
    assert_eq!(meta.name, "openweathermap");
    assert!(!meta.description.is_empty());
    assert_eq!(meta.api_key_var, "OPEN_WEATHER_MAP_API_KEY");
}

#[test]
fn test_get_provider_metadata_for_unknown_provider() {
    let meta = get_provider_metadata("nonexistent");
    assert!(meta.is_none(), "Unknown provider should return None");
}

#[test]
fn test_all_provider_names_includes_known_providers() {
    let names: Vec<&str> = all_provider_names().collect();

    assert!(
        names.contains(&"stormglass"),
        "Should include stormglass provider"
    );
    assert!(
        names.contains(&"openweathermap"),
        "Should include openweathermap provider"
    );
    assert!(names.len() >= 2, "Should have at least 2 providers");
}

#[test]
fn test_all_provider_descriptions_includes_known_providers() {
    let descriptions: Vec<(&str, &str)> = all_provider_descriptions().collect();

    // Check that we have entries
    assert!(
        descriptions.len() >= 2,
        "Should have at least 2 provider descriptions"
    );

    // Check stormglass is present
    let has_stormglass = descriptions
        .iter()
        .any(|(name, _)| *name == "stormglass");
    assert!(has_stormglass, "Should include stormglass in descriptions");

    // Check openweathermap is present
    let has_openweathermap = descriptions
        .iter()
        .any(|(name, _)| *name == "openweathermap");
    assert!(
        has_openweathermap,
        "Should include openweathermap in descriptions"
    );

    // Verify descriptions are not empty
    for (name, desc) in descriptions {
        assert!(
            !desc.is_empty(),
            "Provider {} should have non-empty description",
            name
        );
    }
}

// ============================================================================
// Provider Validation Tests
// ============================================================================

#[test]
fn test_validate_provider_name_accepts_stormglass() {
    let result = validate_provider_name("stormglass");
    assert!(result.is_ok(), "stormglass should be valid");
}

#[test]
fn test_validate_provider_name_accepts_openweathermap() {
    let result = validate_provider_name("openweathermap");
    assert!(result.is_ok(), "openweathermap should be valid");
}

#[test]
fn test_validate_provider_name_rejects_unknown() {
    let result = validate_provider_name("nonexistent");
    assert!(result.is_err(), "Unknown provider should fail validation");

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unknown provider"),
        "Error should mention unknown provider"
    );
    assert!(
        err_msg.contains("nonexistent"),
        "Error should include the invalid name"
    );
    assert!(
        err_msg.contains("Available providers"),
        "Error should list available providers"
    );
}

#[test]
fn test_validate_provider_name_error_lists_available_providers() {
    let result = validate_provider_name("invalid");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();

    // Should list known providers
    assert!(
        err_msg.contains("stormglass"),
        "Error should list stormglass"
    );
    assert!(
        err_msg.contains("openweathermap"),
        "Error should list openweathermap"
    );
}

// ============================================================================
// Provider Instantiation Tests
// ============================================================================
// Note: These tests will fail if API keys are not set in environment
// They test the error path (missing API key) rather than successful instantiation

#[test]
fn test_create_provider_with_unknown_name_returns_error() {
    let result = create_provider("nonexistent");
    assert!(result.is_err(), "Unknown provider should fail");

    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("Unknown provider"),
            "Error should indicate unknown provider"
        );
        assert!(
            err_msg.contains("nonexistent"),
            "Error should include the invalid name"
        );
        assert!(
            err_msg.contains("Available providers"),
            "Error should list available providers"
        );
    }
}

#[test]
fn test_create_provider_error_lists_available_providers() {
    let result = create_provider("invalid");
    assert!(result.is_err());

    if let Err(e) = result {
        let err_msg = e.to_string();
        assert!(
            err_msg.contains("stormglass"),
            "Should list stormglass as available"
        );
        assert!(
            err_msg.contains("openweathermap"),
            "Should list openweathermap as available"
        );
    }
}

// ============================================================================
// Duplicate Detection Tests
// ============================================================================

#[test]
fn test_check_duplicates_does_not_panic_with_unique_providers() {
    // This test verifies that check_duplicates() doesn't panic
    // when all providers have unique names (normal case)
    check_duplicates();
    // If we get here without panic, test passes
}

// ============================================================================
// Registry Integrity Tests
// ============================================================================

#[test]
fn test_all_registered_providers_have_valid_metadata() {
    // Verify all registered providers have proper metadata structure
    for (name, description) in all_provider_descriptions() {
        // Name should not be empty
        assert!(!name.is_empty(), "Provider name should not be empty");

        // Description should not be empty
        assert!(
            !description.is_empty(),
            "Provider {} should have description",
            name
        );

        // Should be able to get full metadata
        let meta = get_provider_metadata(name);
        assert!(
            meta.is_some(),
            "Should be able to retrieve metadata for {}",
            name
        );

        // API key var should not be empty
        let meta = meta.unwrap();
        assert!(
            !meta.api_key_var.is_empty(),
            "Provider {} should have API key var",
            name
        );
    }
}

#[test]
fn test_provider_names_are_consistent() {
    // Verify that all_provider_names() and all_provider_descriptions()
    // return consistent provider names
    let names: Vec<&str> = all_provider_names().collect();
    let desc_names: Vec<&str> = all_provider_descriptions().map(|(n, _)| n).collect();

    assert_eq!(
        names.len(),
        desc_names.len(),
        "Number of providers should match between iterators"
    );

    for name in names {
        assert!(
            desc_names.contains(&name),
            "Provider {} should be in descriptions",
            name
        );
    }
}

#[test]
fn test_provider_metadata_lookup_is_case_sensitive() {
    // Provider names should be case-sensitive
    assert!(get_provider_metadata("stormglass").is_some());
    assert!(get_provider_metadata("StormGlass").is_none());
    assert!(get_provider_metadata("STORMGLASS").is_none());
}
# Research: AI-Powered Forecast Analysis

**Feature**: 005-ai-integration  
**Date**: 2025-12-10  
**Status**: Complete

## Executive Summary

This feature integrates OpenAI's ChatGPT API to enable natural language analysis of weather forecasts. All technical unknowns have been resolved through clarification sessions documented in the spec. This research document consolidates the architectural decisions, best practices, and implementation patterns.

## Research Topics

### 1. OpenAI API Integration Pattern

**Decision**: Use OpenAI REST API with reqwest async HTTP client and two-message conversation structure (system + user messages)

**Rationale**:
- **REST API over SDK**: OpenAI's official Rust SDK is less mature than Python/JavaScript versions. Direct REST API usage via reqwest provides:
  - Full control over request/response handling
  - Better error handling customization
  - No additional dependency on potentially unstable SDK
  - Standard HTTP patterns familiar to Rust developers

- **Two-Message Conversation Pattern**: 
  - System message: Contains forecast JSON as context (separates data from query)
  - User message: Contains the user's analysis prompt/question
  - This separation aligns with OpenAI's best practices for providing context vs. instructions

- **Sequential Execution**: Forecast generation must complete before AI API call to ensure:
  - AI always analyzes current forecast data
  - Error handling is clearer (forecast errors vs. AI errors)
  - Simpler async flow without race conditions

**Alternatives Considered**:
- **OpenAI Rust SDK (openai-rs)**: Rejected due to maturity concerns and desire for direct control
- **Parallel execution**: Rejected due to complexity and no real performance benefit (AI call dominates total time)
- **Single message with embedded JSON**: Rejected because it mixes context and query, reducing clarity

**References**:
- OpenAI Chat Completions API: https://platform.openai.com/docs/api-reference/chat/create
- reqwest documentation: https://docs.rs/reqwest/latest/reqwest/

---

### 2. HTTP Client Selection (reqwest)

**Decision**: Use reqwest 0.11+ with async support and JSON features

**Rationale**:
- **Industry Standard**: Most popular Rust HTTP client (40M+ downloads)
- **Async-First**: Built on tokio, aligns with Rust async ecosystem
- **Rich Error Handling**: Provides detailed error types for timeouts, connection failures, HTTP status codes
- **JSON Support**: Built-in serde_json integration for request/response serialization
- **Connection Pooling**: Automatic connection reuse for multiple API calls
- **TLS Support**: Secure HTTPS connections out of the box

**Best Practices**:
```rust
use reqwest::Client;
use std::time::Duration;

// Create client with timeout
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;

// Make request with Bearer token authentication
let response = client
    .post("https://api.openai.com/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&request_body)
    .send()
    .await?;
```

**Alternatives Considered**:
- **hyper**: Lower-level, more complex for this use case
- **ureq**: Synchronous, doesn't fit with existing async architecture
- **curl bindings**: Not idiomatic Rust, harder to maintain

**References**:
- reqwest: https://docs.rs/reqwest/latest/reqwest/
- reqwest async example: https://github.com/seanmonstar/reqwest/tree/master/examples

---

### 3. Async Runtime Integration (tokio)

**Decision**: Use tokio runtime for async operations, integrate with existing sync main function using `tokio::runtime::Runtime::new().block_on()`

**Rationale**:
- **Existing Pattern**: Project already uses async patterns (though not yet in main)
- **reqwest Requirement**: reqwest async requires a runtime like tokio
- **Minimal Changes**: Can wrap async AI calls without converting entire application
- **Future-Proof**: Enables future async provider implementations if needed

**Implementation Pattern**:
```rust
// In main.rs
fn main() -> anyhow::Result<()> {
    // ... existing setup ...
    
    if args.ai {
        // Create runtime for async AI operations
        let runtime = tokio::runtime::Runtime::new()?;
        let ai_response = runtime.block_on(async {
            // Async AI operations here
            ai::analyze_forecast(&forecast_json, &prompt, &args.model).await
        })?;
        
        println!("{}", ai_response);
        return Ok(());
    }
    
    // ... existing non-AI flow ...
}
```

**Alternatives Considered**:
- **Full async conversion**: Rejected due to scope creep and unnecessary complexity
- **async-std**: Rejected because tokio is more widely used and better documented
- **Synchronous HTTP client**: Rejected because it doesn't fit modern Rust patterns

**References**:
- tokio documentation: https://docs.rs/tokio/latest/tokio/
- Bridging sync/async: https://tokio.rs/tokio/topics/bridging

---

### 4. Error Handling Strategy

**Decision**: Use anyhow for application-level errors, thiserror for AI-specific error types, with actionable error messages

**Rationale**:
- **Consistency**: Matches existing error handling pattern in the codebase
- **Actionable Messages**: All errors provide clear next steps (constitution principle)
- **Structured AI Errors**: Use thiserror for AI module to distinguish error types:
  - Missing/invalid API key
  - Network/timeout errors
  - API rate limits
  - Invalid responses
  - Prompt file errors

**Error Type Design**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("OPENAI_API_KEY not found. Add it to your .env file with: OPENAI_API_KEY=sk-...")]
    MissingApiKey,
    
    #[error("OpenAI API request timed out after 60 seconds. Please try again.")]
    Timeout,
    
    #[error("OpenAI API rate limit exceeded. Please try again later or check your quota at platform.openai.com")]
    RateLimit,
    
    #[error("Prompt file not found: {0}")]
    PromptFileNotFound(String),
    
    #[error("Prompt file too large: {size}. Maximum allowed: 32KB")]
    PromptFileTooLarge { size: String },
    
    #[error("OpenAI API error ({status}): {message}")]
    ApiError { status: u16, message: String },
}
```

**Alternatives Considered**:
- **Only anyhow**: Rejected because structured errors improve debugging
- **Custom Result types**: Rejected due to complexity, anyhow is sufficient

**References**:
- Error handling in Rust: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- thiserror: https://docs.rs/thiserror/latest/thiserror/

---

### 5. Model Selection and Validation

**Decision**: Support gpt-5 and gpt-4o models with enum-based validation at CLI parsing time

**Rationale**:
- **Early Validation**: Validate model names during argument parsing (fail fast principle)
- **Type Safety**: Use enum to prevent invalid model names at compile time
- **Extensibility**: Easy to add new models as they become available
- **Default Choice**: gpt-5 as default balances quality and cost per spec requirements

**Implementation Pattern**:
```rust
use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum ChatModel {
    #[value(name = "gpt-5")]
    Gpt5,
    #[value(name = "gpt-4o")]
    Gpt4o,
}

impl ChatModel {
    pub fn api_name(&self) -> &str {
        match self {
            ChatModel::Gpt5 => "gpt-5",
            ChatModel::Gpt4o => "gpt-4o",
        }
    }
}

impl Default for ChatModel {
    fn default() -> Self {
        ChatModel::Gpt5
    }
}
```

**Alternatives Considered**:
- **String validation**: Rejected because enum provides compile-time safety
- **Runtime validation only**: Rejected because fail-fast at parse time is better UX

**References**:
- clap ValueEnum: https://docs.rs/clap/latest/clap/derive.ValueEnum.html

---

### 6. Prompt Management

**Decision**: Support both interactive prompts (stdin) and file-based prompts with size validation

**Rationale**:
- **Flexibility**: Interactive for ad-hoc questions, file-based for automation
- **Size Limits**: 32KB limit prevents excessive API costs and token usage
- **UTF-8 Encoding**: Standard for text files, matches JSON and Rust string handling
- **User Experience**: Clear error messages when files are missing, empty, or too large

**Implementation Pattern**:
```rust
use std::fs;
use std::io::{self, Write};

pub fn load_prompt(prompt_file: Option<&str>) -> anyhow::Result<String> {
    match prompt_file {
        Some(path) => {
            // Validate file exists
            let metadata = fs::metadata(path)
                .map_err(|_| AIError::PromptFileNotFound(path.to_string()))?;
            
            // Validate file size (32KB = 32768 bytes)
            if metadata.len() > 32768 {
                let size = format!("{}KB", metadata.len() / 1024);
                return Err(AIError::PromptFileTooLarge { size }.into());
            }
            
            // Read as UTF-8
            let content = fs::read_to_string(path)?;
            
            // Validate not empty
            if content.trim().is_empty() {
                return Err(anyhow::anyhow!("Prompt file is empty: {}", path));
            }
            
            Ok(content)
        }
        None => {
            // Interactive prompt
            print!("Enter your forecast analysis question: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            Ok(input.trim().to_string())
        }
    }
}
```

**Alternatives Considered**:
- **No size limit**: Rejected due to cost and API constraints
- **Binary prompt files**: Rejected because text-only makes sense for prompts

---

### 7. Timeout Configuration

**Decision**: Fixed 60-second timeout for all OpenAI API requests

**Rationale**:
- **Balance**: Allows complex analysis while preventing indefinite hangs
- **User Expectation**: 60 seconds is reasonable for AI processing
- **Explicit Feedback**: Timeout errors provide clear message about duration
- **Not Configurable**: Fixed timeout reduces complexity, can be made configurable later if needed

**Implementation Pattern**:
```rust
use std::time::Duration;

const OPENAI_TIMEOUT: Duration = Duration::from_secs(60);

let client = reqwest::Client::builder()
    .timeout(OPENAI_TIMEOUT)
    .build()?;
```

**Alternatives Considered**:
- **No timeout**: Rejected due to poor UX (indefinite hangs)
- **Configurable timeout**: Rejected to reduce complexity in v1
- **Shorter timeout (30s)**: Rejected because complex analysis may need more time

---

### 8. Testing Strategy

**Decision**: Unit tests for all AI module components, mock HTTP responses using httpmock crate (consistent with existing test pattern)

**Rationale**:
- **Consistency**: Matches existing provider test patterns (stormglass_test.rs, openweathermap_test.rs)
- **No Real API Calls**: Tests must not depend on actual OpenAI API (cost, reliability, speed)
- **Coverage Target**: >80% line coverage for AI module per constitution
- **Fast Execution**: Tests should complete in <1 second like existing tests

**Test Cases to Implement**:
1. Successful AI analysis with valid response
2. Missing API key error
3. Invalid model name handling
4. Timeout simulation
5. Rate limit error (429 status)
6. API error responses (401, 500, etc.)
7. Prompt file loading (exists, missing, empty, too large)
8. Flag dependency validation (--prompt-file requires --ai, etc.)

**Implementation Pattern**:
```rust
use httpmock::prelude::*;

#[tokio::test]
async fn test_successful_ai_analysis() {
    let server = MockServer::start();
    
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/chat/completions")
            .header("Authorization", "Bearer test-key")
            .json_body(json!({
                "model": "gpt-5",
                "messages": [/* ... */]
            }));
        then.status(200)
            .json_body(json!({
                "choices": [{
                    "message": {
                        "content": "Good conditions for sailing"
                    }
                }]
            }));
    });
    
    // Test implementation
}
```

**Alternatives Considered**:
- **Integration tests with real API**: Rejected due to cost and flakiness
- **Manual testing only**: Rejected because automated tests catch regressions

**References**:
- httpmock: https://docs.rs/httpmock/latest/httpmock/
- Existing test patterns: tests/stormglass_test.rs, tests/openweathermap_test.rs

---

## Dependencies Summary

### New Dependencies

```toml
[dependencies]
# Existing dependencies remain unchanged
# ...

# NEW: OpenAI API integration
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["rt", "macros"] }

[dev-dependencies]
# Existing test dependencies remain unchanged
# ...

# NEW: HTTP mocking for AI tests
httpmock = "0.7"
```

### Justification
- **reqwest**: Required for async HTTP requests to OpenAI API
- **tokio**: Required runtime for async operations (reqwest dependency)
- **httpmock**: Testing dependency, consistent with existing test infrastructure

---

## Implementation Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| OpenAI API changes breaking compatibility | High | Version API endpoint in code, monitor OpenAI changelog |
| Rate limiting affecting users | Medium | Clear error messages, suggest checking quota |
| Large forecast JSON exceeding context limits | Low | Current forecasts are small, add size check if needed |
| Async runtime overhead | Low | Minimal overhead, only used when --ai flag is present |
| API key security concerns | Medium | Use .env file (gitignored), document best practices |

---

## Open Questions

None - All clarifications resolved in spec clarification sessions.

---

## References

1. OpenAI API Documentation: https://platform.openai.com/docs/api-reference
2. reqwest Documentation: https://docs.rs/reqwest/latest/reqwest/
3. tokio Documentation: https://docs.rs/tokio/latest/tokio/
4. Rust Async Book: https://rust-lang.github.io/async-book/
5. Existing codebase patterns: src/providers/stormglass.rs, src/providers/openweathermap.rs
6. Constitution: .specify/memory/constitution.md

---

## Next Steps

Proceed to Phase 1:
1. Generate data-model.md (entity definitions)
2. Generate contracts/ (API contracts and types)
3. Generate quickstart.md (implementation guide)
4. Update agent context (.roo/rules-code/AGENTS.md)
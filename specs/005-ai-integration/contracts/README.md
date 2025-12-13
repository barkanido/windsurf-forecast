# API Contracts: AI-Powered Forecast Analysis

**Feature**: 005-ai-integration  
**Date**: 2025-12-10

## Overview

This directory contains the contract definitions (type signatures, interfaces, and API schemas) for the AI integration feature. These contracts serve as the specification for implementation and must be followed exactly.

## Contract Files

1. **[`types.rs`](types.rs)** - Core type definitions for OpenAI API integration
2. **[`errors.rs`](errors.rs)** - Error type definitions with actionable messages
3. **[`client.rs`](client.rs)** - OpenAI HTTP client interface contract
4. **[`prompt.rs`](prompt.rs)** - Prompt loading and validation interface
5. **[`args.rs`](args.rs)** - CLI argument extensions for AI flags
6. **[`test_contract.rs`](test_contract.rs)** - Test case specifications

## Integration Points

### 1. CLI Arguments ([`src/args.rs`](../../../src/args.rs))

Add three new flags to the existing `Args` struct:

```rust
#[derive(Parser, Debug)]
pub struct Args {
    // ... existing fields ...
    
    /// Enable AI-powered forecast analysis
    #[arg(long, default_value_t = false)]
    pub ai: bool,
    
    /// Path to file containing analysis prompt
    #[arg(long, requires = "ai")]
    pub prompt_file: Option<String>,
    
    /// ChatGPT model to use for analysis
    #[arg(long, default_value = "gpt-5", requires = "ai")]
    pub model: ChatModel,
}
```

### 2. Main Flow ([`src/main.rs`](../../../src/main.rs))

After forecast generation, add conditional AI flow:

```rust
fn main() -> anyhow::Result<()> {
    // ... existing setup and forecast generation ...
    
    if args.ai {
        // Create tokio runtime for async operations
        let runtime = tokio::runtime::Runtime::new()?;
        
        // Run async AI analysis
        let ai_response = runtime.block_on(async {
            ai::analyze_forecast(&forecast_data, &args).await
        })?;
        
        // Display only AI response (not forecast JSON)
        println!("{}", ai_response);
        return Ok(());
    }
    
    // ... existing non-AI output ...
}
```

### 3. New AI Module ([`src/ai/mod.rs`](../../../src/ai/mod.rs))

Create new module structure:

```
src/ai/
├── mod.rs       # Public exports and main analyze_forecast() function
├── types.rs     # OpenAI API types (ChatRequest, ChatResponse, etc.)
├── errors.rs    # AIError enum with actionable messages
├── client.rs    # HTTP client for OpenAI API
└── prompt.rs    # Prompt loading from file or stdin
```

### 4. Test File ([`tests/ai_test.rs`](../../../tests/ai_test.rs))

New test file following existing patterns from `tests/stormglass_test.rs`.

## OpenAI API Endpoint

**URL**: `https://api.openai.com/v1/chat/completions`  
**Method**: `POST`  
**Authentication**: Bearer token in `Authorization` header  
**Timeout**: 60 seconds

**Request Format**:
```json
{
  "model": "gpt-5",
  "messages": [
    {
      "role": "system",
      "content": "[forecast JSON string]"
    },
    {
      "role": "user",
      "content": "[user's analysis prompt]"
    }
  ]
}
```

**Response Format**:
```json
{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-5",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "[AI analysis text]"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 100,
    "completion_tokens": 50,
    "total_tokens": 150
  }
}
```

## Error Handling

All errors must follow the Error Transparency principle with actionable messages:

- **Missing API Key**: Tell user to add `OPENAI_API_KEY` to `.env` file
- **Timeout**: Inform user of 60-second timeout and suggest retry
- **Rate Limit**: Direct user to platform.openai.com to check quota
- **Invalid Model**: List supported models (gpt-5, gpt-4o)
- **Prompt File Errors**: Specify exact issue (missing, too large, empty)

## Dependencies

Required additions to `Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["rt", "macros"] }

[dev-dependencies]
httpmock = "0.7"
```

## Testing Strategy

1. **Unit Tests**: Mock HTTP responses using httpmock (consistent with existing provider tests)
2. **No Real API Calls**: Tests must not depend on actual OpenAI API
3. **Coverage Target**: >80% line coverage for AI module
4. **Execution Time**: All tests complete in <1 second

## Validation Checklist

Before implementation:
- [ ] All type definitions from contracts/ are implemented exactly
- [ ] Error messages match the specifications in errors.rs
- [ ] CLI argument dependencies enforced by clap
- [ ] HTTP timeout set to 60 seconds
- [ ] API key loaded from OPENAI_API_KEY environment variable
- [ ] Prompt file validation (size, existence, non-empty)
- [ ] Two-message structure (system + user) in API requests
- [ ] Only AI response displayed (no forecast JSON)
- [ ] All edge cases from spec.md handled

## References

- Feature Spec: [`../spec.md`](../spec.md)
- Data Model: [`../data-model.md`](../data-model.md)
- Research: [`../research.md`](../research.md)
- OpenAI API Docs: https://platform.openai.com/docs/api-reference/chat
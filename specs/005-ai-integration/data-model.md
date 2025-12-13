# Data Model: AI-Powered Forecast Analysis

**Feature**: 005-ai-integration  
**Date**: 2025-12-10  
**Status**: Complete

## Overview

This document defines the data entities, types, and relationships for the AI integration feature. The design follows the existing codebase patterns and integrates with the forecast generation pipeline.

## Core Entities

### 1. ChatModel (Enum)

Represents the OpenAI ChatGPT model to use for analysis.

**Fields**:
- `Gpt5`: Default model, represents "gpt-5" in API calls
- `Gpt4o`: Alternative model, represents "gpt-4o" in API calls

**Constraints**:
- Must be validated at CLI parsing time using clap's `ValueEnum` trait
- Invalid model names cause argument parsing failure with clear error message

**Implementation**:
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

**Validation Rules**:
- Only "gpt-5" and "gpt-4o" are valid values
- Case-sensitive (lowercase with hyphen)
- Defaults to "gpt-5" if not specified

---

### 2. AIArgs (CLI Arguments Extension)

Extension to the existing `Args` struct to support AI flags.

**Fields**:
- `ai: bool` - Enables AI analysis mode (default: false)
- `prompt_file: Option<String>` - Path to prompt file (optional)
- `model: ChatModel` - Model to use (default: ChatModel::Gpt5)

**Constraints**:
- `--prompt-file` requires `--ai` flag to be set
- `--model` requires `--ai` flag to be set
- Validated at argument parsing time (fail fast)

**Implementation**:
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
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

**Validation Rules**:
- `prompt_file` path must exist if provided
- `prompt_file` must be readable and ≤32KB
- `prompt_file` must not be empty (non-whitespace content required)

---

### 3. ChatMessage (OpenAI API Request)

Represents a single message in the conversation with OpenAI.

**Fields**:
- `role: String` - Message role: "system" or "user"
- `content: String` - Message content (JSON for system, prompt for user)

**Constraints**:
- `role` must be one of: "system", "user", "assistant" (only system/user used in requests)
- `content` must not be empty
- System message contains forecast JSON, user message contains prompt

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
    
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
}
```

**Validation Rules**:
- Role must be valid OpenAI message role
- Content cannot be empty string

---

### 4. ChatRequest (OpenAI API Request Body)

Complete request structure sent to OpenAI Chat Completions API.

**Fields**:
- `model: String` - Model identifier (e.g., "gpt-5")
- `messages: Vec<ChatMessage>` - Conversation messages (system + user)
- `temperature: Option<f32>` - Sampling temperature (0.0-2.0, default: 1.0)
- `max_tokens: Option<u32>` - Maximum response tokens (optional)

**Constraints**:
- `model` must match ChatModel enum values
- `messages` must contain exactly 2 messages: system (forecast) + user (prompt)
- `temperature` if provided must be between 0.0 and 2.0
- Total tokens (input + output) must fit within model's context window

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    pub fn new(model: ChatModel, forecast_json: String, prompt: String) -> Self {
        Self {
            model: model.api_name().to_string(),
            messages: vec![
                ChatMessage::system(forecast_json),
                ChatMessage::user(prompt),
            ],
            temperature: None,
            max_tokens: None,
        }
    }
}
```

**Validation Rules**:
- Messages array must have exactly 2 elements
- First message must be system role with forecast JSON
- Second message must be user role with prompt
- Model string must match known model identifiers

---

### 5. ChatChoice (OpenAI API Response Choice)

Represents a single completion choice from the API response.

**Fields**:
- `index: u32` - Choice index (0 for first choice)
- `message: ChatMessage` - Response message with "assistant" role
- `finish_reason: Option<String>` - Why generation stopped ("stop", "length", etc.)

**Constraints**:
- `message.role` should be "assistant" for responses
- `finish_reason` indicates if response is complete or truncated

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}
```

**Validation Rules**:
- Index should be 0 (we only request 1 completion)
- Message must have content
- finish_reason "length" indicates truncation

---

### 6. ChatResponse (OpenAI API Response Body)

Complete response structure received from OpenAI Chat Completions API.

**Fields**:
- `id: String` - Unique response identifier
- `object: String` - Object type (should be "chat.completion")
- `created: u64` - Unix timestamp of response creation
- `model: String` - Model that generated the response
- `choices: Vec<ChatChoice>` - Array of completion choices (we use first one)
- `usage: Option<TokenUsage>` - Token usage statistics

**Constraints**:
- `choices` array should have at least 1 element
- We extract `choices[0].message.content` as the analysis result

**Implementation**:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<TokenUsage>,
}

impl ChatResponse {
    pub fn extract_content(&self) -> anyhow::Result<String> {
        self.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No choices in OpenAI response"))
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
```

**Validation Rules**:
- Choices array cannot be empty
- First choice must have content
- If finish_reason is "length", response may be truncated

---

### 7. AIError (Error Types)

Custom error types for AI integration failures.

**Variants**:
- `MissingApiKey` - OPENAI_API_KEY not found in environment
- `Timeout` - Request exceeded 60-second timeout
- `RateLimit` - OpenAI rate limit exceeded (429 status)
- `PromptFileNotFound(String)` - Prompt file doesn't exist
- `PromptFileTooLarge { size: String }` - Prompt file exceeds 32KB
- `PromptFileEmpty(String)` - Prompt file has no content
- `ApiError { status: u16, message: String }` - Generic API error
- `InvalidResponse(String)` - Response parsing failed
- `EmptyPrompt` - User entered empty prompt interactively

**Implementation**:
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
    
    #[error("Prompt file is empty: {0}")]
    PromptFileEmpty(String),
    
    #[error("OpenAI API error ({status}): {message}")]
    ApiError { status: u16, message: String },
    
    #[error("Invalid OpenAI response: {0}")]
    InvalidResponse(String),
    
    #[error("Empty prompt provided. Please enter a question about the forecast.")]
    EmptyPrompt,
}
```

**Error Handling Patterns**:
- All errors implement `Display` trait for user-friendly messages
- Errors include actionable guidance (where to get API key, quota URL, etc.)
- HTTP status codes map to specific error types:
  - 401: Invalid API key (ApiError)
  - 429: Rate limit (RateLimit)
  - 500+: Server error (ApiError)
  - Timeout: Connection timeout (Timeout)

---

## Entity Relationships

```
Args (CLI)
  └─> ai: bool
  └─> prompt_file: Option<String>
  └─> model: ChatModel
         │
         ├─> Gpt5 (default)
         └─> Gpt4o
         
Main Flow:
  1. Parse Args → validate ai flag dependencies
  2. Load prompt → String (from file or stdin)
  3. Generate forecast → Vec<WeatherDataPoint> → JSON String
  4. Create ChatRequest:
       ├─> ChatMessage::system(forecast_json)
       └─> ChatMessage::user(prompt)
  5. Send to OpenAI → ChatResponse
  6. Extract ChatResponse.choices[0].message.content → display to user

Error Flow:
  Any step can produce AIError → display error message → exit with error code
```

## Data Flow Diagram

```
┌─────────────────┐
│   CLI Args      │
│  --ai           │
│  --prompt-file  │
│  --model        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Load Prompt     │
│ (file or stdin) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Generate        │
│ Forecast        │
│ (existing flow) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Serialize       │
│ to JSON         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Create          │
│ ChatRequest     │
│ - system msg    │
│ - user msg      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ HTTP POST       │
│ to OpenAI       │
│ (with Bearer)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Parse           │
│ ChatResponse    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Extract         │
│ choices[0]      │
│ .message        │
│ .content        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Display to      │
│ stdout          │
└─────────────────┘
```

## Validation Summary

| Entity | Validation Point | Validation Rules |
|--------|-----------------|------------------|
| ChatModel | CLI parsing | Only "gpt-5", "gpt-4o" allowed |
| AIArgs | CLI parsing | --prompt-file and --model require --ai |
| Prompt file | Before API call | Must exist, ≤32KB, non-empty |
| API key | Before API call | OPENAI_API_KEY must be set |
| ChatRequest | Construction | Exactly 2 messages (system + user) |
| ChatResponse | After API call | choices array not empty |

## State Transitions

```
START
  │
  ├─> ai flag = false → Skip AI entirely → END (existing flow)
  │
  └─> ai flag = true
        │
        ├─> prompt_file provided → Load from file
        │     │
        │     ├─> File missing → ERROR: PromptFileNotFound
        │     ├─> File too large → ERROR: PromptFileTooLarge
        │     ├─> File empty → ERROR: PromptFileEmpty
        │     └─> Success → Continue
        │
        └─> prompt_file not provided → Prompt user for input
              │
              ├─> Empty input → ERROR: EmptyPrompt
              └─> Success → Continue
                    │
                    ├─> API key missing → ERROR: MissingApiKey
                    │
                    └─> Make API request
                          │
                          ├─> Timeout → ERROR: Timeout
                          ├─> Rate limit → ERROR: RateLimit
                          ├─> API error → ERROR: ApiError
                          ├─> Invalid response → ERROR: InvalidResponse
                          └─> Success → Display analysis → END
```

## Type Safety Guarantees

1. **Model Selection**: `ChatModel` enum prevents invalid model names at compile time
2. **Flag Dependencies**: clap's `requires` attribute enforces --ai dependency at parse time
3. **Message Structure**: Type system ensures exactly 2 messages (system + user) in request
4. **Error Handling**: `Result<T, AIError>` forces explicit error handling at each step
5. **Async Safety**: tokio runtime ensures proper async execution without blocking

## Performance Characteristics

| Operation | Expected Duration | Constraint |
|-----------|------------------|------------|
| Prompt file read | <10ms | Max 32KB file |
| JSON serialization | <50ms | Typical forecast ~10KB |
| HTTP request/response | 2-30 seconds | 60s timeout |
| Response parsing | <10ms | JSON deserialize |
| Total AI flow | 2-30 seconds | Dominated by API call |

## Dependencies on Existing Entities

- `WeatherDataPoint` (from [`forecast_provider.rs`](../../src/forecast_provider.rs:19)) - Forecast data structure
- `Args` (from [`args.rs`](../../src/args.rs:1)) - Extended with AI fields
- `.env` file - Add `OPENAI_API_KEY` alongside existing provider keys

## Next Steps

1. Generate contracts/ directory with Rust type definitions
2. Generate quickstart.md with implementation guide
3. Update agent context with new patterns
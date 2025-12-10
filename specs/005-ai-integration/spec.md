# Feature Specification: AI-Powered Forecast Analysis

**Feature Branch**: `005-ai-integration`
**Created**: 2025-12-10
**Status**: Draft
**Input**: User description: "Add AI integration with OpenAI API for forecast analysis. Support --ai flag for enabling AI analysis, --prompt-file for custom prompts, and --model flag for selecting ChatGPT models (default: gpt-4)"

## Clarifications

### Session 2025-12-10

- Q: When the AI feature is invoked (--ai flag), should the system first generate the forecast from weather providers and THEN call OpenAI, or should these operations happen in parallel for faster response times? → A: Sequential: forecast must be generated first, then fed into the AI model chat
- Q: What HTTP client library should be used for making OpenAI API requests? → A: reqwest (async, most popular Rust HTTP client with good OpenAI integration)
- Q: When displaying the AI response to the user, should the system show ONLY the AI analysis, or should it also display the original forecast JSON data alongside the AI response? → A: AI response only (cleaner output, user requested AI analysis)
- Q: What timeout duration should be set for OpenAI API requests to balance between allowing sufficient processing time and avoiding indefinite hangs? → A: 60 seconds (allows complex analysis)
- Q: How should the system structure the OpenAI API conversation? Should it send the forecast JSON as a system message for context, or include it directly in the user message alongside the prompt? → A: System message with forecast JSON + user message with prompt (cleaner separation of context and query)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic AI Forecast Analysis (Priority: P1)

A user wants to get AI-powered insights about their weather forecast without crafting a custom prompt. They run the application with the `--ai` flag and are prompted to describe what they want to know about the forecast. The AI analyzes the generated forecast data and provides natural language insights.

**Why this priority**: This is the core value proposition - enabling users to ask questions about forecast data in natural language rather than parsing JSON manually. This delivers immediate value and can be tested independently.

**Independent Test**: Can be fully tested by running the app with `--ai` flag, entering a simple question like "Should I go sailing tomorrow?", and receiving a relevant AI-generated response based on the forecast data. Success means the AI receives the forecast and responds meaningfully.

**Acceptance Scenarios**:

1. **Given** user has valid API credentials, **When** user runs app with `--ai` flag and no prompt file, **Then** system prompts user to enter their question
2. **Given** user enters "Is it safe to windsurf tomorrow?", **When** AI processes forecast data, **Then** system returns analysis considering wind conditions, weather patterns, and safety factors
3. **Given** AI analysis completes successfully, **When** response is returned, **Then** application displays ONLY the AI response (no forecast JSON) and exits

---

### User Story 2 - Predefined Prompt Analysis (Priority: P2)

A user wants to repeatedly analyze forecasts with the same question (e.g., daily surf condition check). They create a text file with their prompt and use `--prompt-file` to automatically send it with each forecast request, eliminating the need to type the same question repeatedly.

**Why this priority**: Enables automation and repeated analysis workflows. Users can script daily forecast checks with consistent prompting. This builds on P1 functionality.

**Independent Test**: Create a prompt file with "Rate today's conditions for kitesurfing on a scale of 1-10 with explanation", run with `--prompt-file surf-check.txt --ai`, and verify the AI response matches the prompt's intent without user interaction.

**Acceptance Scenarios**:

1. **Given** user has a prompt file at specified path, **When** user runs app with `--prompt-file <path> --ai`, **Then** system reads prompt from file without prompting user
2. **Given** prompt file contains "Analyze wind patterns for sailing", **When** AI processes request, **Then** system provides sailing-specific analysis without additional user input
3. **Given** prompt file doesn't exist, **When** user specifies `--prompt-file`, **Then** system shows clear error message indicating file not found

---

### User Story 3 - Model Selection for Different Use Cases (Priority: P3)

A user wants to choose between different OpenAI models based on their needs - choosing between gpt-4o and gpt-5 depending on their requirements. They use the `--model` flag to specify which model to use.

**Why this priority**: Provides flexibility for cost/quality tradeoffs and future-proofs the feature as new models become available. This is enhancement functionality that builds on the core AI feature.

**Independent Test**: Run with `--ai --model gpt-4o` and verify response comes from the specified model (can be validated through API logs or response characteristics). Repeat with `--model gpt-5` to confirm model selection works.

**Acceptance Scenarios**:

1. **Given** no model specified, **When** user runs with `--ai` flag only, **Then** system defaults to gpt-5
2. **Given** user specifies `--model gpt-4o`, **When** AI request is made, **Then** system uses gpt-4o model
3. **Given** user specifies invalid model name, **When** app starts, **Then** system shows error with list of supported models (gpt-5, gpt-4o)
4. **Given** user specifies `--model gpt-5`, **When** AI analysis runs, **Then** system uses gpt-5 model

---

### Edge Cases

1. **OpenAI API key is missing or invalid**: When `--ai` flag is present but OPENAI_API_KEY is missing, show error "OPENAI_API_KEY not found. Add it to your .env file." and exit with error code. Do not generate or display forecast data.

2. **Network timeouts during AI API calls (>60 seconds)**: Show error message "OpenAI API request timed out after 60 seconds. Please try again." and exit with error code. Do not display the forecast.

3. **Prompt file exists but is empty**: Show error "Prompt file is empty: [path]" and exit with error code. Do not proceed with AI analysis.

4. **OpenAI API rate limits or quota exhaustion**: Show error message "OpenAI API rate limit exceeded. Please try again later or check your quota at platform.openai.com" and exit with error code.

5. **Forecast data is empty or malformed**: Show error "Cannot perform AI analysis: forecast data is invalid" and exit with error code. Do not attempt to call OpenAI API.

6. **Very large prompt files (>32KB)**: Show error "Prompt file too large: [size]. Maximum allowed: 32KB" and exit with error code.

7. **AI response is truncated due to token limits**: Display the truncated response as-is with a warning footer: "Note: Response may be incomplete due to length limits".

8. **User tries to use `--prompt-file` without `--ai` flag**: Show error "--prompt-file requires --ai flag to be set" and exit with error code.

9. **Unsupported model names**: Show error "Unsupported model: [name]. Supported models: gpt-5, gpt-4o" and exit with error code during argument validation.

10. **User provides `--model` without `--ai` flag**: Show error "--model requires --ai flag to be set" and exit with error code.

All error messages follow the Error Transparency principle by providing actionable guidance to users.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support an `--ai` flag that enables AI-powered forecast analysis
- **FR-002**: System MUST support a `--prompt-file <path>` flag that reads the initial prompt from a file
- **FR-003**: System MUST support a `--model <model_name>` flag that specifies which OpenAI ChatGPT model to use
- **FR-004**: System MUST default to `gpt-5` model when `--model` flag is not provided
- **FR-005**: System MUST prompt user for input when `--ai` is enabled but `--prompt-file` is not provided
- **FR-006**: System MUST generate the complete forecast data BEFORE initiating any AI API calls (sequential execution: forecast generation → AI analysis)
- **FR-007**: System MUST structure OpenAI API conversation with two messages: (1) system message containing the forecast JSON as context, (2) user message containing the user's prompt/question
- **FR-007a**: System MUST send this conversation structure to OpenAI Chat Completions REST API (https://platform.openai.com/docs/api-reference/chat/create)
- **FR-008**: System MUST authenticate with OpenAI REST API using API key from `OPENAI_API_KEY` environment variable (following `.env` pattern)
- **FR-009**: System MUST display ONLY the AI response text to the user (without the original forecast JSON) and exit the application
- **FR-010**: System MUST validate that the model name is supported before making API calls
- **FR-011**: System MUST provide clear error messages when API key is missing, invalid, or API calls fail
- **FR-012**: System MUST support currently available ChatGPT models: gpt-5 and gpt-4o
- **FR-013**: System MUST read prompt file content as UTF-8 text when `--prompt-file` is specified
- **FR-014**: System MUST validate that prompt file exists and is readable before attempting to use it
- **FR-015**: System MUST include the complete forecast JSON in the system message role to provide context for the AI analysis
- **FR-016**: System MUST validate flag dependencies at argument parsing (e.g., `--prompt-file` and `--model` require `--ai`)
- **FR-017**: System MUST validate prompt file size (maximum 32KB) before reading content

### Constitution Compliance

- **Configuration**: MUST use `OPENAI_API_KEY` environment variable via .env file, never hardcode API keys
- **Authentication**: MUST use API key authentication with OpenAI REST API (Bearer token in Authorization header)
- **Error handling**: MUST provide actionable error messages per Error Transparency principle (e.g., "Missing OPENAI_API_KEY. Add it to your .env file with: OPENAI_API_KEY=sk-...")
- **CLI-First**: All AI features MUST be accessible via command-line with proper --help documentation
- **Argument validation**: Use clap's validation features to enforce flag dependencies and constraints at parse time
- **Unit handling**: Weather data sent to AI MUST preserve existing unit formats from forecast providers
- **HTTP Client**: MUST use `reqwest` crate (async) for OpenAI API requests due to its robust error handling, connection pooling, and wide ecosystem support
- **Timeout**: MUST set 60-second timeout for all OpenAI API requests to allow complex analysis while preventing indefinite hangs

### Key Entities

- **AI Prompt**: Text input (from user or file) that describes what analysis the user wants from the forecast (sent as user message)
- **OpenAI Model**: Identifier for which ChatGPT model to use (currently: "gpt-5" or "gpt-4o")
- **AI Response**: Natural language analysis returned from OpenAI API based on forecast data and prompt
- **Forecast Context**: The JSON forecast data that is sent to AI in the system message role to provide context for analysis
- **Message Structure**: Two-message conversation pattern: system message (forecast JSON) + user message (analysis prompt)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can enable AI analysis by adding a single flag (`--ai`) to their existing forecast command
- **SC-002**: Users receive AI responses within typical API latency times, with requests timing out after 60 seconds if OpenAI does not respond
- **SC-003**: System successfully handles 100% of supported OpenAI model names without errors
- **SC-004**: Error messages provide clear next steps for 100% of common failure scenarios (missing API key, invalid model, network errors)
- **SC-005**: Users can automate forecast analysis by using prompt files, eliminating need for manual prompt entry
- **SC-006**: System gracefully degrades when AI features are unavailable, allowing core forecast functionality to continue working

### Assumptions

- OpenAI REST API endpoint (https://api.openai.com/v1/chat/completions) is used for all AI requests
- API key authentication follows standard Bearer token pattern in Authorization header
- OpenAI API pricing and rate limits are acceptable for typical usage patterns
- Users have network connectivity to reach OpenAI API endpoints (api.openai.com)
- Forecast JSON data size will be within OpenAI's context window limits
- Default model (gpt-5) provides sufficient quality for general forecast analysis
- Users understand that AI analysis quality depends on the model selected and prompt quality
- Application will use async runtime (tokio) to support reqwest's async HTTP operations

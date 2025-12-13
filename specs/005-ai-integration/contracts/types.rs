// AI Integration Type Contracts
// Feature: 005-ai-integration
// These types define the contract for OpenAI API integration

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

// ============================================================================
// ChatModel Enum - Supported OpenAI Models
// ============================================================================

/// Represents the OpenAI ChatGPT model to use for forecast analysis.
/// 
/// Validated at CLI parsing time using clap's ValueEnum trait.
/// Invalid model names cause argument parsing failure.
#[derive(Debug, Clone, ValueEnum)]
pub enum ChatModel {
    /// GPT-5 model (default) - Latest and most capable
    #[value(name = "gpt-5")]
    Gpt5,
    
    /// GPT-4o model - Alternative option
    #[value(name = "gpt-4o")]
    Gpt4o,
}

impl ChatModel {
    /// Returns the API-compatible model name string.
    /// 
    /// Used when constructing ChatRequest for OpenAI API.
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

// ============================================================================
// ChatMessage - Individual Message in Conversation
// ============================================================================

/// Represents a single message in the OpenAI chat conversation.
/// 
/// Used in both request (system, user) and response (assistant) messages.
/// 
/// # Message Roles
/// - `system`: Provides context (forecast JSON) to the AI
/// - `user`: Contains the user's analysis prompt/question
/// - `assistant`: AI's response message (in ChatResponse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role: "system", "user", or "assistant"
    pub role: String,
    
    /// Message content (JSON string for system, text for user/assistant)
    pub content: String,
}

impl ChatMessage {
    /// Creates a system message with the given content.
    /// 
    /// System messages provide context to the AI.
    /// For this feature, contains the forecast JSON.
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
        }
    }
    
    /// Creates a user message with the given content.
    /// 
    /// User messages contain the analysis prompt or question.
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }
}

// ============================================================================
// ChatRequest - OpenAI API Request Structure
// ============================================================================

/// Complete request body for OpenAI Chat Completions API.
/// 
/// # Required Fields
/// - `model`: Model identifier (from ChatModel enum)
/// - `messages`: Exactly 2 messages (system with forecast + user with prompt)
/// 
/// # Optional Fields
/// - `temperature`: Sampling temperature (0.0-2.0, default: 1.0)
/// - `max_tokens`: Maximum response tokens (default: model limit)
/// 
/// # Example
/// ```rust
/// let request = ChatRequest::new(
///     ChatModel::Gpt5,
///     forecast_json,
///     "Should I go sailing tomorrow?".to_string()
/// );
/// ```
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    /// Model identifier (e.g., "gpt-5", "gpt-4o")
    pub model: String,
    
    /// Array of messages: [system with forecast, user with prompt]
    pub messages: Vec<ChatMessage>,
    
    /// Optional: Sampling temperature (0.0 = deterministic, 2.0 = very random)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Optional: Maximum tokens in response (limits output length)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    /// Creates a new ChatRequest with the standard two-message pattern.
    /// 
    /// # Arguments
    /// - `model`: ChatGPT model to use
    /// - `forecast_json`: Serialized forecast data (system message)
    /// - `prompt`: User's analysis question (user message)
    /// 
    /// # Returns
    /// ChatRequest ready to send to OpenAI API
    pub fn new(model: ChatModel, forecast_json: String, prompt: String) -> Self {
        Self {
            model: model.api_name().to_string(),
            messages: vec![
                ChatMessage::system(forecast_json),
                ChatMessage::user(prompt),
            ],
            temperature: None,      // Use OpenAI default (1.0)
            max_tokens: None,       // Use model's maximum
        }
    }
}

// ============================================================================
// ChatChoice - Single Completion Choice in Response
// ============================================================================

/// Represents one completion choice from the OpenAI API response.
/// 
/// The API can return multiple choices, but we only use the first one.
#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    /// Choice index (0 for first choice)
    pub index: u32,
    
    /// Response message with "assistant" role
    pub message: ChatMessage,
    
    /// Why generation stopped: "stop" (complete), "length" (truncated), etc.
    pub finish_reason: Option<String>,
}

// ============================================================================
// TokenUsage - Token Statistics
// ============================================================================

/// Token usage statistics from OpenAI API response.
/// 
/// Useful for cost tracking and debugging context window issues.
#[derive(Debug, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt (forecast + user question)
    pub prompt_tokens: u32,
    
    /// Tokens in the AI's response
    pub completion_tokens: u32,
    
    /// Total tokens used (prompt + completion)
    pub total_tokens: u32,
}

// ============================================================================
// ChatResponse - OpenAI API Response Structure
// ============================================================================

/// Complete response from OpenAI Chat Completions API.
/// 
/// # Response Structure
/// ```json
/// {
///   "id": "chatcmpl-abc123",
///   "object": "chat.completion",
///   "created": 1677652288,
///   "model": "gpt-5",
///   "choices": [
///     {
///       "index": 0,
///       "message": {
///         "role": "assistant",
///         "content": "AI analysis here..."
///       },
///       "finish_reason": "stop"
///     }
///   ],
///   "usage": {
///     "prompt_tokens": 100,
///     "completion_tokens": 50,
///     "total_tokens": 150
///   }
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// Unique response identifier
    pub id: String,
    
    /// Object type (should be "chat.completion")
    pub object: String,
    
    /// Unix timestamp when response was created
    pub created: u64,
    
    /// Model that generated the response
    pub model: String,
    
    /// Array of completion choices (we use first one)
    pub choices: Vec<ChatChoice>,
    
    /// Optional token usage statistics
    pub usage: Option<TokenUsage>,
}

impl ChatResponse {
    /// Extracts the AI analysis text from the first choice.
    /// 
    /// # Returns
    /// - `Ok(String)`: The AI's analysis text
    /// - `Err(anyhow::Error)`: If choices array is empty
    /// 
    /// # Example
    /// ```rust
    /// let analysis = response.extract_content()?;
    /// println!("{}", analysis);
    /// ```
    pub fn extract_content(&self) -> anyhow::Result<String> {
        self.choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| anyhow::anyhow!("No choices in OpenAI response"))
    }
    
    /// Checks if the response was truncated due to length limits.
    /// 
    /// # Returns
    /// `true` if finish_reason is "length" (truncated), `false` otherwise
    pub fn is_truncated(&self) -> bool {
        self.choices
            .first()
            .and_then(|choice| choice.finish_reason.as_deref())
            .map(|reason| reason == "length")
            .unwrap_or(false)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_model_api_name() {
        assert_eq!(ChatModel::Gpt5.api_name(), "gpt-5");
        assert_eq!(ChatModel::Gpt4o.api_name(), "gpt-4o");
    }

    #[test]
    fn test_chat_model_default() {
        let default = ChatModel::default();
        assert_eq!(default.api_name(), "gpt-5");
    }

    #[test]
    fn test_chat_message_constructors() {
        let system_msg = ChatMessage::system("context".to_string());
        assert_eq!(system_msg.role, "system");
        assert_eq!(system_msg.content, "context");

        let user_msg = ChatMessage::user("question".to_string());
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "question");
    }

    #[test]
    fn test_chat_request_creation() {
        let request = ChatRequest::new(
            ChatModel::Gpt5,
            "forecast json".to_string(),
            "user prompt".to_string(),
        );

        assert_eq!(request.model, "gpt-5");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "forecast json");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "user prompt");
    }

    #[test]
    fn test_chat_response_extract_content() {
        let response = ChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 123456,
            model: "gpt-5".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: "AI analysis".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        assert_eq!(response.extract_content().unwrap(), "AI analysis");
    }

    #[test]
    fn test_chat_response_is_truncated() {
        let truncated = ChatResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 123456,
            model: "gpt-5".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: "truncated...".to_string(),
                },
                finish_reason: Some("length".to_string()),
            }],
            usage: None,
        };

        assert!(truncated.is_truncated());
    }
}
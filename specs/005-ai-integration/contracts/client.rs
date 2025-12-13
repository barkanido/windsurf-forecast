// AI Integration HTTP Client Contract
// Feature: 005-ai-integration
// Defines the interface for OpenAI API HTTP client

use super::errors::AIError;
use super::types::{ChatRequest, ChatResponse};
use reqwest::Client;
use std::time::Duration;

/// OpenAI API client for chat completions.
/// 
/// Handles HTTP communication with OpenAI's REST API, including:
/// - Authentication via Bearer token
/// - Request/response serialization
/// - Error handling and mapping
/// - Timeout enforcement (60 seconds)
/// 
/// # Example
/// ```rust
/// let client = AIClient::new("sk-...")?;
/// let response = client.chat_completion(request).await?;
/// let analysis = response.extract_content()?;
/// ```
pub struct AIClient {
    /// HTTP client with 60-second timeout
    client: Client,
    
    /// OpenAI API key for authentication
    api_key: String,
}

impl AIClient {
    /// OpenAI Chat Completions API endpoint
    const ENDPOINT: &'static str = "https://api.openai.com/v1/chat/completions";
    
    /// Request timeout duration (60 seconds)
    const TIMEOUT: Duration = Duration::from_secs(60);
    
    /// Creates a new AI client with the provided API key.
    /// 
    /// # Arguments
    /// - `api_key`: OpenAI API key (from OPENAI_API_KEY env var)
    /// 
    /// # Returns
    /// - `Ok(AIClient)`: Client ready to make API calls
    /// - `Err(AIError)`: If client creation fails
    /// 
    /// # Example
    /// ```rust
    /// let api_key = std::env::var("OPENAI_API_KEY")?;
    /// let client = AIClient::new(api_key)?;
    /// ```
    pub fn new(api_key: String) -> Result<Self, AIError> {
        let client = Client::builder()
            .timeout(Self::TIMEOUT)
            .build()
            .map_err(|e| AIError::ApiError {
                status: 0,
                message: format!("Failed to create HTTP client: {}", e),
            })?;
        
        Ok(Self { client, api_key })
    }
    
    /// Loads API key from OPENAI_API_KEY environment variable.
    /// 
    /// # Returns
    /// - `Ok(String)`: API key value
    /// - `Err(AIError::MissingApiKey)`: If env var not set
    /// 
    /// # Example
    /// ```rust
    /// let api_key = AIClient::load_api_key()?;
    /// let client = AIClient::new(api_key)?;
    /// ```
    pub fn load_api_key() -> Result<String, AIError> {
        std::env::var("OPENAI_API_KEY")
            .map_err(|_| AIError::MissingApiKey)
    }
    
    /// Sends a chat completion request to OpenAI API.
    /// 
    /// # Arguments
    /// - `request`: ChatRequest with model, messages, and options
    /// 
    /// # Returns
    /// - `Ok(ChatResponse)`: Successful API response with AI analysis
    /// - `Err(AIError)`: Various failure modes:
    ///   - `Timeout`: Request took longer than 60 seconds
    ///   - `RateLimit`: HTTP 429 rate limit exceeded
    ///   - `ApiError`: Other HTTP errors (401, 500, etc.)
    ///   - `InvalidResponse`: Response parsing failed
    /// 
    /// # Example
    /// ```rust
    /// let request = ChatRequest::new(
    ///     ChatModel::Gpt5,
    ///     forecast_json,
    ///     "Should I go sailing?".to_string()
    /// );
    /// let response = client.chat_completion(request).await?;
    /// ```
    pub async fn chat_completion(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, AIError> {
        // Send POST request with Bearer token authentication
        let response = self
            .client
            .post(Self::ENDPOINT)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AIError::Timeout
                } else {
                    AIError::ApiError {
                        status: 0,
                        message: format!("Request failed: {}", e),
                    }
                }
            })?;
        
        // Check HTTP status code
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(AIError::from_status(status.as_u16(), error_body));
        }
        
        // Parse response JSON
        response
            .json::<ChatResponse>()
            .await
            .map_err(|e| AIError::InvalidResponse(e.to_string()))
    }
}

// ============================================================================
// Module-level Helper Functions
// ============================================================================

/// High-level function to analyze forecast with AI.
/// 
/// This is the main entry point for AI analysis, wrapping all the steps:
/// 1. Load API key from environment
/// 2. Create HTTP client
/// 3. Create request with forecast and prompt
/// 4. Send request to OpenAI
/// 5. Extract and return analysis text
/// 
/// # Arguments
/// - `forecast_json`: Serialized forecast data (system message context)
/// - `prompt`: User's analysis question (user message)
/// - `model`: ChatGPT model to use
/// 
/// # Returns
/// - `Ok(String)`: AI analysis text
/// - `Err(AIError)`: Any failure in the pipeline
/// 
/// # Example
/// ```rust
/// let analysis = analyze_forecast(
///     forecast_json,
///     "Is it safe to windsurf?".to_string(),
///     ChatModel::Gpt5
/// ).await?;
/// println!("{}", analysis);
/// ```
pub async fn analyze_forecast(
    forecast_json: String,
    prompt: String,
    model: super::types::ChatModel,
) -> Result<String, AIError> {
    // Load API key
    let api_key = AIClient::load_api_key()?;
    
    // Create client
    let client = AIClient::new(api_key)?;
    
    // Create request
    let request = ChatRequest::new(model, forecast_json, prompt);
    
    // Send request
    let response = client.chat_completion(request).await?;
    
    // Extract content
    let content = response.extract_content()
        .map_err(|_| AIError::EmptyResponse)?;
    
    // Check for truncation and warn user
    if response.is_truncated() {
        Ok(format!(
            "{}\n\nNote: Response may be incomplete due to length limits",
            content
        ))
    } else {
        Ok(content)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_constant() {
        assert_eq!(
            AIClient::ENDPOINT,
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn test_timeout_constant() {
        assert_eq!(AIClient::TIMEOUT, Duration::from_secs(60));
    }

    #[test]
    fn test_load_api_key_missing() {
        // Clear the env var for this test
        std::env::remove_var("OPENAI_API_KEY");
        
        let result = AIClient::load_api_key();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::MissingApiKey));
    }

    #[test]
    fn test_load_api_key_success() {
        // Set env var for this test
        std::env::set_var("OPENAI_API_KEY", "sk-test123");
        
        let result = AIClient::load_api_key();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test123");
        
        // Clean up
        std::env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_client_creation() {
        let client = AIClient::new("sk-test".to_string());
        assert!(client.is_ok());
    }
}
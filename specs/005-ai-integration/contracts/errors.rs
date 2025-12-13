// AI Integration Error Contracts
// Feature: 005-ai-integration
// These error types follow the Error Transparency constitution principle

use thiserror::Error;

/// Custom error types for AI integration failures.
/// 
/// All error messages are actionable and guide users toward resolution,
/// following the Error Transparency principle from the constitution.
/// 
/// # Error Categories
/// - Configuration errors: MissingApiKey
/// - Network errors: Timeout, RateLimit, ApiError
/// - Prompt errors: PromptFileNotFound, PromptFileTooLarge, PromptFileEmpty, EmptyPrompt
/// - Response errors: InvalidResponse, EmptyResponse
#[derive(Error, Debug)]
pub enum AIError {
    /// OpenAI API key not found in environment.
    /// 
    /// Provides clear guidance on how to set the API key.
    #[error("OPENAI_API_KEY not found. Add it to your .env file with: OPENAI_API_KEY=sk-...")]
    MissingApiKey,
    
    /// Request exceeded the 60-second timeout.
    /// 
    /// Suggests retry as OpenAI API may be slow under load.
    #[error("OpenAI API request timed out after 60 seconds. Please try again.")]
    Timeout,
    
    /// OpenAI rate limit exceeded (HTTP 429).
    /// 
    /// Directs user to check their quota on the platform.
    #[error("OpenAI API rate limit exceeded. Please try again later or check your quota at platform.openai.com")]
    RateLimit,
    
    /// Prompt file specified but doesn't exist.
    /// 
    /// Shows the exact path that was not found.
    #[error("Prompt file not found: {0}")]
    PromptFileNotFound(String),
    
    /// Prompt file exceeds 32KB size limit.
    /// 
    /// Shows actual file size to help user understand the issue.
    #[error("Prompt file too large: {size}. Maximum allowed: 32KB")]
    PromptFileTooLarge {
        /// Human-readable file size (e.g., "50KB")
        size: String,
    },
    
    /// Prompt file exists but contains no content.
    /// 
    /// Helps distinguish from file-not-found errors.
    #[error("Prompt file is empty: {0}")]
    PromptFileEmpty(String),
    
    /// Generic OpenAI API error with status code.
    /// 
    /// Provides HTTP status and message from API response.
    /// Common status codes:
    /// - 401: Invalid API key
    /// - 402: Billing issue
    /// - 403: Permission denied
    /// - 500: Server error
    #[error("OpenAI API error ({status}): {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from API
        message: String,
    },
    
    /// Response from OpenAI could not be parsed.
    /// 
    /// Indicates unexpected response format or corruption.
    #[error("Invalid OpenAI response: {0}")]
    InvalidResponse(String),
    
    /// User provided empty prompt when prompted interactively.
    /// 
    /// Prevents sending empty questions to the API.
    #[error("Empty prompt provided. Please enter a question about the forecast.")]
    EmptyPrompt,
    
    /// OpenAI response had no choices array or it was empty.
    /// 
    /// Indicates malformed API response.
    #[error("OpenAI response contains no choices")]
    EmptyResponse,
    
    /// Forecast data is invalid or malformed.
    /// 
    /// Prevents sending invalid data to OpenAI API.
    #[error("Cannot perform AI analysis: forecast data is invalid")]
    InvalidForecast,
}

impl AIError {
    /// Creates an ApiError from HTTP status code and message.
    /// 
    /// # Arguments
    /// - `status`: HTTP status code (e.g., 401, 429, 500)
    /// - `message`: Error message from API response body
    /// 
    /// # Returns
    /// Appropriate AIError variant based on status code
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            429 => AIError::RateLimit,
            _ => AIError::ApiError { status, message },
        }
    }
    
    /// Checks if this error is retryable.
    /// 
    /// # Returns
    /// `true` if the error is transient and might succeed on retry
    /// 
    /// # Retryable Errors
    /// - Timeout (server may be slow)
    /// - RateLimit (after waiting)
    /// - ApiError with 5xx status (server issues)
    pub fn is_retryable(&self) -> bool {
        match self {
            AIError::Timeout => true,
            AIError::RateLimit => true,
            AIError::ApiError { status, .. } => *status >= 500,
            _ => false,
        }
    }
}

/// Converts std::io::Error to AIError.
/// 
/// Maps I/O errors from file operations to appropriate AIError variants.
impl From<std::io::Error> for AIError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                AIError::PromptFileNotFound("unknown".to_string())
            }
            std::io::ErrorKind::TimedOut => AIError::Timeout,
            _ => AIError::ApiError {
                status: 0,
                message: err.to_string(),
            },
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_status_rate_limit() {
        let error = AIError::from_status(429, "Rate limit exceeded".to_string());
        assert!(matches!(error, AIError::RateLimit));
    }

    #[test]
    fn test_from_status_generic() {
        let error = AIError::from_status(500, "Internal error".to_string());
        assert!(matches!(error, AIError::ApiError { status: 500, .. }));
    }

    #[test]
    fn test_is_retryable_timeout() {
        let error = AIError::Timeout;
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let error = AIError::RateLimit;
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_retryable_server_error() {
        let error = AIError::ApiError {
            status: 500,
            message: "Server error".to_string(),
        };
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_missing_key() {
        let error = AIError::MissingApiKey;
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_empty_prompt() {
        let error = AIError::EmptyPrompt;
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_error_messages() {
        // Verify error messages are actionable
        let missing_key = AIError::MissingApiKey;
        assert!(missing_key.to_string().contains(".env"));
        assert!(missing_key.to_string().contains("OPENAI_API_KEY"));

        let timeout = AIError::Timeout;
        assert!(timeout.to_string().contains("60 seconds"));
        assert!(timeout.to_string().contains("try again"));

        let rate_limit = AIError::RateLimit;
        assert!(rate_limit.to_string().contains("platform.openai.com"));
        assert!(rate_limit.to_string().contains("quota"));
    }
}
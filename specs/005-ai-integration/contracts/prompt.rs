// AI Integration Prompt Management Contract
// Feature: 005-ai-integration
// Defines the interface for loading and validating prompts

use super::errors::AIError;
use std::fs;
use std::io::{self, Write};

/// Maximum allowed prompt file size (32KB = 32768 bytes).
/// 
/// This limit prevents:
/// - Excessive API costs from large prompts
/// - Token limit issues with OpenAI API
/// - Accidental use of large files as prompts
const MAX_PROMPT_SIZE: u64 = 32 * 1024; // 32KB

/// Loads a prompt from file or interactive input.
/// 
/// # Behavior
/// - If `prompt_file` is `Some(path)`: Loads prompt from file with validation
/// - If `prompt_file` is `None`: Prompts user for interactive input
/// 
/// # File Validation
/// 1. File must exist and be readable
/// 2. File size must be ≤ 32KB
/// 3. File content must not be empty (after trimming whitespace)
/// 4. File must be valid UTF-8
/// 
/// # Arguments
/// - `prompt_file`: Optional path to prompt file
/// 
/// # Returns
/// - `Ok(String)`: Valid prompt text (trimmed)
/// - `Err(AIError)`: Various validation failures
/// 
/// # Errors
/// - `PromptFileNotFound`: File doesn't exist
/// - `PromptFileTooLarge`: File exceeds 32KB
/// - `PromptFileEmpty`: File has no content
/// - `EmptyPrompt`: User entered empty input interactively
/// - IO errors: File read failures
/// 
/// # Example
/// ```rust
/// // From file
/// let prompt = load_prompt(Some("my-prompt.txt"))?;
/// 
/// // Interactive
/// let prompt = load_prompt(None)?;
/// ```
pub fn load_prompt(prompt_file: Option<&str>) -> Result<String, AIError> {
    match prompt_file {
        Some(path) => load_prompt_from_file(path),
        None => load_prompt_interactive(),
    }
}

/// Loads prompt from a file with validation.
/// 
/// # Validation Steps
/// 1. Check file exists (metadata call)
/// 2. Check file size ≤ 32KB
/// 3. Read file as UTF-8
/// 4. Check content is not empty
/// 
/// # Arguments
/// - `path`: Path to prompt file
/// 
/// # Returns
/// - `Ok(String)`: Valid prompt text (trimmed)
/// - `Err(AIError)`: Validation failure
fn load_prompt_from_file(path: &str) -> Result<String, AIError> {
    // Check file exists and get size
    let metadata = fs::metadata(path)
        .map_err(|_| AIError::PromptFileNotFound(path.to_string()))?;
    
    // Validate file size
    let size = metadata.len();
    if size > MAX_PROMPT_SIZE {
        let size_kb = size / 1024;
        return Err(AIError::PromptFileTooLarge {
            size: format!("{}KB", size_kb),
        });
    }
    
    // Read file content as UTF-8
    let content = fs::read_to_string(path)
        .map_err(|e| AIError::ApiError {
            status: 0,
            message: format!("Failed to read prompt file: {}", e),
        })?;
    
    // Validate not empty
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err(AIError::PromptFileEmpty(path.to_string()));
    }
    
    Ok(trimmed.to_string())
}

/// Prompts user for interactive input via stdin.
/// 
/// Displays a prompt message and waits for user input.
/// Input is validated to ensure it's not empty.
/// 
/// # Returns
/// - `Ok(String)`: User's input (trimmed)
/// - `Err(AIError)`: Empty input or IO error
fn load_prompt_interactive() -> Result<String, AIError> {
    // Display prompt
    print!("Enter your forecast analysis question: ");
    io::stdout()
        .flush()
        .map_err(|e| AIError::ApiError {
            status: 0,
            message: format!("Failed to flush stdout: {}", e),
        })?;
    
    // Read input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| AIError::ApiError {
            status: 0,
            message: format!("Failed to read input: {}", e),
        })?;
    
    // Validate not empty
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AIError::EmptyPrompt);
    }
    
    Ok(trimmed.to_string())
}

/// Validates a prompt string.
/// 
/// Ensures prompt is not empty after trimming whitespace.
/// Useful for additional validation after loading.
/// 
/// # Arguments
/// - `prompt`: Prompt text to validate
/// 
/// # Returns
/// - `Ok(())`: Prompt is valid
/// - `Err(AIError::EmptyPrompt)`: Prompt is empty
pub fn validate_prompt(prompt: &str) -> Result<(), AIError> {
    if prompt.trim().is_empty() {
        Err(AIError::EmptyPrompt)
    } else {
        Ok(())
    }
}

/// Formats a prompt file size in human-readable format.
/// 
/// Used in error messages to show file sizes.
/// 
/// # Arguments
/// - `bytes`: File size in bytes
/// 
/// # Returns
/// Human-readable size string (e.g., "45KB")
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else {
        format!("{}KB", bytes / 1024)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_max_prompt_size() {
        assert_eq!(MAX_PROMPT_SIZE, 32768);
    }

    #[test]
    fn test_load_prompt_from_nonexistent_file() {
        let result = load_prompt_from_file("/nonexistent/file.txt");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::PromptFileNotFound(_)));
    }

    #[test]
    fn test_load_prompt_from_valid_file() -> anyhow::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test prompt content")?;
        
        let path = temp_file.path().to_str().unwrap();
        let result = load_prompt_from_file(path)?;
        
        assert_eq!(result, "Test prompt content");
        Ok(())
    }

    #[test]
    fn test_load_prompt_from_empty_file() -> anyhow::Result<()> {
        let temp_file = NamedTempFile::new()?;
        
        let path = temp_file.path().to_str().unwrap();
        let result = load_prompt_from_file(path);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::PromptFileEmpty(_)));
        Ok(())
    }

    #[test]
    fn test_load_prompt_from_whitespace_only_file() -> anyhow::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "   \n\t  \n   ")?;
        
        let path = temp_file.path().to_str().unwrap();
        let result = load_prompt_from_file(path);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::PromptFileEmpty(_)));
        Ok(())
    }

    #[test]
    fn test_load_prompt_trims_whitespace() -> anyhow::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "  \n  Test prompt  \n  ")?;
        
        let path = temp_file.path().to_str().unwrap();
        let result = load_prompt_from_file(path)?;
        
        assert_eq!(result, "Test prompt");
        Ok(())
    }

    #[test]
    fn test_validate_prompt_valid() {
        let result = validate_prompt("Valid prompt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_empty() {
        let result = validate_prompt("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::EmptyPrompt));
    }

    #[test]
    fn test_validate_prompt_whitespace_only() {
        let result = validate_prompt("   \n\t  ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AIError::EmptyPrompt));
    }

    #[test]
    fn test_format_file_size_bytes() {
        assert_eq!(format_file_size(512), "512B");
    }

    #[test]
    fn test_format_file_size_kilobytes() {
        assert_eq!(format_file_size(2048), "2KB");
        assert_eq!(format_file_size(32768), "32KB");
    }
}
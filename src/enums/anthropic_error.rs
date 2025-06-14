use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum AnthropicError {
    ApiError(String),
    NetworkError(String),
    SerializationError(String),
    AuthenticationError(String),
}

impl fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnthropicError::ApiError(msg) => write!(f, "Anthropic API Error: {}", msg),
            AnthropicError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            AnthropicError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            AnthropicError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
        }
    }
}

impl Error for AnthropicError {}
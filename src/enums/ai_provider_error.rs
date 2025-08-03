use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum AiProviderError {
    ApiError(String),
    NetworkError(String),
    SerializationError(String),
    AuthenticationError(String),
}

impl fmt::Display for AiProviderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AiProviderError::ApiError(msg) => write!(f, "Anthropic API Error: {}", msg),
            AiProviderError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            AiProviderError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            AiProviderError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
        }
    }
}

impl Error for AiProviderError {}
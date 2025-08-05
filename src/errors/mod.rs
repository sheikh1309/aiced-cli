use std::fmt;
use std::error::Error as StdError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AicedError {
    ConfigurationError {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },
    ConfigurationFileError {
        path: String,
        reason: String,
    },

    RepositoryError {
        repository: String,
        operation: String,
        reason: String,
    },
    RepositoryNotFound {
        name: String,
        available: Vec<String>,
    },

    FileOperationError {
        file_path: String,
        operation: String,
        reason: String,
    },
    FileValidationError {
        file_path: String,
        line_number: Option<usize>,
        expected: String,
        actual: String,
    },

    ParseError {
        content_type: String,
        line_number: Option<usize>,
        reason: String,
        context: Option<String>,
    },

    AnalysisError {
        repository: String,
        stage: String,
        reason: String,
        recoverable: bool,
    },

    NetworkError {
        operation: String,
        url: Option<String>,
        status_code: Option<u16>,
        reason: String,
    },

    ValidationError {
        field: String,
        value: String,
        constraint: String,
        suggestion: Option<String>,
    },

    SystemError {
        operation: String,
        reason: String,
    },

    UserInputError {
        input: String,
        expected: String,
        suggestion: String,
    },

    MultipleErrors {
        errors: Vec<AicedError>,
        context: String,
    },
}

impl AicedError {
    pub fn config_error(message: &str, field: Option<&str>, suggestion: Option<&str>) -> Self {
        Self::ConfigurationError {
            message: message.to_string(),
            field: field.map(|s| s.to_string()),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    pub fn configuration_error(message: &str, field: Option<&str>, suggestion: Option<&str>) -> Self {
        Self::ConfigurationError {
            message: message.to_string(),
            field: field.map(|s| s.to_string()),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    pub fn repo_error(repository: &str, operation: &str, reason: &str) -> Self {
        Self::RepositoryError {
            repository: repository.to_string(),
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn file_error(file_path: &str, operation: &str, reason: &str) -> Self {
        Self::FileOperationError {
            file_path: file_path.to_string(),
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn system_error(operation: &str, reason: &str) -> Self {
        Self::SystemError {
            operation: operation.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn parse_error(content_type: &str, line_number: Option<usize>, reason: &str, context: Option<&str>) -> Self {
        Self::ParseError {
            content_type: content_type.to_string(),
            line_number,
            reason: reason.to_string(),
            context: context.map(|s| s.to_string()),
        }
    }

    pub fn validation_error(field: &str, value: &str, constraint: &str, suggestion: Option<&str>) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            value: value.to_string(),
            constraint: constraint.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }

    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::AnalysisError { recoverable, .. } => *recoverable,
            Self::NetworkError { .. } => true,
            Self::UserInputError { .. } => true,
            Self::ValidationError { .. } => true,
            Self::ConfigurationError { .. } => true,
            Self::FileValidationError { .. } => false,
            Self::RepositoryNotFound { .. } => false,
            Self::SystemError { .. } => false,
            Self::MultipleErrors { errors, .. } => errors.iter().any(|e| e.is_recoverable()),
            _ => false,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::SystemError { .. } => ErrorSeverity::Critical,
            Self::FileOperationError { .. } => ErrorSeverity::High,
            Self::AnalysisError { .. } => ErrorSeverity::High,
            Self::ConfigurationFileError { .. } => ErrorSeverity::High,
            Self::RepositoryNotFound { .. } => ErrorSeverity::Medium,
            Self::ParseError { .. } => ErrorSeverity::Medium,
            Self::NetworkError { .. } => ErrorSeverity::Medium,
            Self::ValidationError { .. } => ErrorSeverity::Low,
            Self::ConfigurationError { .. } => ErrorSeverity::Low,
            Self::UserInputError { .. } => ErrorSeverity::Low,
            Self::FileValidationError { .. } => ErrorSeverity::Medium,
            Self::RepositoryError { .. } => ErrorSeverity::Medium,
            Self::MultipleErrors { errors, .. } => {
                errors.iter()
                    .map(|e| e.severity())
                    .max()
                    .unwrap_or(ErrorSeverity::Low)
            }
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::ConfigurationError { message, field, suggestion } => {
                let mut msg = format!("Configuration Error: {}", message);
                if let Some(field) = field {
                    msg.push_str(&format!(" (field: {})", field));
                }
                if let Some(suggestion) = suggestion {
                    msg.push_str(&format!("\n💡 Suggestion: {}", suggestion));
                }
                msg
            }
            Self::ConfigurationFileError { path, reason } => {
                format!("Configuration file error at '{}': {}\n💡 Check file permissions and syntax", path, reason)
            }
            Self::RepositoryError { repository, operation, reason } => {
                format!("Repository '{}' error during {}: {}\n💡 Check repository path and permissions", repository, operation, reason)
            }
            Self::RepositoryNotFound { name, available } => {
                let mut msg = format!("Repository '{}' not found", name);
                if !available.is_empty() {
                    msg.push_str(&format!("\n💡 Available repositories: {}", available.join(", ")));
                }
                msg
            }
            Self::FileOperationError { file_path, operation, reason } => {
                format!("File operation '{}' failed for '{}': {}\n💡 Check file permissions and path", operation, file_path, reason)
            }
            Self::FileValidationError { file_path, line_number, expected, actual } => {
                let mut msg = format!("File validation failed for '{}'", file_path);
                if let Some(line) = line_number {
                    msg.push_str(&format!(" at line {}", line));
                }
                msg.push_str(&format!("\nExpected: '{}'\nActual: '{}'", expected, actual));
                msg.push_str("\n💡 File may have been modified since analysis");
                msg
            }
            Self::ParseError { content_type, line_number, reason, context } => {
                let mut msg = format!("Parse error in {}: {}", content_type, reason);
                if let Some(line) = line_number {
                    msg.push_str(&format!(" (line {})", line));
                }
                if let Some(ctx) = context {
                    msg.push_str(&format!("\nContext: {}", ctx));
                }
                msg.push_str("\n💡 Check the format and syntax of the input");
                msg
            }
            Self::AnalysisError { repository, stage, reason, recoverable } => {
                let mut msg = format!("Analysis error in repository '{}' during {}: {}", repository, stage, reason);
                if *recoverable {
                    msg.push_str("\n💡 This error is recoverable - you can retry the operation");
                } else {
                    msg.push_str("\n⚠️ This error requires manual intervention");
                }
                msg
            }
            Self::NetworkError { operation, url, status_code, reason } => {
                let mut msg = format!("Network error during {}: {}", operation, reason);
                if let Some(url) = url {
                    msg.push_str(&format!(" (URL: {})", url));
                }
                if let Some(code) = status_code {
                    msg.push_str(&format!(" (Status: {})", code));
                }
                msg.push_str("\n💡 Check your internet connection and try again");
                msg
            }
            Self::ValidationError { field, value, constraint, suggestion } => {
                let mut msg = format!("Validation error for field '{}': value '{}' violates constraint '{}'", field, value, constraint);
                if let Some(suggestion) = suggestion {
                    msg.push_str(&format!("\n💡 Suggestion: {}", suggestion));
                }
                msg
            }
            Self::SystemError { operation, reason } => {
                format!("System error during {}: {}", operation, reason)
            }
            Self::UserInputError { input, expected, suggestion } => {
                format!("Invalid input '{}': expected {}\n💡 {}", input, expected, suggestion)
            }
            Self::MultipleErrors { errors, context } => {
                let mut msg = format!("Multiple errors occurred during {}:\n", context);
                for (i, error) in errors.iter().enumerate() {
                    msg.push_str(&format!("  {}. {}\n", i + 1, error.user_message().replace('\n', "\n     ")));
                }
                msg
            }
        }
    }

    pub fn technical_details(&self) -> String {
        format!("{:?}", self)
    }
}

impl fmt::Display for AicedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl StdError for AicedError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Low => "🟢",
            Self::Medium => "🟡",
            Self::High => "🟠",
            Self::Critical => "🔴",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

pub type AicedResult<T> = Result<T, AicedError>;

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn handle_error(error: &AicedError) {
        let severity = error.severity();

        log::error!("[{}] {}", severity.name(), error.technical_details());

        log::error!("{} {}", severity.emoji(), error.user_message());

        match severity {
            ErrorSeverity::Critical => {
                log::error!("🚨 Critical error detected - application may need to exit");
            }
            ErrorSeverity::High => {
                log::error!("⚠️ High severity error - operation failed");
            }
            ErrorSeverity::Medium => {
                log::error!("⚠️ Error occurred - some functionality may be affected");
            }
            ErrorSeverity::Low => {
                log::error!("ℹ️ Minor issue detected - operation can continue");
            }
        }

        if error.is_recoverable() {
            log::error!("🔄 This error is recoverable - you can retry the operation");
        }
    }
}

impl From<std::io::Error> for AicedError {
    fn from(error: std::io::Error) -> Self {
        AicedError::SystemError {
            operation: "I/O operation".to_string(),
            reason: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for AicedError {
    fn from(error: serde_json::Error) -> Self {
        AicedError::ParseError {
            content_type: "JSON".to_string(),
            line_number: Some(error.line()),
            reason: error.to_string(),
            context: None,
        }
    }
}

impl From<toml::de::Error> for AicedError {
    fn from(error: toml::de::Error) -> Self {
        AicedError::ParseError {
            content_type: "TOML".to_string(),
            line_number: None,
            reason: error.message().to_string(),
            context: None,
        }
    }
}

impl From<reqwest::Error> for AicedError {
    fn from(error: reqwest::Error) -> Self {
        AicedError::NetworkError {
            operation: "HTTP request".to_string(),
            url: error.url().map(|u| u.to_string()),
            status_code: error.status().map(|s| s.as_u16()),
            reason: error.to_string(),
        }
    }
}


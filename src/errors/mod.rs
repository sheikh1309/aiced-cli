use std::fmt;
use std::error::Error as StdError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AilyzerError {
    // Configuration errors
    ConfigurationError {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
    },
    ConfigurationFileError {
        path: String,
        reason: String,
    },

    // Repository errors
    RepositoryError {
        repository: String,
        operation: String,
        reason: String,
    },
    RepositoryNotFound {
        name: String,
        available: Vec<String>,
    },

    // File operation errors
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

    // Parser errors
    ParseError {
        content_type: String,
        line_number: Option<usize>,
        reason: String,
        context: Option<String>,
    },

    // Analysis errors
    AnalysisError {
        repository: String,
        stage: String,
        reason: String,
        recoverable: bool,
    },

    // Network/API errors
    NetworkError {
        operation: String,
        url: Option<String>,
        status_code: Option<u16>,
        reason: String,
    },

    // Validation errors
    ValidationError {
        field: String,
        value: String,
        constraint: String,
        suggestion: Option<String>,
    },

    // System errors
    SystemError {
        operation: String,
        reason: String,
    },

    // User input errors
    UserInputError {
        input: String,
        expected: String,
        suggestion: String,
    },

    // Multiple errors (for batch operations)
    MultipleErrors {
        errors: Vec<AilyzerError>,
        context: String,
    },
}

impl AilyzerError {
    pub fn config_error(message: &str, field: Option<&str>, suggestion: Option<&str>) -> Self {
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

    pub fn analysis_error(repository: &str, stage: &str, reason: &str, recoverable: bool) -> Self {
        Self::AnalysisError {
            repository: repository.to_string(),
            stage: stage.to_string(),
            reason: reason.to_string(),
            recoverable,
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
                format!("System error during {}: {}\n💡 This may require administrator intervention", operation, reason)
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

    pub fn with_context(self, context: &str) -> Self {
        match self {
            Self::MultipleErrors { mut errors, context: existing_context } => {
                Self::MultipleErrors {
                    errors,
                    context: format!("{} -> {}", existing_context, context),
                }
            }
            _ => Self::MultipleErrors {
                errors: vec![self],
                context: context.to_string(),
            }
        }
    }
}

impl fmt::Display for AilyzerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl StdError for AilyzerError {}

/// Error severity levels
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

/// Result type alias for ailyzer operations
pub type AilyzerResult<T> = Result<T, AilyzerError>;

/// Error handler for consistent error processing
pub struct ErrorHandler;

impl ErrorHandler {
    /// Handle error with appropriate logging and user feedback
    pub fn handle_error(error: &AilyzerError) {
        let severity = error.severity();

        // Log technical details
        log::error!("[{}] {}", severity.name(), error.technical_details());

        // Print user-friendly message
        eprintln!("{} {}", severity.emoji(), error.user_message());

        // Additional actions based on severity
        match severity {
            ErrorSeverity::Critical => {
                eprintln!("🚨 Critical error detected - application may need to exit");
            }
            ErrorSeverity::High => {
                eprintln!("⚠️ High severity error - operation failed");
            }
            ErrorSeverity::Medium => {
                eprintln!("⚠️ Error occurred - some functionality may be affected");
            }
            ErrorSeverity::Low => {
                eprintln!("ℹ️ Minor issue detected - operation can continue");
            }
        }

        // Recovery suggestions
        if error.is_recoverable() {
            eprintln!("🔄 This error is recoverable - you can retry the operation");
        }
    }

    /// Handle multiple errors with summary
    pub fn handle_multiple_errors(errors: &[AilyzerError], context: &str) {
        if errors.is_empty() {
            return;
        }

        eprintln!("❌ Multiple errors occurred during {}:", context);

        let mut by_severity: std::collections::BTreeMap<ErrorSeverity, Vec<&AilyzerError>> = std::collections::BTreeMap::new();

        for error in errors {
            by_severity.entry(error.severity()).or_insert_with(Vec::new).push(error);
        }

        // Print summary by severity (highest first)
        for (severity, severity_errors) in by_severity.iter().rev() {
            eprintln!("\n{} {} ({} errors):", severity.emoji(), severity.name(), severity_errors.len());
            for (i, error) in severity_errors.iter().enumerate() {
                eprintln!("  {}. {}", i + 1, error.user_message().replace('\n', "\n     "));
            }
        }

        // Overall recommendations
        let critical_count = by_severity.get(&ErrorSeverity::Critical).map_or(0, |v| v.len());
        let high_count = by_severity.get(&ErrorSeverity::High).map_or(0, |v| v.len());

        if critical_count > 0 {
            eprintln!("\n🚨 {} critical errors require immediate attention", critical_count);
        } else if high_count > 0 {
            eprintln!("\n⚠️ {} high severity errors should be addressed", high_count);
        }

        let recoverable_count = errors.iter().filter(|e| e.is_recoverable()).count();
        if recoverable_count > 0 {
            eprintln!("🔄 {} errors are recoverable and can be retried", recoverable_count);
        }
    }

    /// Convert standard errors to AilyzerError
    pub fn from_std_error(error: Box<dyn StdError>, operation: &str) -> AilyzerError {
        AilyzerError::SystemError {
            operation: operation.to_string(),
            reason: error.to_string(),
        }
    }

    /// Create error from string with context
    pub fn from_string(message: &str, operation: &str) -> AilyzerError {
        AilyzerError::SystemError {
            operation: operation.to_string(),
            reason: message.to_string(),
        }
    }
}

/// Macro for easy error creation
#[macro_export]
macro_rules! ailyzer_error {
    (config, $msg:expr) => {
        AilyzerError::config_error($msg, None, None)
    };
    (config, $msg:expr, $field:expr) => {
        AilyzerError::config_error($msg, Some($field), None)
    };
    (config, $msg:expr, $field:expr, $suggestion:expr) => {
        AilyzerError::config_error($msg, Some($field), Some($suggestion))
    };
    (repo, $repo:expr, $op:expr, $reason:expr) => {
        AilyzerError::repo_error($repo, $op, $reason)
    };
    (file, $path:expr, $op:expr, $reason:expr) => {
        AilyzerError::file_error($path, $op, $reason)
    };
    (parse, $type:expr, $reason:expr) => {
        AilyzerError::parse_error($type, None, $reason, None)
    };
    (parse, $type:expr, $line:expr, $reason:expr) => {
        AilyzerError::parse_error($type, Some($line), $reason, None)
    };
    (analysis, $repo:expr, $stage:expr, $reason:expr) => {
        AilyzerError::analysis_error($repo, $stage, $reason, true)
    };
    (analysis, $repo:expr, $stage:expr, $reason:expr, $recoverable:expr) => {
        AilyzerError::analysis_error($repo, $stage, $reason, $recoverable)
    };
    (validation, $field:expr, $value:expr, $constraint:expr) => {
        AilyzerError::validation_error($field, $value, $constraint, None)
    };
    (validation, $field:expr, $value:expr, $constraint:expr, $suggestion:expr) => {
        AilyzerError::validation_error($field, $value, $constraint, Some($suggestion))
    };
}

/// Extension trait for Result to add context
pub trait ResultExt<T> {
    fn with_context(self, context: &str) -> AilyzerResult<T>;
    fn with_operation(self, operation: &str) -> AilyzerResult<T>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<AilyzerError>,
{
    fn with_context(self, context: &str) -> AilyzerResult<T> {
        self.map_err(|e| e.into().with_context(context))
    }

    fn with_operation(self, operation: &str) -> AilyzerResult<T> {
        self.map_err(|e| {
            let ailyzer_error: AilyzerError = e.into();
            match ailyzer_error {
                AilyzerError::SystemError { reason, .. } => {
                    AilyzerError::SystemError {
                        operation: operation.to_string(),
                        reason,
                    }
                }
                other => other.with_context(operation),
            }
        })
    }
}

/// Convert from standard library errors
impl From<std::io::Error> for AilyzerError {
    fn from(error: std::io::Error) -> Self {
        AilyzerError::SystemError {
            operation: "I/O operation".to_string(),
            reason: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for AilyzerError {
    fn from(error: serde_json::Error) -> Self {
        AilyzerError::ParseError {
            content_type: "JSON".to_string(),
            line_number: Some(error.line()),
            reason: error.to_string(),
            context: None,
        }
    }
}

impl From<toml::de::Error> for AilyzerError {
    fn from(error: toml::de::Error) -> Self {
        AilyzerError::ParseError {
            content_type: "TOML".to_string(),
            line_number: None,
            reason: error.message().to_string(),
            context: None,
        }
    }
}

impl From<reqwest::Error> for AilyzerError {
    fn from(error: reqwest::Error) -> Self {
        AilyzerError::NetworkError {
            operation: "HTTP request".to_string(),
            url: error.url().map(|u| u.to_string()),
            status_code: error.status().map(|s| s.as_u16()),
            reason: error.to_string(),
        }
    }
}


use std::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    MissingField { field: String, line: usize },

    InvalidFormat { line: usize, expected: String, found: String },

    InvalidNumber { value: String, field: String, line: usize },

    UnexpectedEof { context: String },

    UnknownChangeType { change_type: String, line: usize },

    UnknownActionType { action_type: String, line: usize },

    ParseError { message: String, line: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingField { field, line } => {
                write!(f, "Missing required field '{}' at line {}", field, line)
            }
            ParseError::InvalidFormat { line, expected, found } => {
                write!(f, "Invalid format at line {}: expected '{}', found '{}'", line, expected, found)
            }
            ParseError::InvalidNumber { value, field, line } => {
                write!(f, "Invalid number '{}' for field '{}' at line {}", value, field, line)
            }
            ParseError::UnexpectedEof { context } => {
                write!(f, "Unexpected end of input while parsing {}", context)
            }
            ParseError::UnknownChangeType { change_type, line } => {
                write!(f, "Unknown change type '{}' at line {}", change_type, line)
            }
            ParseError::UnknownActionType { action_type, line } => {
                write!(f, "Unknown action type '{}' at line {}", action_type, line)
            }
            ParseError::ParseError { message, line } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}
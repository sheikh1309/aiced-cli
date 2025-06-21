#[derive(Debug, Clone)]
pub enum ApplicationMode {
    Individual,  // Review each change individually
    Priority,    // Apply by priority order
    Category,    // Apply by category
    Severity,    // Apply by severity level
    All,         // Apply all changes at once
    Skip,        // Skip this repository
}
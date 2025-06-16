#[derive(Debug, Clone, Copy)]
pub enum DiffDisplayMode {
    Full,           // Show all changes at once
    Paged,          // Use system pager (less/more)
    Summary,        // Show summary first, then ask to see details
    Interactive,    // Show changes one by one with prompts
    Compact,        // Condensed view with fewer context lines
}
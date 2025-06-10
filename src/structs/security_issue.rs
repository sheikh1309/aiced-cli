pub struct SecurityIssue {
    pub file_path: String,
    pub line_number: usize,
    pub issue: String,
    pub severity: String,
    pub recommendation: String,
}
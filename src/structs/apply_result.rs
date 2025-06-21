#[derive(Debug, Default)]
pub struct ApplyResult {
    pub security_applied: usize,
    pub bugs_applied: usize,
    pub performance_applied: usize,
    pub architecture_applied: usize,
    pub clean_code_applied: usize,
    pub duplicate_code_applied: usize,
    pub total_applied: usize,
    pub failed: usize,
}
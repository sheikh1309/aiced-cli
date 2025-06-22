use crate::enums::file_change::FileChange;
use crate::structs::technology_stack::TechnologyStack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub technology_stack: Option<TechnologyStack>,
    pub analysis_summary: String,
    pub changes: Vec<FileChange>,
}

impl AnalysisResponse {
    pub fn get_changes_by_severity(&self, severity: &str) -> Vec<&FileChange> {
        self.changes.iter()
            .filter(|change| change.get_severity() == severity)
            .collect()
    }

    pub fn get_changes_by_category(&self, category: &str) -> Vec<&FileChange> {
        self.changes.iter()
            .filter(|change| change.get_category().as_deref() == Some(category))
            .collect()
    }

    pub fn get_critical_changes(&self) -> Vec<&FileChange> {
        self.get_changes_by_severity("critical")
    }

    pub fn get_high_priority_changes(&self) -> Vec<&FileChange> {
        let mut changes = self.get_changes_by_severity("critical");
        changes.extend(self.get_changes_by_severity("high"));
        changes
    }

    pub fn has_security_issues(&self) -> bool {
        !self.get_changes_by_category("SECURITY").is_empty()
    }

    pub fn has_architecture_issues(&self) -> bool {
        !self.get_changes_by_category("ARCHITECTURE").is_empty()
    }

    pub fn get_summary_stats(&self) -> AnalysisStats {
        let mut stats = AnalysisStats::default();

        for change in &self.changes {
            match change.get_severity() {
                "critical" => stats.critical_count += 1,
                "high" => stats.high_count += 1,
                "medium" => stats.medium_count += 1,
                "low" => stats.low_count += 1,
                _ => stats.unknown_count += 1,
            }

            if let Some(category) = change.get_category() {
                match category {
                    "BUGS" => stats.bugs_count += 1,
                    "SECURITY" => stats.security_count += 1,
                    "PERFORMANCE" => stats.performance_count += 1,
                    "CLEAN_CODE" => stats.clean_code_count += 1,
                    "ARCHITECTURE" => stats.architecture_count += 1,
                    "DUPLICATE_CODE" => stats.duplicate_code_count += 1,
                    _ => stats.other_count += 1,
                }
            }
        }

        stats
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisStats {
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub unknown_count: usize,
    pub bugs_count: usize,
    pub security_count: usize,
    pub performance_count: usize,
    pub clean_code_count: usize,
    pub architecture_count: usize,
    pub duplicate_code_count: usize,
    pub other_count: usize,
}

impl AnalysisStats {
    pub fn high_priority_issues(&self) -> usize {
        self.critical_count + self.high_count
    }
}
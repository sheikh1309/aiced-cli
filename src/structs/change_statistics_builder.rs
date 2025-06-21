use crate::structs::change_statistics::ChangeStatistics;

pub struct ChangeStatisticsBuilder {
    stats: ChangeStatistics,
}

impl ChangeStatisticsBuilder {
    pub fn new() -> Self {
        Self {
            stats: ChangeStatistics::default(),
        }
    }

    pub fn total_count(mut self, count: usize) -> Self {
        self.stats.total_count = count;
        self
    }

    pub fn add_severity_counts(mut self, critical: usize, high: usize, medium: usize, low: usize) -> Self {
        self.stats.critical_count = critical;
        self.stats.high_count = high;
        self.stats.medium_count = medium;
        self.stats.low_count = low;
        self
    }

    pub fn add_category_counts(mut self, security: usize, bugs: usize, performance: usize, clean_code: usize, architecture: usize, duplicate_code: usize) -> Self {
        self.stats.security_count = security;
        self.stats.bugs_count = bugs;
        self.stats.performance_count = performance;
        self.stats.clean_code_count = clean_code;
        self.stats.architecture_count = architecture;
        self.stats.duplicate_code_count = duplicate_code;
        self
    }

    pub fn add_file_impact(mut self, file_path: String, change_count: usize) -> Self {
        self.stats.files_affected.insert(file_path.clone(), change_count);

        // Update largest file impact
        if let Some((_, current_max)) = &self.stats.largest_file_impact {
            if change_count > *current_max {
                self.stats.largest_file_impact = Some((file_path, change_count));
            }
        } else {
            self.stats.largest_file_impact = Some((file_path, change_count));
        }

        self
    }

    pub fn build(self) -> ChangeStatistics {
        self.stats
    }
}

impl Default for ChangeStatisticsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
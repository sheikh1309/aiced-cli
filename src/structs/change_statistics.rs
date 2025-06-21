use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub(crate) use crate::enums::application_strategy::ApplicationStrategy;
use crate::enums::priority_recommendation::PriorityRecommendation;
use crate::structs::category_stats::CategoryStats;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeStatistics {
    // Total counts
    pub total_count: usize,
    pub total_line_changes: usize,
    pub multi_line_changes: usize,

    // By change type
    pub modify_count: usize,
    pub create_count: usize,
    pub delete_count: usize,

    // By severity
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub unknown_severity_count: usize,

    // By category
    pub bugs_count: usize,
    pub security_count: usize,
    pub performance_count: usize,
    pub clean_code_count: usize,
    pub architecture_count: usize,
    pub duplicate_code_count: usize,
    pub other_category_count: usize,

    // File impact analysis
    pub files_affected: HashMap<String, usize>, // file_path -> change_count
    pub largest_file_impact: Option<(String, usize)>, // (file_path, change_count)

    // Risk assessment
    pub high_risk_changes: usize, // critical + high severity
    pub security_risk_score: u32,
    pub complexity_score: u32,
}

impl ChangeStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_high_priority_count(&self) -> usize {
        self.critical_count + self.high_count
    }

    pub fn get_security_and_bugs_count(&self) -> usize {
        self.security_count + self.bugs_count
    }

    pub fn get_code_quality_count(&self) -> usize {
        self.clean_code_count + self.architecture_count + self.duplicate_code_count
    }

    pub fn calculate_risk_score(&self) -> u32 {
        let mut score = 0u32;

        // Severity-based scoring
        score += self.critical_count as u32 * 25;  // Critical issues: 25 points each
        score += self.high_count as u32 * 15;      // High issues: 15 points each
        score += self.medium_count as u32 * 8;     // Medium issues: 8 points each
        score += self.low_count as u32 * 3;        // Low issues: 3 points each

        // Category-based bonus
        score += self.security_count as u32 * 10;  // Security bonus: 10 points each
        score += self.bugs_count as u32 * 8;       // Bug bonus: 8 points each

        // Multi-line change complexity
        score += self.multi_line_changes as u32 * 2;

        // Cap at 100
        std::cmp::min(score, 100)
    }

    pub fn get_priority_recommendation(&self) -> PriorityRecommendation {
        let risk_score = self.calculate_risk_score();
        let security_and_bugs = self.get_security_and_bugs_count();
        let high_priority = self.get_high_priority_count();

        if risk_score >= 80 || security_and_bugs >= 5 {
            PriorityRecommendation::Immediate
        } else if risk_score >= 60 || high_priority >= 3 {
            PriorityRecommendation::High
        } else if risk_score >= 40 || self.total_count >= 10 {
            PriorityRecommendation::Medium
        } else {
            PriorityRecommendation::Low
        }
    }

    pub fn get_application_strategy(&self) -> ApplicationStrategy {
        let security_and_bugs = self.get_security_and_bugs_count();
        let high_priority = self.get_high_priority_count();

        if security_and_bugs > 0 && high_priority > 0 {
            ApplicationStrategy::PriorityBased
        } else if self.security_count > 0 {
            ApplicationStrategy::SecurityFirst
        } else if self.total_count > 20 {
            ApplicationStrategy::CategoryBased
        } else {
            ApplicationStrategy::AllAtOnce
        }
    }

    pub fn print_summary(&self) {
        println!("\n📊 Change Analysis Summary");
        println!("═══════════════════════════════════════");

        // Overview
        println!("📈 Overview:");
        println!("   Total Changes: {}", self.total_count);
        println!("   Total Line Changes: {}", self.total_line_changes);
        println!("   Multi-line Changes: {}", self.multi_line_changes);
        println!("   Files Affected: {}", self.files_affected.len());

        // Risk Assessment
        let risk_score = self.calculate_risk_score();
        let risk_level = match risk_score {
            80..=100 => "🔴 CRITICAL",
            60..=79 => "🟠 HIGH",
            40..=59 => "🟡 MEDIUM",
            20..=39 => "🟢 LOW",
            _ => "⚪ MINIMAL",
        };
        println!("\n🎯 Risk Assessment:");
        println!("   Risk Score: {}/100 ({})", risk_score, risk_level);
        println!("   Priority: {:?}", self.get_priority_recommendation());
        println!("   Strategy: {:?}", self.get_application_strategy());

        // By Type
        println!("\n📝 By Change Type:");
        println!("   Modify Files: {}", self.modify_count);
        println!("   Create Files: {}", self.create_count);
        println!("   Delete Files: {}", self.delete_count);

        // By Severity
        println!("\n⚡ By Severity:");
        println!("   Critical: {} 🔴", self.critical_count);
        println!("   High: {} 🟠", self.high_count);
        println!("   Medium: {} 🟡", self.medium_count);
        println!("   Low: {} 🟢", self.low_count);
        if self.unknown_severity_count > 0 {
            println!("   Unknown: {} ⚪", self.unknown_severity_count);
        }

        // By Category
        println!("\n🏷️ By Category:");
        if self.security_count > 0 {
            println!("   Security: {} 🔒", self.security_count);
        }
        if self.bugs_count > 0 {
            println!("   Bugs: {} 🐛", self.bugs_count);
        }
        if self.performance_count > 0 {
            println!("   Performance: {} 🚀", self.performance_count);
        }
        if self.architecture_count > 0 {
            println!("   Architecture: {} 🏗️", self.architecture_count);
        }
        if self.clean_code_count > 0 {
            println!("   Clean Code: {} ✨", self.clean_code_count);
        }
        if self.duplicate_code_count > 0 {
            println!("   Duplicate Code: {} 🔄", self.duplicate_code_count);
        }
        if self.other_category_count > 0 {
            println!("   Other: {} 📦", self.other_category_count);
        }

        // Key Insights
        println!("\n💡 Key Insights:");
        let high_priority = self.get_high_priority_count();
        let security_and_bugs = self.get_security_and_bugs_count();

        if security_and_bugs > 0 {
            println!("   ⚠️ {} security/bug issues need immediate attention", security_and_bugs);
        }

        if high_priority > 0 {
            println!("   🔥 {} high-priority changes should be applied first", high_priority);
        }

        if self.multi_line_changes > 0 {
            let percentage = (self.multi_line_changes * 100) / self.total_line_changes.max(1);
            println!("   📏 {}% of line changes affect multiple lines", percentage);
        }

        if let Some((file, count)) = &self.largest_file_impact {
            println!("   📁 Most impacted file: {} ({} changes)", file, count);
        }

        // Recommendations
        println!("\n🎯 Recommendations:");
        match self.get_application_strategy() {
            ApplicationStrategy::PriorityBased => {
                println!("   1. Apply security and bug fixes first");
                println!("   2. Then apply high-severity changes");
                println!("   3. Finally apply code quality improvements");
            }
            ApplicationStrategy::SecurityFirst => {
                println!("   1. Focus on security issues immediately");
                println!("   2. Review and apply other changes as time permits");
            }
            ApplicationStrategy::CategoryBased => {
                println!("   1. Group changes by category for easier review");
                println!("   2. Apply in batches to manage complexity");
            }
            ApplicationStrategy::AllAtOnce => {
                println!("   1. Changes are manageable - can apply all at once");
                println!("   2. Review carefully before applying");
            }
        }

        println!("═══════════════════════════════════════\n");
    }

    pub fn print_compact_summary(&self) {
        let risk_score = self.calculate_risk_score();
        let risk_emoji = match risk_score {
            80..=100 => "🔴",
            60..=79 => "🟠",
            40..=59 => "🟡",
            _ => "🟢",
        };

        println!("📊 {} changes | {} Risk: {}/100 | 🔒{} 🐛{} ⚡{} | Strategy: {:?}",
                 self.total_count,
                 risk_emoji,
                 risk_score,
                 self.security_count,
                 self.bugs_count,
                 self.get_high_priority_count(),
                 self.get_application_strategy()
        );
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn get_category_stats(&self, category: &str) -> CategoryStats {
        let count = match category {
            "SECURITY" => self.security_count,
            "BUGS" => self.bugs_count,
            "PERFORMANCE" => self.performance_count,
            "CLEAN_CODE" => self.clean_code_count,
            "ARCHITECTURE" => self.architecture_count,
            "DUPLICATE_CODE" => self.duplicate_code_count,
            _ => self.other_category_count,
        };

        let percentage = if self.total_count > 0 {
            (count * 100) / self.total_count
        } else {
            0
        };

        CategoryStats {
            category: category.to_string(),
            count,
            percentage,
            is_high_impact: count > 0 && matches!(category, "SECURITY" | "BUGS"),
        }
    }
}

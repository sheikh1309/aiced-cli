use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub count: usize,
    pub percentage: usize,
    pub is_high_impact: bool,
}

impl CategoryStats {
    pub fn print(&self) {
        let impact_indicator = if self.is_high_impact { "🔥" } else { "📝" };
        println!("   {} {}: {} ({}%)", impact_indicator, self.category, self.count, self.percentage);
    }
}
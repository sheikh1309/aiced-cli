use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationStrategy {
    PriorityBased,  // Apply by priority order (security -> bugs -> severity)
    SecurityFirst,  // Focus on security issues first
    CategoryBased,  // Group by category for batch processing
    AllAtOnce,      // Small enough to apply all changes together
}

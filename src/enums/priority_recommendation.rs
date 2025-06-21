use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityRecommendation {
    Immediate,  // Apply ASAP - critical security/bugs
    High,       // Apply soon - high severity issues
    Medium,     // Apply when convenient - moderate issues
    Low,        // Apply when time permits - minor improvements
}
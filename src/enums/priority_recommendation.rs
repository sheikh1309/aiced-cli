use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityRecommendation {
    Immediate,  
    High,       
    Medium,     
    Low,        
}
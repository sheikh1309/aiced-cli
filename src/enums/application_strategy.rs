use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApplicationStrategy {
    PriorityBased,  
    SecurityFirst,  
    CategoryBased,  
    AllAtOnce,      
}

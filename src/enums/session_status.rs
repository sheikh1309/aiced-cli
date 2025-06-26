use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
}
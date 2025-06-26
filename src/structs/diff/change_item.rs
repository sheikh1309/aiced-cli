use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeItem {
    pub id: String,
    pub change_type: String,
    pub line_number: usize,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub applied: bool,
    pub reason: String,
}
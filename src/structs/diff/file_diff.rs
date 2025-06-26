use serde::{Deserialize, Serialize};
use crate::structs::diff::change_item::ChangeItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub changes: Vec<ChangeItem>,
    pub original_content: String,
    pub preview_content: String,
    pub file_type: String,
}
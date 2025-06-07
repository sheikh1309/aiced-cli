use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "action")]
pub enum LineChange {
    #[serde(rename = "replace")]
    Replace {
        line_number: usize,
        old_content: String,
        new_content: String,
    },
    #[serde(rename = "insert_after")]
    InsertAfter {
        line_number: usize,
        new_content: String,
    },
    #[serde(rename = "insert_before")]
    InsertBefore {
        line_number: usize,
        new_content: String,
    },
    #[serde(rename = "delete")]
    Delete {
        line_number: usize,
    },
    #[serde(rename = "replace_range")]
    ReplaceRange {
        start_line: usize,
        end_line: usize,
        old_content: Vec<String>,
        new_content: Vec<String>,
    },
}
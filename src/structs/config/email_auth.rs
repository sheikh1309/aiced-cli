use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailAuth {
    pub username: String,
    pub password_env: String,
}
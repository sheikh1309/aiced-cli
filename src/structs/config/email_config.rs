use serde::{Deserialize, Serialize};
use crate::structs::config::email_auth::EmailAuth;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub from: String,
    pub to: Vec<String>,
    pub auth: EmailAuth,
}
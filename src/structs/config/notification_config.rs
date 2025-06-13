use serde::{Deserialize, Serialize};
use crate::structs::config::email_config::EmailConfig;
use crate::structs::config::slack_config::SlackConfig;
use crate::structs::config::webhook_config::WebhookConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[derive(Default)]
pub struct NotificationConfig {
    #[serde(default)]
    pub email: Option<EmailConfig>,

    #[serde(default)]
    pub slack: Option<SlackConfig>,

    #[serde(default)]
    pub webhook: Option<WebhookConfig>,

    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub summary_report: bool,
}
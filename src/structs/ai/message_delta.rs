use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
}

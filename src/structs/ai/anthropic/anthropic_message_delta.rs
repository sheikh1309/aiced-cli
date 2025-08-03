use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicMessageDelta {
    pub stop_reason: Option<String>,
}

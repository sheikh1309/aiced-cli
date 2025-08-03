use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AnthropicContentDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    pub text: Option<String>,
}
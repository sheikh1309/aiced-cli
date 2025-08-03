use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

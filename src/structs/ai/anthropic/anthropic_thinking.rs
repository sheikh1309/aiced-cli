use serde::Serialize;

#[derive(Serialize)]
pub struct AnthropicThinking {
    pub r#type: String,
    pub budget_tokens: u32,
}
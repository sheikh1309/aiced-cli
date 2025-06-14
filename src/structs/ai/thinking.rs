use serde::Serialize;

#[derive(Serialize)]
pub struct Thinking {
    pub r#type: String,
    pub budget_tokens: u32,
}
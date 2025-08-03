use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
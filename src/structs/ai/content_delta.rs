use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ContentDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    pub text: Option<String>,
}
use serde::Deserialize;
use crate::enums::stream_event_data::StreamEventData;

#[derive(Deserialize, Debug, Clone)]
pub struct StreamResponse {
    #[serde(flatten)]
    pub data: StreamEventData,
}
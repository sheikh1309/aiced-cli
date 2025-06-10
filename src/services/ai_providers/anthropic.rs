use crate::structs::message::Message;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use crate::structs::stream_item::StreamItem;
use crate::traits::ai_provider::AiProvider;

#[derive(Serialize)]
struct Thinking {
    r#type: String,
    budget_tokens: u32,
}

#[derive(Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    temperature: Option<f32>,
    messages: Vec<AnthropicMessage>,
    stream: bool,
    thinking: Thinking,
}

#[derive(Serialize, Debug)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct StreamResponse {
    #[serde(flatten)]
    data: StreamEventData,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum StreamEventData {
    ContentBlockDelta {
        delta: ContentDelta,
    },
    MessageDelta {
        delta: MessageDelta,
    },
    Error {
        error: ApiError,
    },
}

#[derive(Deserialize, Debug)]
struct ContentDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct MessageDelta {
    stop_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

#[derive(Debug, Clone)]
pub enum AnthropicError {
    ApiError(String),
    NetworkError(String),
    SerializationError(String),
    AuthenticationError(String),
}

impl fmt::Display for AnthropicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnthropicError::ApiError(msg) => write!(f, "Anthropic API Error: {}", msg),
            AnthropicError::NetworkError(msg) => write!(f, "Network Error: {}", msg),
            AnthropicError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            AnthropicError::AuthenticationError(msg) => write!(f, "Authentication Error: {}", msg),
        }
    }
}

impl Error for AnthropicError {}

#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: Client,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }

    fn get_anthropic_messages(&self, messages: &[Message]) -> Vec<AnthropicMessage> {
        messages
            .iter()
            .map(|msg| AnthropicMessage {
                role: match msg.role.as_str() {
                    "system" => "user".to_string(), // Anthropic handles system messages differently
                    role => role.to_string(),
                },
                content: if msg.role == "system" {
                    format!("<system>{}</system>\n\nPlease follow the instructions in the system message above.", msg.content)
                } else {
                    msg.content.clone()
                },
            })
            .collect()
    }

    fn get_request(&self, messages: Vec<AnthropicMessage>, stream: bool) -> MessageRequest {
        MessageRequest {
            model: self.model.clone(),
            max_tokens: 64000,
            temperature: Some(1.0), // Set to 0 for more consistent output
            messages,
            stream,
            thinking: Thinking {
                r#type: "enabled".to_string(), // Disable thinking for now
                budget_tokens: 63999,
            },
        }
    }

    async fn make_request(&self, url: String, request_body: MessageRequest) -> Result<reqwest::Response, AnthropicError> {
        println!("🔍 Making request to: {}", url);
        println!("📦 Request model: {}", request_body.model);

        self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream") // Important for SSE
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AnthropicError::NetworkError(e.to_string()))
    }

    fn parse_sse_line(line: &str) -> Option<Result<StreamItem, AnthropicError>> {
        if line.is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];

        if data.trim() == "[DONE]" {
            return None;
        }

        match serde_json::from_str::<StreamResponse>(data) {
            Ok(response) => {
                let item = match response.data {
                    StreamEventData::ContentBlockDelta { delta, .. } => {
                        if delta.delta_type == "text_delta" {
                            StreamItem::new(delta.text.unwrap_or_default())
                        } else {
                            StreamItem::new(String::new())
                        }
                    }
                    StreamEventData::MessageDelta { delta, .. } => {
                        if let Some(stop_reason) = delta.stop_reason {
                            StreamItem::complete(String::new(), Some(stop_reason))
                        } else {
                            StreamItem::new(String::new())
                        }
                    }
                    StreamEventData::Error { error } => {
                        return Some(Err(AnthropicError::ApiError(
                            format!("{}: {}", error.error_type, error.message)
                        )));
                    }
                };
                Some(Ok(item))
            }
            Err(e) => {
                Some(Err(AnthropicError::SerializationError(format!("Failed to parse event: {}", e))))
            }
        }
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    type Error = AnthropicError;

    async fn create_stream_request(&self, messages: &[Message]) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AnthropicError>> + Send>>, AnthropicError> {
        let url = format!("{}/messages", self.base_url);
        let anthropic_messages = self.get_anthropic_messages(messages);
        let request_body = self.get_request(anthropic_messages, true);

        let response = self.make_request(url, request_body).await?;

        println!("📡 Response status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            eprintln!("❌ API Error Response: {}", error_text);

            return Err(match status.as_u16() {
                401 => AnthropicError::AuthenticationError(error_text),
                _ => AnthropicError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        // Create a stream that properly handles SSE format
        let stream = response
            .bytes_stream()
            .map(move |chunk_result| {
                match chunk_result {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);

                        // Split by newlines and process each line
                        let items: Vec<Result<StreamItem, AnthropicError>> = text
                            .lines()
                            .filter_map(|line| Self::parse_sse_line(line))
                            .collect();

                        futures::stream::iter(items)
                    }
                    Err(e) => {
                        let error = AnthropicError::NetworkError(format!("Stream error: {}", e));
                        futures::stream::iter(vec![Err(error)])
                    }
                }
            })
            .flatten();

        Ok(Box::pin(stream))
    }
}
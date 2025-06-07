use crate::structs::message::Message;
use crate::structs::stream_item::StreamItem;
use crate::helpers::continuation::run_continuation_task;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use crate::traits::ai_provider::AiProvider;
use futures::TryStreamExt;
use futures::FutureExt;

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

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct StreamResponse {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<StreamDelta>,
    index: Option<u32>,
}

#[derive(Deserialize)]
struct StreamMessage {
    id: Option<String>,
    #[serde(rename = "type")]
    message_type: Option<String>,
    role: Option<String>,
    content: Option<Vec<ContentBlock>>,
    model: Option<String>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct StreamDelta {
    #[serde(rename = "type")]
    delta_type: Option<String>,
    text: Option<String>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

#[derive(Debug)]
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
            model: "claude-opus-4-20250514".to_string(),
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
                    format!("System: {}", msg.content)
                } else {
                    msg.content.clone()
                },
            })
            .collect()
    }

    fn get_request(&self, messages: Vec<AnthropicMessage>, stream: bool) -> MessageRequest {
        MessageRequest {
            model: self.model.clone(),
            max_tokens: 32000,
            temperature: Some(1.0),
            messages,
            stream,
            thinking: Thinking {
                r#type: "enabled".to_string(),
                budget_tokens: 31999,
            },
        }
    }

    async fn make_request(&self, url: String, request_body: MessageRequest) -> Result<reqwest::Response, AnthropicError> {
        self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AnthropicError::NetworkError(e.to_string()))
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    type Error = AnthropicError;

    async fn generate_completion_stream(&self, messages: &[Message]) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, Self::Error>> + Send>>, Self::Error> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let messages = messages.to_vec();
        let provider = self.clone();

        tokio::spawn(async move {
            run_continuation_task(provider, messages, tx).await;
        });

        Ok(Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx)))
    }

    async fn create_stream_request(&self, messages: &[Message]) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AnthropicError>> + Send>>, AnthropicError> {
        let url = format!("{}/messages", self.base_url);
        let anthropic_messages = self.get_anthropic_messages(messages);
        let request_body = self.get_request(anthropic_messages, true);
        let response = self.make_request(url, request_body).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                401 => AnthropicError::AuthenticationError(error_text),
                _ => AnthropicError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let stream = response
            .bytes_stream()
            .map_err(|e| AnthropicError::NetworkError(e.to_string()))
            .fold(String::new(), |mut buffer, chunk_result| async move {
                match chunk_result {
                    Ok(chunk) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                        buffer
                    }
                    Err(_) => buffer,
                }
            })
            .map(|complete_data| {
                let items: Vec<Result<StreamItem, AnthropicError>> = complete_data
                    .lines()
                    .filter(|line| line.starts_with("data: "))
                    .map(|line| &line[6..])
                    .filter(|line| *line != "[DONE]")
                    .map(|line| {
                        serde_json::from_str::<StreamResponse>(line)
                            .map_err(|e| AnthropicError::SerializationError(e.to_string()))
                            .and_then(|response| {
                                match response.event_type.as_str() {
                                    "content_block_delta" => {
                                        if let Some(delta) = response.delta {
                                            let content = delta.text.unwrap_or_default();
                                            let finish_reason = delta.stop_reason;
                                            if finish_reason.is_some() {
                                                Ok(StreamItem::complete(content, finish_reason))
                                            } else {
                                                Ok(StreamItem::new(content))
                                            }
                                        } else {
                                            Ok(StreamItem::new("".to_string()))
                                        }
                                    },
                                    "message_delta" => {
                                        if let Some(delta) = response.delta {
                                            let finish_reason = delta.stop_reason;
                                            if finish_reason.is_some() {
                                                Ok(StreamItem::complete("".to_string(), finish_reason))
                                            } else {
                                                Ok(StreamItem::new("".to_string()))
                                            }
                                        } else {
                                            Ok(StreamItem::new("".to_string()))
                                        }
                                    },
                                    "message_start" | "content_block_start" | "content_block_stop" | "message_stop" => {
                                        Ok(StreamItem::new("".to_string()))
                                    },
                                    _ => Ok(StreamItem::new("".to_string()))
                                }
                            })
                    })
                    .collect();

                futures::stream::iter(items)
            })
            .flatten_stream();

        Ok(Box::pin(stream))
    }
}
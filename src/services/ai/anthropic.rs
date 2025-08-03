use std::option::Option;
use reqwest::Client;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use async_trait::async_trait;
use futures::future;
use crate::enums::ai_provider_error::AiProviderError;
use crate::enums::stream_event_data::StreamEventData;
use crate::structs::ai::anthropic::anthropic_message::AnthropicMessage;
use crate::structs::ai::anthropic::anthropic_message_request::AnthropicMessageRequest;
use crate::structs::ai::anthropic::anthropic_thinking::AnthropicThinking;
use crate::structs::stream_item::StreamItem;
use crate::traits::ai_provider::AiProvider;

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

    fn get_anthropic_messages(&self, user_prompts: Vec<String>) -> Vec<AnthropicMessage> {
        user_prompts
            .iter()
            .map(|msg| AnthropicMessage {
                role: String::from("user"),
                content: msg.clone(),
            })
            .collect()
    }

    fn get_request(&self, system_prompt: String, messages: Vec<AnthropicMessage>, stream: bool) -> AnthropicMessageRequest {
        AnthropicMessageRequest {
            model: self.model.clone(),
            max_tokens: 64000,
            temperature: Some(1.0),
            system: system_prompt,
            messages,
            stream,
            thinking: AnthropicThinking {
                r#type: "enabled".to_string(),
                budget_tokens: 63999,
            },
        }
    }

    async fn make_request(&self, url: String, request_body: AnthropicMessageRequest) -> Result<reqwest::Response, AiProviderError> {
        log::info!("📦 Request model: {}", request_body.model);

        self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream") // Important for SSE
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AiProviderError::NetworkError(e.to_string()))
    }

    fn parse_sse_line(line: &str) -> Option<Result<StreamItem, AiProviderError>> {
        if line.trim().is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];

        if data.trim() == "[DONE]" {
            return None;
        }

        if data.contains("\"type\":\"message_stop\"") {
            return None;
        }

        match serde_json::from_str::<StreamEventData>(data) {
            Ok(event_data) => {
                let item = match event_data {
                    StreamEventData::MessageStart { message } => {
                        StreamItem::with_tokens(
                            String::new(),
                            Some(message.usage.input_tokens),
                            Some(message.usage.output_tokens),
                        )
                    },
                    StreamEventData::ContentBlockDelta { delta, .. } => {
                        if delta.delta_type == "text_delta" {
                            StreamItem::new(delta.text.unwrap_or_default())
                        } else {
                            StreamItem::new(String::new())
                        }
                    }
                    StreamEventData::MessageDelta { delta, usage } => {
                        if let Some(stop_reason) = delta.stop_reason {
                            StreamItem::complete(
                                String::new(),
                                Some(stop_reason.to_string()),
                                usage.output_tokens
                            )
                        } else {
                            StreamItem::new(String::new())
                        }
                    }
                    StreamEventData::ContentBlockStart { .. } => {
                        StreamItem {
                            content: String::new(),
                            input_tokens: None,
                            output_tokens: None,
                            is_complete: false,
                            stop_reason: None,
                        }
                    },
                    StreamEventData::ContentBlockStop { .. } => {
                        StreamItem {
                            content: String::new(),
                            input_tokens: None,
                            output_tokens: None,
                            is_complete: false,
                            stop_reason: None,
                        }
                    },
                    StreamEventData::MessageStop => {
                        StreamItem {
                            content: String::new(),
                            input_tokens: None,
                            output_tokens: None,
                            is_complete: true,
                            stop_reason: None,
                        }
                    },
                    StreamEventData::Ping => {
                        StreamItem {
                            content: String::new(),
                            input_tokens: None,
                            output_tokens: None,
                            is_complete: false,
                            stop_reason: None,
                        }
                    },
                    StreamEventData::Error { error } => {
                        return Some(Err(AiProviderError::ApiError(format!("{}: {}", error.error_type, error.message))));
                    }
                };
                Some(Ok(item))
            }
            Err(e) => Some(Err(AiProviderError::SerializationError(format!("Failed to parse event: {}", e))))
        }
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {

    async fn stream_chat(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AiProviderError>> + Send>>, AiProviderError> {
        let url = format!("{}/messages", self.base_url);
        let anthropic_messages = self.get_anthropic_messages(user_prompts);
        let request_body = self.get_request(system_prompt, anthropic_messages, true);

        let response = self.make_request(url, request_body).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            log::error!("❌ API Error Response: {}", error_text);

            return Err(match status.as_u16() {
                401 => AiProviderError::AuthenticationError(error_text),
                _ => AiProviderError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        // Use scan for stateful stream processing
        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, chunk_result| {
                future::ready(match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);

                        let mut items = Vec::new();

                        // Process buffer line by line
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer.drain(..=newline_pos);

                            if let Some(result) = Self::parse_sse_line(&line) {
                                items.push(result);
                            }
                        }

                        Some(futures::stream::iter(items))
                    }
                    Err(e) => {
                        let error = AiProviderError::NetworkError(format!("Stream error: {}", e));
                        Some(futures::stream::iter(vec![Err(error)]))
                    }
                })
            })
            .flatten();

        Ok(Box::pin(stream))
    }
}
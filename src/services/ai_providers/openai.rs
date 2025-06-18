use std::option::Option;
use reqwest::Client;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use futures::future;

use crate::enums::ai_provider_error::AiProviderError;
use crate::services::rate_limiter::ApiRateLimiter;
use crate::structs::ai::openai::openai_message::OpenAIMessage;
use crate::structs::ai::openai::openai_request::OpenAIRequest;
use crate::structs::stream_item::StreamItem;

#[derive(Clone)]
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: Client,
    model: String,
    rate_limiter: Arc<ApiRateLimiter>,
}


impl OpenAIProvider {
    pub fn new(api_key: String, rate_limiter: Arc<ApiRateLimiter>) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
            model: "gpt-4o-mini".to_string(),
            rate_limiter,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    fn get_openai_messages(&self, system_prompt: String, user_prompts: Vec<String>) -> Vec<OpenAIMessage> {
        let mut messages = Vec::new();

        if !system_prompt.is_empty() {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
        }

        for prompt in user_prompts {
            messages.push(OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            });
        }

        messages
    }

    fn get_request(&self, system_prompt: String, user_prompts: Vec<String>, stream: bool) -> OpenAIRequest {
        let messages = self.get_openai_messages(system_prompt, user_prompts);

        OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(4096),
            temperature: Some(1.0),
            stream,
            top_p: Some(0.95),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
        }
    }

    async fn make_request(&self, url: String, request_body: OpenAIRequest) -> Result<reqwest::Response, AiProviderError> {
        println!("📦 Request model: {}", request_body.model);

        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", if request_body.stream { "text/event-stream" } else { "application/json" })
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AiProviderError::NetworkError(e.to_string()))
    }

    fn parse_openai_sse_line(line: &str) -> Option<Result<StreamItem, AiProviderError>> {
        if line.trim().is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];

        if data.trim() == "[DONE]" {
            return None;
        }

        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(json) => {
                // Handle standard streaming chunk
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        // Content delta
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                return Some(Ok(StreamItem::new(content.to_string())));
                            }
                        }

                        // Finish reason
                        if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                            if finish_reason != "null" {
                                return Some(Ok(StreamItem::complete(
                                    String::new(),
                                    Some(finish_reason.to_string()),
                                    0,
                                )));
                            }
                        }
                    }
                }

                // Handle error response inside stream
                if let Some(error) = json.get("error") {
                    let error_message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                    let error_type = error.get("type").and_then(|t| t.as_str()).unwrap_or("api_error");
                    return Some(Err(AiProviderError::ApiError(format!("{}: {}", error_type, error_message))));
                }

                None
            }
            Err(e) => Some(Err(AiProviderError::SerializationError(format!("Failed to parse OpenAI event: {}", e))))
        }
    }

    pub async fn trigger_stream_request(
        &self,
        system_prompt: String,
        user_prompts: Vec<String>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AiProviderError>> + Send>>, AiProviderError> {
        let _ = &self
            .rate_limiter
            .acquire()
            .await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!(
            "🚦 Rate limit: {} requests remaining this minute",
            &self.rate_limiter.check_remaining()
        );

        let url = format!("{}/chat/completions", self.base_url);
        let request_body = self.get_request(system_prompt, user_prompts, true);

        let response = self.make_request(url, request_body).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            eprintln!("❌ OpenAI API Error Response: {}", error_text);

            return Err(match status.as_u16() {
                401 => AiProviderError::AuthenticationError(error_text),
                429 => AiProviderError::ApiError(format!("Rate limit exceeded: {}", error_text)),
                _ => AiProviderError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        // Convert byte stream into newline‑delimited SSE events
        let stream = response
            .bytes_stream()
            .scan(String::new(), |buffer, chunk_result| {
                future::ready(match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);

                        let mut items = Vec::new();

                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].to_string();
                            buffer.drain(..=newline_pos);

                            if let Some(result) = Self::parse_openai_sse_line(&line) {
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

    pub async fn get_non_streaming_response(
        &self,
        system_prompt: String,
        user_prompts: Vec<String>,
    ) -> Result<String, AiProviderError> {
        let _ = &self
            .rate_limiter
            .acquire()
            .await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!(
            "🚦 Rate limit: {} requests remaining this minute",
            &self.rate_limiter.check_remaining()
        );

        let url = format!("{}/chat/completions", self.base_url);
        let request_body = self.get_request(system_prompt, user_prompts, false);

        let response = self.make_request(url, request_body).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                401 => AiProviderError::AuthenticationError(error_text),
                429 => AiProviderError::ApiError(format!("Rate limit exceeded: {}", error_text)),
                _ => AiProviderError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AiProviderError::SerializationError(e.to_string()))?;

        let content = json
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .ok_or_else(|| AiProviderError::SerializationError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    pub async fn token_count(
        &self,
        system_prompt: String,
        user_prompts: Vec<String>,
    ) -> Result<(), AiProviderError> {
        let _ = &self
            .rate_limiter
            .acquire()
            .await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!(
            "🚦 Rate limit: {} requests remaining this minute",
            &self.rate_limiter.check_remaining()
        );

        // Very rough approximation — OpenAI typically averages ~3.7 characters per token for English
        let total_chars: usize = system_prompt.len() + user_prompts.iter().map(|p| p.len()).sum::<usize>();
        let estimated_tokens = total_chars / 4; // Simplified heuristic

        println!("estimated_input_tokens = {}", estimated_tokens);
        println!("⚠️  Note: This is an estimated token count. Use a local tokenizer (e.g. tiktoken) for accurate numbers.");

        Ok(())
    }
}
use std::option::Option;
use reqwest::Client;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use async_trait::async_trait;
use futures::future;
use crate::enums::ai_provider_error::AiProviderError;
use crate::services::rate_limiter::ApiRateLimiter;
use crate::structs::ai::deepseek::deepseek_message::DeepSeekMessage;
use crate::structs::ai::deepseek::deepseek_request::DeepSeekRequest;
use crate::structs::stream_item::StreamItem;
use crate::traits::ai_provider::AiProvider;

#[derive(Clone)]
pub struct DeepSeekProvider {
    api_key: String,
    base_url: String,
    client: Client,
    model: String,
    rate_limiter: Arc<ApiRateLimiter>,
}

impl DeepSeekProvider {
    pub fn new(api_key: String, rate_limiter: Arc<ApiRateLimiter>) -> Self {
        Self {
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            client: Client::new(),
            model: "deepseek-chat".to_string(), // Default DeepSeek model
            rate_limiter,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    fn get_deepseek_messages(&self, system_prompt: String, user_prompts: Vec<String>) -> Vec<DeepSeekMessage> {
        let mut messages = Vec::new();

        // Add system message if provided
        if !system_prompt.is_empty() {
            messages.push(DeepSeekMessage {
                role: "system".to_string(),
                content: system_prompt,
            });
        }

        // Add user messages
        for prompt in user_prompts {
            messages.push(DeepSeekMessage {
                role: "user".to_string(),
                content: prompt,
            });
        }

        messages
    }

    fn get_request(&self, system_prompt: String, user_prompts: Vec<String>, stream: bool) -> DeepSeekRequest {
        let messages = self.get_deepseek_messages(system_prompt, user_prompts);

        DeepSeekRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(4096), // DeepSeek typical max tokens
            temperature: Some(1.0),
            stream,
            top_p: Some(0.95),
            frequency_penalty: Some(0.0),
            presence_penalty: Some(0.0),
        }
    }

    async fn make_request(&self, url: String, request_body: DeepSeekRequest) -> Result<reqwest::Response, AiProviderError> {
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

    fn parse_deepseek_sse_line(line: &str) -> Option<Result<StreamItem, AiProviderError>> {
        if line.trim().is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];

        if data.trim() == "[DONE]" {
            return None;
        }

        // Parse DeepSeek streaming response format
        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(json) => {
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        // Handle content delta
                        if let Some(delta) = choice.get("delta") {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                return Some(Ok(StreamItem::new(content.to_string())));
                            }
                        }

                        // Handle finish reason
                        if let Some(finish_reason) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                            if finish_reason != "null" {
                                return Some(Ok(StreamItem::complete(
                                    String::new(),
                                    Some(finish_reason.to_string()),
                                    0
                                )));
                            }
                        }
                    }
                }

                // Handle usage information
                if let Some(usage) = json.get("usage") {
                    let input_tokens = usage.get("prompt_tokens").and_then(|t| t.as_u64()).map(|t| t as u32);
                    let output_tokens = usage.get("completion_tokens").and_then(|t| t.as_u64()).map(|t| t as u32);

                    if input_tokens.is_some() || output_tokens.is_some() {
                        return Some(Ok(StreamItem::with_tokens(
                            String::new(),
                            input_tokens,
                            output_tokens,
                        )));
                    }
                }

                // Handle errors
                if let Some(error) = json.get("error") {
                    let error_message = error.get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    let error_type = error.get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("api_error");

                    return Some(Err(AiProviderError::ApiError(format!("{}: {}", error_type, error_message))));
                }

                None
            }
            Err(e) => Some(Err(AiProviderError::SerializationError(format!("Failed to parse DeepSeek event: {}", e))))
        }
    }
}

#[async_trait]
impl AiProvider for DeepSeekProvider {

    async fn stream_chat(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AiProviderError>> + Send>>, AiProviderError> {
        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

        let url = format!("{}/chat/completions", self.base_url);
        let request_body = self.get_request(system_prompt, user_prompts, true);

        let response = self.make_request(url, request_body).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            eprintln!("❌ DeepSeek API Error Response: {}", error_text);

            return Err(match status.as_u16() {
                401 => AiProviderError::AuthenticationError(error_text),
                429 => AiProviderError::ApiError(format!("Rate limit exceeded: {}", error_text)),
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

                            if let Some(result) = Self::parse_deepseek_sse_line(&line) {
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

    async fn chat(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<String, AiProviderError> {
        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

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

        let json: serde_json::Value = response.json().await
            .map_err(|e| AiProviderError::SerializationError(e.to_string()))?;

        // Extract content from DeepSeek response
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

    async fn token_count(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<(), AiProviderError> {
        // Note: DeepSeek may not have a dedicated token counting endpoint like Anthropic
        // This is a placeholder implementation - you might need to estimate tokens or use a different approach

        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

        // For now, we'll estimate token count based on character count
        // This is a rough approximation - actual token count may vary
        let total_chars: usize = system_prompt.len() + user_prompts.iter().map(|p| p.len()).sum::<usize>();
        let estimated_tokens = total_chars / 4; // Rough estimate: ~4 chars per token

        println!("estimated_input_tokens = {}", estimated_tokens);
        println!("⚠️  Note: This is an estimated token count. DeepSeek may not provide exact token counting.");

        Ok(())
    }
}

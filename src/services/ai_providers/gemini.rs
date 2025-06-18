use std::option::Option;
use reqwest::Client;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use futures::future;
use crate::enums::ai_provider_error::AiProviderError;
use crate::services::rate_limiter::ApiRateLimiter;
use crate::structs::ai::gemini::gemini_request::GeminiRequest;
use crate::structs::ai::gemini::gemini_content::GeminiContent;
use crate::structs::ai::gemini::gemini_part::GeminiPart;
use crate::structs::ai::gemini::gemini_generation_config::GeminiGenerationConfig;
use crate::structs::stream_item::StreamItem;

#[derive(Clone)]
pub struct GeminiProvider {
    api_key: String,
    base_url: String,
    client: Client,
    model: String,
    rate_limiter: Arc<ApiRateLimiter>,
}

impl GeminiProvider {
    pub fn new(api_key: String, rate_limiter: Arc<ApiRateLimiter>) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
            model: "gemini-1.5-pro".to_string(), // Default Gemini model
            rate_limiter,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    fn get_gemini_contents(&self, system_prompt: String, user_prompts: Vec<String>) -> Vec<GeminiContent> {
        let mut contents = Vec::new();

        // Gemini handles system prompt differently - it can be included as a system instruction
        // or as the first user message. For simplicity, we'll include it as the first user message
        if !system_prompt.is_empty() {
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
            });
        }

        // Add user messages
        for prompt in user_prompts {
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt,
                }],
            });
        }

        contents
    }

    fn get_request(&self, system_prompt: String, user_prompts: Vec<String>) -> GeminiRequest {
        let contents = self.get_gemini_contents(system_prompt, user_prompts);

        GeminiRequest {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: Some(1.0),
                top_p: Some(0.95),
                top_k: Some(40),
                max_output_tokens: Some(8192),
                candidate_count: Some(1),
                stop_sequences: None,
            }),
            safety_settings: None, // You can add safety settings if needed
        }
    }

    async fn make_request(&self, url: String, request_body: GeminiRequest, stream: bool) -> Result<reqwest::Response, AiProviderError> {
        println!("📦 Request model: {}", self.model);

        let mut request_builder = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body);

        if stream {
            request_builder = request_builder.header("Accept", "text/event-stream");
        }

        request_builder
            .send()
            .await
            .map_err(|e| AiProviderError::NetworkError(e.to_string()))
    }

    fn parse_gemini_sse_line(line: &str) -> Option<Result<StreamItem, AiProviderError>> {
        if line.trim().is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];

        if data.trim() == "[DONE]" {
            return None;
        }

        // Parse Gemini streaming response format
        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(json) => {
                if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                    if let Some(candidate) = candidates.first() {
                        // Handle content from candidate
                        if let Some(content) = candidate.get("content") {
                            if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                                if let Some(part) = parts.first() {
                                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                        return Some(Ok(StreamItem::new(text.to_string())));
                                    }
                                }
                            }
                        }

                        // Handle finish reason
                        if let Some(finish_reason) = candidate.get("finishReason").and_then(|f| f.as_str()) {
                            return Some(Ok(StreamItem::complete(
                                String::new(),
                                Some(finish_reason.to_string()),
                                0
                            )));
                        }
                    }
                }

                // Handle usage metadata
                if let Some(usage_metadata) = json.get("usageMetadata") {
                    let input_tokens = usage_metadata.get("promptTokenCount").and_then(|t| t.as_u64()).map(|t| t as u32);
                    let output_tokens = usage_metadata.get("candidatesTokenCount").and_then(|t| t.as_u64()).map(|t| t as u32);

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
                    let error_code = error.get("code")
                        .and_then(|c| c.as_i64())
                        .unwrap_or(0);

                    return Some(Err(AiProviderError::ApiError(format!("Code {}: {}", error_code, error_message))));
                }

                None
            }
            Err(e) => Some(Err(AiProviderError::SerializationError(format!("Failed to parse Gemini event: {}", e))))
        }
    }

    pub async fn trigger_stream_request(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AiProviderError>> + Send>>, AiProviderError> {
        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

        let url = format!("{}/models/{}:streamGenerateContent?key={}",
                          self.base_url, self.model, self.api_key);
        let request_body = self.get_request(system_prompt, user_prompts);

        let response = self.make_request(url, request_body, true).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            eprintln!("❌ Gemini API Error Response: {}", error_text);

            return Err(match status.as_u16() {
                400 => AiProviderError::ApiError(format!("Bad request: {}", error_text)),
                401 => AiProviderError::AuthenticationError(error_text),
                403 => AiProviderError::ApiError(format!("Forbidden: {}", error_text)),
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

                            if let Some(result) = Self::parse_gemini_sse_line(&line) {
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

    pub async fn get_non_streaming_response(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<String, AiProviderError> {
        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

        let url = format!("{}/models/{}:generateContent?key={}",
                          self.base_url, self.model, self.api_key);
        let request_body = self.get_request(system_prompt, user_prompts);

        let response = self.make_request(url, request_body, false).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                400 => AiProviderError::ApiError(format!("Bad request: {}", error_text)),
                401 => AiProviderError::AuthenticationError(error_text),
                403 => AiProviderError::ApiError(format!("Forbidden: {}", error_text)),
                429 => AiProviderError::ApiError(format!("Rate limit exceeded: {}", error_text)),
                _ => AiProviderError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| AiProviderError::SerializationError(e.to_string()))?;

        // Extract content from Gemini response
        let content = json
            .get("candidates")
            .and_then(|candidates| candidates.as_array())
            .and_then(|candidates| candidates.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part.get("text"))
            .and_then(|text| text.as_str())
            .ok_or_else(|| AiProviderError::SerializationError("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    pub async fn token_count(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<(), AiProviderError> {
        let _ = &self.rate_limiter.acquire().await
            .map_err(|e| AiProviderError::ApiError(format!("Rate limit error: {}", e)))?;

        println!("🚦 Rate limit: {} requests remaining this minute",
                 &self.rate_limiter.check_remaining());

        let url = format!("{}/models/{}:countTokens?key={}",
                          self.base_url, self.model, self.api_key);
        let request_body = self.get_request(system_prompt, user_prompts);

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AiProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(match status.as_u16() {
                400 => AiProviderError::ApiError(format!("Bad request: {}", error_text)),
                401 => AiProviderError::AuthenticationError(error_text),
                403 => AiProviderError::ApiError(format!("Forbidden: {}", error_text)),
                _ => AiProviderError::ApiError(format!("HTTP {}: {}", status, error_text)),
            });
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| AiProviderError::SerializationError(e.to_string()))?;

        let total_tokens = json
            .get("totalTokens")
            .and_then(|t| t.as_u64())
            .unwrap_or(0);

        println!("input_tokens = {}", total_tokens);

        Ok(())
    }
}
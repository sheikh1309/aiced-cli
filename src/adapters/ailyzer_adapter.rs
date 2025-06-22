use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use crate::errors::{AilyzerError, AilyzerResult};
use crate::logger::animated_logger::AnimatedLogger;
use crate::structs::api_response::ApiResponse;

pub struct AiLyzerAdapter {
    client: Client,
    base_url: String,
    api_key: String,
}

impl AiLyzerAdapter {

    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn post_json<T, R>(
        &self,
        endpoint: &str,
        request_body: &T,
        logger: &mut AnimatedLogger,
        operation_name: &str,
    ) -> AilyzerResult<ApiResponse<R>>  where T: Serialize, R: for<'de> Deserialize<'de>{
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));

        let response = match self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(request_body)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                logger.stop(&format!("{} failed", operation_name)).await;
                log::error!("Network error during {} request: {}", operation_name, e);
                return Err(AilyzerError::system_error(
                    "analysis Error",
                    &format!("Failed to connect to {} server", operation_name)
                ).into());
            }
        };

        let body: ApiResponse<R> = match response.status() {
            StatusCode::OK => {
                match response.json().await {
                    Ok(data) => data,
                    Err(e) => {
                        logger.stop(&format!("{} failed", operation_name)).await;
                        log::error!("Failed to parse JSON response for {}: {}", operation_name, e);
                        return Err(AilyzerError::system_error(
                            "analysis Error",
                            &format!("Invalid response format from {} server", operation_name)
                        ).into());
                    }
                }
            },
            StatusCode::REQUEST_TIMEOUT => {
                logger.stop(&format!("{} failed", operation_name)).await;
                log::error!("{} request timed out (408)", operation_name);
                return Err(AilyzerError::system_error(
                    "analysis Error",
                    &format!("{} request timed out (408)", operation_name)
                ).into());
            },
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                logger.stop(&format!("{} failed", operation_name)).await;
                log::error!("{} request failed with status {}: {}", operation_name, status, error_text);
                return Err(AilyzerError::system_error(
                    "analysis Error",
                    &format!("{} request failed with status {}: {}", operation_name, status, error_text)
                ).into());
            }
        };

        if !body.success {
            logger.stop(&format!("{} failed", operation_name)).await;
            log::error!("API returned error for {}: {}", operation_name, body.message);
            return Err(AilyzerError::system_error(
                "analysis Error",
                &format!("{} failed: {}", operation_name, body.message)
            ).into());
        }

        Ok(body)
    }

    pub async fn post_json_extract_data<T, R>(
        &self,
        endpoint: &str,
        request_body: &T,
        logger: &mut AnimatedLogger,
        operation_name: &str,
    ) -> AilyzerResult<R>  where T: Serialize, R: for<'de> Deserialize<'de> {
        let response = self.post_json(endpoint, request_body, logger, operation_name).await?;

        match response.data {
            Some(data) => Ok(data),
            None => {
                logger.stop(&format!("{} failed", operation_name)).await;
                log::error!("API response missing data field for {}", operation_name);
                Err(AilyzerError::system_error(
                    "analysis Error",
                    &format!("API response missing data field for {}", operation_name)
                ).into())
            }
        }
    }
}
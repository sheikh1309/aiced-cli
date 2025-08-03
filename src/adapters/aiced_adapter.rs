use std::sync::Arc;
use crate::errors::{AicedError, AicedResult};
use crate::structs::stream_result::StreamResult;
use crate::traits::ai_provider::AiProvider;
use futures::StreamExt;

pub struct AicedAdapter {
    ai_provider: Arc<dyn AiProvider>
}

impl AicedAdapter {

    pub fn new(ai_provider: Arc<dyn AiProvider>) -> Self {
        Self { ai_provider }
    }

    pub async fn stream_llm_chat(&self, user_prompt: String, system_prompt: String) -> AicedResult<StreamResult> {
        let mut full_content = String::new();
        let mut input_tokens = 0u32;
        let mut output_tokens = 0u32;

        let mut stream = match self.ai_provider.stream_chat(system_prompt, vec![user_prompt]).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(AicedError::system_error(
                    "analysis Error",
                    &format!("Failed to connect to {} server", e)
                ).into());
            },
        };

        let mut item_count = 0;
        while let Some(result) = stream.next().await {
            item_count += 1;

            match result {
                Ok(item) => {
                    if !item.content.is_empty() {
                        full_content.push_str(&item.content);
                    }

                    match item.input_tokens {
                        Some(usage_input_tokens) => {
                            input_tokens += usage_input_tokens;
                        },
                        None => {},
                    }

                    match item.output_tokens {
                        Some(usage_output_tokens) => {
                            output_tokens += usage_output_tokens;
                        },
                        None => {},
                    }

                    if item.is_complete {
                        break;
                    }
                }
                Err(e) => {
                    log::info!("Stream error on item #{}: {}", item_count, e);
                    return Err(AicedError::system_error(
                        "analysis Error",
                        &format!("Failed to connect to {} server", "analyze")
                    ).into());
                },
            }
        }

        Ok(StreamResult { content: full_content, input_tokens, output_tokens })
    }
}
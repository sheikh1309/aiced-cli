use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::enums::ai_provider_error::AiProviderError;
use crate::structs::stream_item::StreamItem;

#[async_trait]
pub trait AiProvider: Send + Sync {
    
    async fn stream_chat(&self, system_prompt: String, user_prompts: Vec<String>) 
        -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, AiProviderError>> + Send>>, AiProviderError>;

    async fn chat(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<String, AiProviderError>;

    async fn token_count(&self, system_prompt: String, user_prompts: Vec<String>) -> Result<(), AiProviderError>;
}

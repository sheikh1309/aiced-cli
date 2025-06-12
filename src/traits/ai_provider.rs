use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use crate::structs::message::Message;
use crate::structs::stream_item::StreamItem;

#[async_trait]
pub trait AiProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn trigger_stream_request(&self, messages: &[Message]) -> Result<Pin<Box<dyn Stream<Item = Result<StreamItem, Self::Error>> + Send>>, Self::Error>;
    
}
use futures::{Stream, StreamExt};
use std::pin::Pin;
use tokio::sync::mpsc::UnboundedSender;
use crate::structs::stream_item::StreamItem;

pub trait StreamItemLike {
    fn content(&self) -> &str;
    fn is_complete(&self) -> bool;
    fn finish_reason(&self) -> &Option<String>;
    fn create_complete(content: String, finish_reason: Option<String>) -> Self;
}

// Simplified to work with concrete StreamItem and generic error
pub async fn process_single_stream<E>(
    stream: &mut Pin<Box<dyn Stream<Item = Result<StreamItem, E>> + Send>>,
    tx: &UnboundedSender<Result<StreamItem, E>>,
) -> (String, bool)
where
    E: Send + 'static,
{
    let mut chunk_response = String::new();
    let mut was_truncated = false;

    while let Some(result) = stream.next().await {
        match result {
            Ok(item) => {
                chunk_response.push_str(item.content());

                if item.is_complete() {
                    was_truncated = item.finish_reason().as_deref() == Some("length");
                    break;
                } else {
                    let _ = tx.send(Ok(item));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e));
                return (chunk_response, false);
            }
        }
    }

    (chunk_response, was_truncated)
}

pub fn send_final_completion<E>(tx: &UnboundedSender<Result<StreamItem, E>>)
where
    E: Send + 'static,
{
    let final_item = StreamItem::create_complete("".to_string(), Some("stop".to_string()));
    let _ = tx.send(Ok(final_item));
}
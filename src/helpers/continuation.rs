use crate::structs::message::Message;
use crate::structs::stream_item::StreamItem;
use crate::traits::stream_processor::{process_single_stream, send_final_completion};
use tokio::sync::mpsc::UnboundedSender;
use crate::traits::ai_provider::AiProvider;

pub async fn run_continuation_task<T>(
    provider: T,
    mut messages: Vec<Message>,
    tx: UnboundedSender<Result<StreamItem, T::Error>>,
) where
    T: AiProvider + Send + 'static,
    T::Error: Send + 'static,
{
    let mut full_response = String::new();

    loop {
        let mut stream = match provider.create_stream_request(&messages).await {
            Ok(stream) => stream,
            Err(e) => {
                let _ = tx.send(Err(e));
                return;
            }
        };

        let (chunk_text, was_truncated) = process_single_stream(&mut stream, &tx).await;
        full_response.push_str(&chunk_text);

        if !was_truncated {
            send_final_completion(&tx);
            return;
        }

        add_continuation_messages(&mut messages, &full_response);
    }
}

pub fn add_continuation_messages(messages: &mut Vec<Message>, full_response: &str) {
    messages.push(Message {
        role: "assistant".to_string(),
        content: full_response.to_string(),
    });

    messages.push(Message {
        role: "user".to_string(),
        content: "Please continue where you left off.".to_string(),
    });
}
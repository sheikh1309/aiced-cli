use std::io::Write;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct AnimatedLogger {
    message: String,
    animation_chars: Vec<&'static str>,
    stop_sender: Option<mpsc::UnboundedSender<()>>,
    task_handle: Option<JoinHandle<()>>,
}

impl AnimatedLogger {
    pub fn new(message: String) -> Self {
        let animation_chars = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        Self {
            message,
            animation_chars,
            stop_sender: None,
            task_handle: None,
        }
    }

    pub fn start(&mut self) {
        let (stop_tx, mut stop_rx) = mpsc::unbounded_channel();
        let message = self.message.clone();
        let animation_chars = self.animation_chars.clone();

        let handle = tokio::spawn(async move {
            let mut frame = 0;
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(150));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        eprint!("\r{} {} ", message, animation_chars[frame]);
                        std::io::stderr().flush().unwrap();
                        frame = (frame + 1) % animation_chars.len();
                    }
                    _ = stop_rx.recv() => {
                        break;
                    }
                }
            }
        });

        self.stop_sender = Some(stop_tx);
        self.task_handle = Some(handle);
    }

    pub async fn stop(&mut self, final_message: &str) {
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        eprint!("\r\x1b[K✅  {}\n", final_message);
        std::io::stderr().flush().unwrap();
    }

    pub async fn error(&mut self, error_message: &str) {
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }

        if let Some(handle) = self.task_handle.take() {
            let _ = handle.await;
        }

        eprint!("\r\x1b[K❌ {}\n", error_message);
        std::io::stderr().flush().unwrap();
    }
}
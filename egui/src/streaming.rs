use std::sync::mpsc;

use crate::api::{self, LlmEvent};
use crate::model::{ChatMessage, ProviderConfig};

pub struct StreamBridge {
    runtime: tokio::runtime::Runtime,
    receiver: Option<mpsc::Receiver<LlmEvent>>,
}

impl StreamBridge {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        Self {
            runtime,
            receiver: None,
        }
    }

    pub fn start(
        &mut self,
        config: ProviderConfig,
        messages: Vec<ChatMessage>,
        ctx: egui::Context,
    ) {
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);

        self.runtime.spawn(async move {
            use futures::StreamExt;
            let mut stream = api::stream_completion(config, messages);
            while let Some(event) = stream.next().await {
                let done = matches!(event, LlmEvent::Done | LlmEvent::Error(_));
                if tx.send(event).is_err() {
                    break;
                }
                ctx.request_repaint();
                if done {
                    break;
                }
            }
        });
    }

    pub fn poll(&self) -> Vec<LlmEvent> {
        let mut events = Vec::new();
        if let Some(ref rx) = self.receiver {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }
        events
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        self.receiver = None;
    }
}

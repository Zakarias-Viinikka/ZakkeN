use error_stuff::CqrsErr;
use futures_channel::mpsc::{unbounded, UnboundedReceiver};
use futures_channel::oneshot;
use futures_util::StreamExt;
use leptos::prelude::*;

use super::flush::flush_pending_writes;
use super::handle::{BatcherHandle, PendingWrite};

pub(crate) enum BatcherMsg {
    Poke,
    Flush { reply: oneshot::Sender<Result<(), CqrsErr>> },
}

enum TimerOrMessage {
    TimerExpired,
    Message(BatcherMsg),
    ChannelClosed,
}

/// Batches text edits so every keyboard input isn't an edit. Also
/// supports adding undo/redo support more cleanly.
pub struct Batcher {
    pub(crate) pending: RwSignal<Vec<PendingWrite>>,
    receiver: UnboundedReceiver<BatcherMsg>,
}

impl Batcher {
    pub fn new() -> (Self, BatcherHandle) {
        let pending = RwSignal::new(Vec::new());
        let (sender, receiver) = unbounded();
        (
            Batcher { pending, receiver },
            BatcherHandle { pending, sender },
        )
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.next().await {
            match msg {
                BatcherMsg::Poke => self.debounce_then_flush().await,
                BatcherMsg::Flush { reply } => {
                    let result = self.flush().await;
                    let _ = reply.send(result);
                }
            }
        }
    }

    async fn debounce_then_flush(&mut self) {
        loop {
            match self.next_timer_or_message(500).await {
                TimerOrMessage::TimerExpired => {
                    let _ = self.flush().await;
                    return;
                }
                TimerOrMessage::Message(BatcherMsg::Poke) => continue,
                TimerOrMessage::Message(BatcherMsg::Flush { reply }) => {
                    let result = self.flush().await;
                    let _ = reply.send(result);
                    return;
                }
                TimerOrMessage::ChannelClosed => {
                    let _ = self.flush().await;
                    return;
                }
            }
        }
    }

    async fn next_timer_or_message(&mut self, timeout_ms: u32) -> TimerOrMessage {
        let timeout = gloo_timers::future::TimeoutFuture::new(timeout_ms);
        futures_util::pin_mut!(timeout);

        let next = self.receiver.next();
        futures_util::pin_mut!(next);

        match futures_util::future::select(timeout, next).await {
            futures_util::future::Either::Left(_) => TimerOrMessage::TimerExpired,
            futures_util::future::Either::Right((Some(msg), _)) => TimerOrMessage::Message(msg),
            futures_util::future::Either::Right((None, _)) => TimerOrMessage::ChannelClosed,
        }
    }

    async fn flush(&mut self) -> Result<(), CqrsErr> {
        flush_pending_writes(self.pending).await
    }
}
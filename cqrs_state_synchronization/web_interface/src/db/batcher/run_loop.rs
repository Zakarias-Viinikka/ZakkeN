use error_stuff::CqrsErr;
use futures_channel::mpsc::UnboundedReceiver;
use futures_channel::oneshot;
use futures_util::StreamExt;

use super::flush::flush_pending_writes;
use super::handle::PendingWrite;

pub(crate) enum BatcherMsg {
    Push(PendingWrite),
    Flush {
        reply: oneshot::Sender<Result<(), CqrsErr>>,
    },
}

enum TimerOrMessage {
    TimerExpired,
    Message(BatcherMsg),
    ChannelClosed,
}

pub(crate) async fn run_batcher_task(
    mut rx_receive_message_from_handle: UnboundedReceiver<BatcherMsg>,
) {
    let mut pending: Vec<PendingWrite> = Vec::new();
    while let Some(msg) = rx_receive_message_from_handle.next().await {
        match msg {
            BatcherMsg::Push(write) => {
                pending.push(write);
                debounce_then_flush(&mut pending, &mut rx_receive_message_from_handle).await;
            }
            BatcherMsg::Flush { reply } => {
                let result = flush_pending_writes(&mut pending).await;
                let _ = reply.send(result);
            }
        }
    }
}

async fn debounce_then_flush(
    pending: &mut Vec<PendingWrite>,
    rx_receive_message_from_handle: &mut UnboundedReceiver<BatcherMsg>,
) {
    loop {
        match next_timer_or_message(rx_receive_message_from_handle, 500).await {
            TimerOrMessage::TimerExpired => {
                let _ = flush_pending_writes(pending).await;
                return;
            }
            TimerOrMessage::Message(BatcherMsg::Push(write)) => {
                pending.push(write);
                continue;
            }
            TimerOrMessage::Message(BatcherMsg::Flush { reply }) => {
                let result = flush_pending_writes(pending).await;
                let _ = reply.send(result);
                return;
            }
            TimerOrMessage::ChannelClosed => {
                let _ = flush_pending_writes(pending).await;
                return;
            }
        }
    }
}

async fn next_timer_or_message(
    rx_receive_message_from_handle: &mut UnboundedReceiver<BatcherMsg>,
    timeout_ms: u32,
) -> TimerOrMessage {
    let timeout = gloo_timers::future::TimeoutFuture::new(timeout_ms);
    futures_util::pin_mut!(timeout);

    let next = rx_receive_message_from_handle.next();
    futures_util::pin_mut!(next);

    match futures_util::future::select(timeout, next).await {
        futures_util::future::Either::Left(_) => TimerOrMessage::TimerExpired,
        futures_util::future::Either::Right((Some(msg), _)) => TimerOrMessage::Message(msg),
        futures_util::future::Either::Right((None, _)) => TimerOrMessage::ChannelClosed,
    }
}
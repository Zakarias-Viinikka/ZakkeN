use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::StreamExt;
use leptos::task::spawn_local;
use server_protocol::{Response, UnsentLetter};

use super::enter_lonewolf;

/// Owns every spontaneous response the server pushes. Love letters get
/// forwarded to the task that applies them. A "your drift version
/// changed" push enters lonewolf with whatever reason the server gave.
pub fn spawn_owner_of_spontaneous_messages_from_server(
    mut rx_forward_spontaneous_server_message: UnboundedReceiver<Response>,
    tx_give_love_letter_to_receiving_task: UnboundedSender<(UnsentLetter, String)>,
) {
    spawn_local(async move {
        while let Some(response) = rx_forward_spontaneous_server_message.next().await {
            match response {
                Response::LoveLetterFromServer {
                    letter,
                    drift_version,
                } => {
                    let _ = tx_give_love_letter_to_receiving_task
                        .unbounded_send((letter, drift_version));
                }
                Response::TheDriftVersionChanged {
                    drift_version: _,
                    drift_reason,
                } => {
                    let reason = if drift_reason.is_empty() {
                        None
                    } else {
                        Some(drift_reason)
                    };
                    enter_lonewolf(reason);
                }
                Response::Reply { .. } => {
                    // Replies never come down this channel.
                }
            }
        }
    });
}

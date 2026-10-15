use crate::commands::Command;
use crate::handle::FanOutReport;
use error_stuff::{LogCategory, LogLevel, ServerLogEntry, ServErr};
use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// The single owner of the user_name -> push-sender map. Every handle
/// talks to this loop and nothing else touches the map.
///
/// When a client's channel turns out to be gone, the loop logs it through
/// `log_tx`. That's how this crate reports something without depending on
/// db_core: it sends a `ServerLogEntry` and the executor writes the row.
pub async fn run(
    mut alert_connected_clients_manager_to_fan_out_rx: UnboundedReceiver<Command>,
    log_tx: UnboundedSender<ServerLogEntry>,
) {
    let mut senders: HashMap<String, UnboundedSender<i64>> = HashMap::new();

    while let Some(cmd) = alert_connected_clients_manager_to_fan_out_rx.recv().await {
        match cmd {
            Command::AddClient { user_name, sender } => {
                senders.insert(user_name, sender);
            }
            Command::RemoveClient { user_name } => {
                senders.remove(&user_name);
            }
            Command::SendTo {
                user_name,
                letter_id,
                reply,
            } => {
                let result = match senders.get(&user_name) {
                    Some(tx) => tx.send(letter_id).map_err(|_| ServErr::ClientGone {
                        user_name: user_name.clone(),
                    }),
                    None => Err(ServErr::ClientGone {
                        user_name: user_name.clone(),
                    }),
                };
                let _ = reply.send(result);
            }
            Command::IsConnected { user_name, reply } => {
                let _ = reply.send(senders.contains_key(&user_name));
            }
            Command::ConnectedUserNames { reply } => {
                let _ = reply.send(senders.keys().cloned().collect());
            }
            Command::FanOut {
                letter_id,
                except,
                reply,
            } => {
                let mut reached = Vec::new();
                let mut dropped = Vec::new();
                let mut to_remove = Vec::new();
                for (name, tx) in &senders {
                    if Some(name.as_str()) == except.as_deref() {
                        continue;
                    }
                    match tx.send(letter_id) {
                        Ok(()) => reached.push(name.clone()),
                        Err(_) => {
                            dropped.push(name.clone());
                            to_remove.push(name.clone());
                        }
                    }
                }
                for name in &to_remove {
                    senders.remove(name);
                    let _ = log_tx.send(ServerLogEntry::now(
                        LogLevel::Warn,
                        LogCategory::Connection,
                        "connected_clients::task::run",
                        String::new(),
                        format!("lost contact with {name}, dropped them"),
                    ));
                }
                let _ = reply.send(FanOutReport { reached, dropped });
            }
        }
    }
}
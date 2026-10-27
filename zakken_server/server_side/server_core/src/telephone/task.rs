use super::commands::Command;
use super::handle::NotifyReport;
use error_stuff::{ErrorDetails, LogCategory, LogLevel, ServerLogEntry, ServErr};
use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

struct SocketEntry {
    sender: UnboundedSender<()>,
    user_name: Option<String>,
}

pub async fn run(
    mut alert_telephone_manager_to_nudge_rx: UnboundedReceiver<Command>,
    log_tx: UnboundedSender<ServerLogEntry>,
) {
    let mut sockets: HashMap<u64, SocketEntry> = HashMap::new();

    while let Some(cmd) = alert_telephone_manager_to_nudge_rx.recv().await {
        match cmd {
            Command::RegisterSocket { socket_id, sender } => {
                sockets.insert(socket_id, SocketEntry { sender, user_name: None });
            }
            Command::RemoveSocket { socket_id } => {
                sockets.remove(&socket_id);
            }
            Command::AttachName { socket_id, user_name, reply } => {
                if let Some(entry) = sockets.get_mut(&socket_id) {
                    entry.user_name = Some(user_name);
                }
                let _ = reply.send(());
            }
            Command::UserOf { socket_id, reply } => {
                let name = sockets.get(&socket_id).and_then(|e| e.user_name.clone());
                let _ = reply.send(name);
            }
            Command::NotifySocket { socket_id, reply } => {
                let result = match sockets.get(&socket_id) {
                    Some(entry) => entry.sender.send(()).map_err(|_| {
                        ServErr::ServerError(ErrorDetails {
                            err_msg: format!("socket {socket_id}'s channel is gone"),
                            file: file!().to_string(),
                            method: "telephone::task::run".to_string(),
                            probable_cause: String::new(),
                        })
                    }),
                    None => Err(ServErr::ServerError(ErrorDetails {
                        err_msg: format!("socket {socket_id} is not registered"),
                        file: file!().to_string(),
                        method: "telephone::task::run".to_string(),
                        probable_cause: String::new(),
                    })),
                };
                let _ = reply.send(result);
            }
            Command::IsConnected { user_name, reply } => {
                let connected = sockets
                    .values()
                    .any(|e| e.user_name.as_deref() == Some(user_name.as_str()));
                let _ = reply.send(connected);
            }
            Command::ConnectedUserNames { reply } => {
                let names = sockets
                    .values()
                    .filter_map(|e| e.user_name.clone())
                    .collect();
                let _ = reply.send(names);
            }
            Command::NotifyAll { except, reply } => {
                let mut reached = Vec::new();
                let mut dropped = Vec::new();
                let mut to_remove = Vec::new();
                for (socket_id, entry) in &sockets {
                    let Some(name) = &entry.user_name else {
                        continue;
                    };
                    if Some(name.as_str()) == except.as_deref() {
                        continue;
                    }
                    match entry.sender.send(()) {
                        Ok(()) => reached.push(name.clone()),
                        Err(_) => {
                            dropped.push(name.clone());
                            to_remove.push(*socket_id);
                        }
                    }
                }
                for socket_id in &to_remove {
                    sockets.remove(socket_id);
                    let _ = log_tx.send(ServerLogEntry::now(
                        LogLevel::Warn,
                        LogCategory::Connection,
                        "telephone::task::run",
                        String::new(),
                        format!("lost contact with socket {socket_id}, dropped it"),
                    ));
                }
                let _ = reply.send(NotifyReport { reached, dropped });
            }
        }
    }
}
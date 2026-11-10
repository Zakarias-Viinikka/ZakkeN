mod lookups;
mod notifications;

use super::commands::Command;
use super::telephone_command::TelephoneCommand;
use server_errors::ServerLogEntry;
use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

struct SocketEntry {
    sender: UnboundedSender<TelephoneCommand>,
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
                sockets.insert(
                    socket_id,
                    SocketEntry {
                        sender,
                        user_name: None,
                    },
                );
            }
            Command::RemoveSocket { socket_id } => {
                sockets.remove(&socket_id);
            }
            Command::AttachName {
                socket_id,
                user_name,
                reply,
            } => {
                if let Some(entry) = sockets.get_mut(&socket_id) {
                    entry.user_name = Some(user_name);
                }
                let _ = reply.send(());
            }
            Command::UserOf { socket_id, reply } => {
                lookups::user_of(&sockets, socket_id, reply);
            }
            Command::NotifySocket { socket_id, reply } => {
                notifications::notify_one(&sockets, socket_id, reply);
            }
            Command::IsConnected { user_name, reply } => {
                lookups::is_connected(&sockets, &user_name, reply);
            }
            Command::ConnectedUserNames { reply } => {
                lookups::connected_user_names(&sockets, reply);
            }
            Command::NotifyAll { except, reply } => {
                notifications::notify_all(&mut sockets, &log_tx, except, reply);
            }
            Command::NotifyDrift {
                dont_notify_socket_with_id,
                reply,
            } => {
                notifications::notify_drift(
                    &mut sockets,
                    &log_tx,
                    dont_notify_socket_with_id,
                    reply,
                );
            }
        }
    }
}

use super::SocketEntry;
use std::collections::HashMap;
use tokio::sync::oneshot;

pub fn user_of(
    sockets: &HashMap<u64, SocketEntry>,
    socket_id: u64,
    reply: oneshot::Sender<Option<String>>,
) {
    let name = sockets
        .get(&socket_id)
        .and_then(|entry| entry.user_name.clone());
    let _ = reply.send(name);
}

pub fn is_connected(
    sockets: &HashMap<u64, SocketEntry>,
    user_name: &str,
    reply: oneshot::Sender<bool>,
) {
    let connected = sockets
        .values()
        .any(|entry| entry.user_name.as_deref() == Some(user_name));
    let _ = reply.send(connected);
}

pub fn connected_user_names(
    sockets: &HashMap<u64, SocketEntry>,
    reply: oneshot::Sender<Vec<String>>,
) {
    let names = sockets
        .values()
        .filter_map(|entry| entry.user_name.clone())
        .collect();
    let _ = reply.send(names);
}

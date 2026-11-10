use super::SocketEntry;
use crate::telephone::handle::NotifyReport;
use crate::telephone::telephone_command::TelephoneCommand;
use error_stuff::{ErrorDetails, LogCategory, LogLevel, ServerLogEntry, ServErr};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

fn drop_dead_sockets(
    sockets: &mut HashMap<u64, SocketEntry>,
    dead: Vec<u64>,
    log_tx: &UnboundedSender<ServerLogEntry>,
) {
    for socket_id in dead {
        sockets.remove(&socket_id);
        let _ = log_tx.send(ServerLogEntry::now(
            LogLevel::Warn,
            LogCategory::Connection,
            "telephone::task::notifications::drop_dead_sockets",
            String::new(),
            format!("lost contact with socket {socket_id}, dropped it"),
        ));
    }
}

pub fn notify_one(
    sockets: &HashMap<u64, SocketEntry>,
    socket_id: u64,
    reply: oneshot::Sender<Result<(), ServErr>>,
) {
    let result = match sockets.get(&socket_id) {
        Some(entry) => entry
            .sender
            .send(TelephoneCommand::GoPullYourUnsentLetters)
            .map_err(|_| {
                ServErr::ServerError(ErrorDetails {
                    err_msg: format!("socket {socket_id}'s channel is gone"),
                    file: file!().to_string(),
                    method: "telephone::task::notifications::notify_one".to_string(),
                    probable_cause: String::new(),
                })
            }),
        None => Err(ServErr::ServerError(ErrorDetails {
            err_msg: format!("socket {socket_id} is not registered"),
            file: file!().to_string(),
            method: "telephone::task::notifications::notify_one".to_string(),
            probable_cause: String::new(),
        })),
    };
    let _ = reply.send(result);
}

pub fn notify_all(
    sockets: &mut HashMap<u64, SocketEntry>,
    log_tx: &UnboundedSender<ServerLogEntry>,
    except: Option<String>,
    reply: oneshot::Sender<NotifyReport>,
) {
    let mut reached = Vec::new();
    let mut dropped = Vec::new();
    let mut to_remove = Vec::new();
    for (socket_id, entry) in sockets.iter() {
        let Some(name) = &entry.user_name else {
            continue;
        };
        if Some(name.as_str()) == except.as_deref() {
            continue;
        }
        match entry.sender.send(TelephoneCommand::GoPullYourUnsentLetters) {
            Ok(()) => reached.push(name.clone()),
            Err(_) => {
                dropped.push(name.clone());
                to_remove.push(*socket_id);
            }
        }
    }
    drop_dead_sockets(sockets, to_remove, log_tx);
    let _ = reply.send(NotifyReport { reached, dropped });
}

pub fn notify_drift(
    sockets: &mut HashMap<u64, SocketEntry>,
    log_tx: &UnboundedSender<ServerLogEntry>,
    dont_notify_socket_with_id: Option<u64>,
    reply: oneshot::Sender<usize>,
) {
    let mut reached = 0usize;
    let mut to_remove = Vec::new();
    for (socket_id, entry) in sockets.iter() {
        if entry.user_name.is_none() {
            continue;
        }
        if Some(*socket_id) == dont_notify_socket_with_id {
            continue;
        }
        match entry
            .sender
            .send(TelephoneCommand::TellTheClientTheDriftVersionChangedThenClose)
        {
            Ok(()) => reached += 1,
            Err(_) => to_remove.push(*socket_id),
        }
    }
    drop_dead_sockets(sockets, to_remove, log_tx);
    let _ = reply.send(reached);
}

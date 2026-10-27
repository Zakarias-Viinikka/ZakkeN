use super::commands::Command;
use error_stuff::{ErrorDetails, ServerLogEntry, ServErr};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct NudgeReport {
    pub reached: Vec<String>,
    pub dropped: Vec<String>,
}

#[derive(Clone)]
pub struct Telephone {
    alert_telephone_manager_to_nudge_tx: mpsc::UnboundedSender<Command>,
}

impl Telephone {
    pub fn new(log_tx: mpsc::UnboundedSender<ServerLogEntry>) -> Self {
        let (alert_telephone_manager_to_nudge_tx, alert_telephone_manager_to_nudge_rx) =
            mpsc::unbounded_channel();
        tokio::spawn(super::task::run(
            alert_telephone_manager_to_nudge_rx,
            log_tx,
        ));
        Self {
            alert_telephone_manager_to_nudge_tx,
        }
    }

    /// Register a socket at birth. The sender is stored under the socket id;
    /// the socket already holds the receiver and starts listening right away.
    pub fn register_socket(&self, socket_id: u64, sender: mpsc::UnboundedSender<()>) {
        let _ = self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::RegisterSocket { socket_id, sender });
    }

    /// Forget a socket at disconnect.
    pub fn remove_socket(&self, socket_id: u64) {
        let _ = self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::RemoveSocket { socket_id });
    }

    /// Attach a user name to a socket. Waits until the telephone's task
    /// has recorded it, so a following `user_of` sees it.
    pub async fn attach_name(&self, socket_id: u64, user_name: String) {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::AttachName {
                socket_id,
                user_name,
                reply: reply_tx,
            })
            .is_err()
        {
            return;
        }
        let _ = reply_rx.await;
    }

    /// The name attached to this socket, or None if no login has happened.
    pub async fn user_of(&self, socket_id: u64) -> Option<String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::UserOf {
                socket_id,
                reply: reply_tx,
            })
            .is_err()
        {
            return None;
        }
        reply_rx.await.unwrap_or(None)
    }

    /// Nudge one specific socket by id.
    pub async fn nudge_socket(&self, socket_id: u64) -> Result<(), ServErr> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::NudgeSocket {
                socket_id,
                reply: reply_tx,
            })
            .is_err()
        {
            return Err(task_gone("nudge_socket"));
        }
        match reply_rx.await {
            Ok(r) => r,
            Err(_) => Err(task_gone("nudge_socket")),
        }
    }

    /// Nudge every logged-in socket except `except`.
    pub async fn nudge_all(&self, except: Option<String>) -> NudgeReport {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::NudgeAll {
                except,
                reply: reply_tx,
            })
            .is_err()
        {
            return NudgeReport {
                reached: Vec::new(),
                dropped: Vec::new(),
            };
        }
        reply_rx.await.unwrap_or(NudgeReport {
            reached: Vec::new(),
            dropped: Vec::new(),
        })
    }

    pub async fn is_connected(&self, user_name: String) -> bool {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::IsConnected {
                user_name,
                reply: reply_tx,
            })
            .is_err()
        {
            return false;
        }
        reply_rx.await.unwrap_or(false)
    }

    pub async fn connected_user_names(&self) -> Vec<String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::ConnectedUserNames { reply: reply_tx })
            .is_err()
        {
            return Vec::new();
        }
        reply_rx.await.unwrap_or_default()
    }
}

fn task_gone(method: &str) -> ServErr {
    ServErr::ServerError(ErrorDetails {
        err_msg: "the telephone's task is not running".to_string(),
        file: file!().to_string(),
        method: method.to_string(),
        probable_cause: "the server is shutting down".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn add(telephone: &Telephone, socket_id: u64, name: &str) -> UnboundedReceiver<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        telephone.register_socket(socket_id, tx);
        telephone.attach_name(socket_id, name.to_string()).await;
        rx
    }

    #[tokio::test]
    async fn nudge_all_reaches_everyone_except_the_sender() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let telephone = Telephone::new(log_tx);
        let mut alice = add(&telephone, 1, "alice").await;
        let mut bob = add(&telephone, 2, "bob").await;
        let mut carol = add(&telephone, 3, "carol").await;

        let report = telephone.nudge_all(Some("alice".to_string())).await;

        let mut reached = report.reached.clone();
        reached.sort();
        assert_eq!(reached, vec!["bob".to_string(), "carol".to_string()]);
        assert!(report.dropped.is_empty());
        assert_eq!(alice.try_recv().ok(), None);
        assert_eq!(bob.try_recv().ok(), Some(()));
        assert_eq!(carol.try_recv().ok(), Some(()));
    }

    #[tokio::test]
    async fn nudge_all_drops_a_client_whose_receiver_is_gone() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let telephone = Telephone::new(log_tx);
        let bob = add(&telephone, 2, "bob").await;
        drop(bob);

        let report = telephone.nudge_all(None).await;

        assert!(report.reached.is_empty());
        assert_eq!(report.dropped, vec!["bob".to_string()]);
        assert!(!telephone.is_connected("bob".to_string()).await);
    }

    #[tokio::test]
    async fn a_removed_client_is_not_nudged() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let telephone = Telephone::new(log_tx);
        let mut alice = add(&telephone, 1, "alice").await;
        telephone.remove_socket(1);

        let report = telephone.nudge_all(None).await;

        assert!(report.reached.is_empty());
        assert!(report.dropped.is_empty());
        assert_eq!(alice.try_recv().ok(), None);
    }
}
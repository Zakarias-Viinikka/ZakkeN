use crate::commands::Command;
use error_stuff::{ErrorDetails, ServerLogEntry, ServErr};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct FanOutReport {
    pub reached: Vec<String>,
    pub dropped: Vec<String>,
}

/// A handle to the connections task. Cheap to clone. All methods send a
/// command to the task and, where an answer is needed, await a reply.
#[derive(Clone)]
pub struct ConnectedClients {
    alert_connected_clients_manager_to_fan_out_tx: mpsc::UnboundedSender<Command>,
}

impl ConnectedClients {
    /// Spawn the task that owns the map and return a handle to it.
    /// `log_tx` is where the task sends a log entry when it drops a client
    /// whose channel turned out to be gone.
    pub fn new(log_tx: mpsc::UnboundedSender<ServerLogEntry>) -> Self {
        let (
            alert_connected_clients_manager_to_fan_out_tx,
            alert_connected_clients_manager_to_fan_out_rx,
        ) = mpsc::unbounded_channel();
        tokio::spawn(crate::task::run(
            alert_connected_clients_manager_to_fan_out_rx,
            log_tx,
        ));
        Self {
            alert_connected_clients_manager_to_fan_out_tx,
        }
    }

    /// Register a new connection for `user_name`, returning the receiver
    /// half of that user's push channel. The caller keeps the receiver
    /// and reads pushed letter ids from it.
    ///
    /// Sync on purpose: login has to call this inside the same db-lock
    /// critical section that runs the backfill, so a new letter can't
    /// overtake the backlog. An async version would force the lock to be
    /// released in between.
    ///
    /// If the task is gone, the server is shutting down; a receiver that
    /// will never fire is the logically correct thing to hand back.
    pub fn add_client(&self, user_name: String) -> UnboundedReceiver<i64> {
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = self
            .alert_connected_clients_manager_to_fan_out_tx
            .send(Command::AddClient {
                user_name,
                sender: tx,
            });
        rx
    }

    /// Forget `user_name`. Fire-and-forget: nothing needs a reply. If the
    /// task is gone, the server is shutting down and there is nothing to
    /// forget.
    pub fn remove_client(&self, user_name: String) {
        let _ = self
            .alert_connected_clients_manager_to_fan_out_tx
            .send(Command::RemoveClient { user_name });
    }

    pub async fn send_to(&self, user_name: String, letter_id: i64) -> Result<(), ServErr> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_connected_clients_manager_to_fan_out_tx
            .send(Command::SendTo {
                user_name: user_name.clone(),
                letter_id,
                reply: reply_tx,
            })
            .is_err()
        {
            return Err(task_gone("send_to"));
        }
        match reply_rx.await {
            Ok(r) => r,
            Err(_) => Err(task_gone("send_to")),
        }
    }

    /// False if the task is gone, which is logically correct: nothing is
    /// connected if the connections task isn't running.
    pub async fn is_connected(&self, user_name: String) -> bool {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_connected_clients_manager_to_fan_out_tx
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

    /// Empty if the task is gone, for the same reason as `is_connected`.
    pub async fn connected_user_names(&self) -> Vec<String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_connected_clients_manager_to_fan_out_tx
            .send(Command::ConnectedUserNames { reply: reply_tx })
            .is_err()
        {
            return Vec::new();
        }
        reply_rx.await.unwrap_or_default()
    }

    /// Push `letter_id` to every connected user except `except`. Dead
    /// channels are dropped from the map and reported. Reaching nobody is
    /// not an error — it just means nobody else is online.
    ///
    /// An empty report if the task is gone, for the same reason as
    /// `is_connected`.
    pub async fn fan_out(&self, letter_id: i64, except: Option<String>) -> FanOutReport {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_connected_clients_manager_to_fan_out_tx
            .send(Command::FanOut {
                letter_id,
                except,
                reply: reply_tx,
            })
            .is_err()
        {
            return FanOutReport {
                reached: Vec::new(),
                dropped: Vec::new(),
            };
        }
        reply_rx.await.unwrap_or(FanOutReport {
            reached: Vec::new(),
            dropped: Vec::new(),
        })
    }
}

fn task_gone(method: &str) -> ServErr {
    ServErr::ServerError(ErrorDetails {
        err_msg: "the connected_clients task is not running".to_string(),
        file: file!().to_string(),
        method: method.to_string(),
        probable_cause: "the server is shutting down".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fan_out_reaches_everyone_except_the_sender() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let clients = ConnectedClients::new(log_tx);
        let mut alice = clients.add_client("alice".to_string());
        let mut bob = clients.add_client("bob".to_string());
        let mut carol = clients.add_client("carol".to_string());

        let report = clients.fan_out(7, Some("alice".to_string())).await;

        let mut reached = report.reached.clone();
        reached.sort();
        assert_eq!(reached, vec!["bob".to_string(), "carol".to_string()]);
        assert!(report.dropped.is_empty());
        assert_eq!(alice.try_recv().ok(), None);
        assert_eq!(bob.try_recv().ok(), Some(7));
        assert_eq!(carol.try_recv().ok(), Some(7));
    }

    #[tokio::test]
    async fn fan_out_drops_a_client_whose_receiver_is_gone() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let clients = ConnectedClients::new(log_tx);
        let bob = clients.add_client("bob".to_string());
        drop(bob);

        let report = clients.fan_out(7, None).await;

        assert!(report.reached.is_empty());
        assert_eq!(report.dropped, vec!["bob".to_string()]);
        assert!(!clients.is_connected("bob".to_string()).await);
    }

    #[tokio::test]
    async fn a_removed_client_is_not_reached() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let clients = ConnectedClients::new(log_tx);
        let mut alice = clients.add_client("alice".to_string());
        clients.remove_client("alice".to_string());

        let report = clients.fan_out(7, None).await;

        assert!(report.reached.is_empty());
        assert!(report.dropped.is_empty());
        assert_eq!(alice.try_recv().ok(), None);
    }
}
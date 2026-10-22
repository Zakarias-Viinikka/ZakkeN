use crate::commands::Command;
use error_stuff::{ErrorDetails, ServerLogEntry, ServErr};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub struct NudgeReport {
    pub reached: Vec<String>,
    pub dropped: Vec<String>,
}

/// A handle to the telephone's task. Cheap to clone. All methods send a
/// command to the task and, where an answer is needed, await a reply.
#[derive(Clone)]
pub struct Telephone {
    alert_telephone_manager_to_nudge_tx: mpsc::UnboundedSender<Command>,
}

impl Telephone {
    /// Spawn the task that owns the map and return a handle to it.
    /// `log_tx` is where the task sends a log entry when it drops a client
    /// whose channel turned out to be gone.
    pub fn new(log_tx: mpsc::UnboundedSender<ServerLogEntry>) -> Self {
        let (alert_telephone_manager_to_nudge_tx, alert_telephone_manager_to_nudge_rx) =
            mpsc::unbounded_channel();
        tokio::spawn(crate::task::run(
            alert_telephone_manager_to_nudge_rx,
            log_tx,
        ));
        Self {
            alert_telephone_manager_to_nudge_tx,
        }
    }

    /// Register a new connection for `user_name`, returning the receiver
    /// half of that user's nudge channel. The caller keeps the receiver
    /// and reads nudges from it.
    ///
    /// If the task is gone, the server is shutting down; a receiver that
    /// will never fire is the logically correct thing to hand back.
    pub fn add_client(&self, user_name: String) -> UnboundedReceiver<()> {
        let (tx, rx) = mpsc::unbounded_channel();
        let _ = self
            .alert_telephone_manager_to_nudge_tx
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
            .alert_telephone_manager_to_nudge_tx
            .send(Command::RemoveClient { user_name });
    }

    /// Nudge one client to request its unsent letters.
    pub async fn nudge(&self, user_name: String) -> Result<(), ServErr> {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self
            .alert_telephone_manager_to_nudge_tx
            .send(Command::Nudge {
                user_name: user_name.clone(),
                reply: reply_tx,
            })
            .is_err()
        {
            return Err(task_gone("nudge"));
        }
        match reply_rx.await {
            Ok(r) => r,
            Err(_) => Err(task_gone("nudge")),
        }
    }

    /// False if the task is gone, which is logically correct: nothing is
    /// connected if the telephone's task isn't running.
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

    /// Empty if the task is gone, for the same reason as `is_connected`.
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

    /// Nudge every connected client except `except`. Dead channels are
    /// dropped from the map and reported. Reaching nobody is not an
    /// error — it just means nobody else is online.
    ///
    /// An empty report if the task is gone, for the same reason as
    /// `is_connected`.
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

    #[tokio::test]
    async fn nudge_all_reaches_everyone_except_the_sender() {
        let (log_tx, _log_rx) = mpsc::unbounded_channel();
        let telephone = Telephone::new(log_tx);
        let mut alice = telephone.add_client("alice".to_string());
        let mut bob = telephone.add_client("bob".to_string());
        let mut carol = telephone.add_client("carol".to_string());

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
        let bob = telephone.add_client("bob".to_string());
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
        let mut alice = telephone.add_client("alice".to_string());
        telephone.remove_client("alice".to_string());

        let report = telephone.nudge_all(None).await;

        assert!(report.reached.is_empty());
        assert!(report.dropped.is_empty());
        assert_eq!(alice.try_recv().ok(), None);
    }
}
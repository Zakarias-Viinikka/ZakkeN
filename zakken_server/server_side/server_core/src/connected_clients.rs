use error_stuff::ServErr;
use std::collections::HashMap;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub struct ConnectedClients {
    senders: HashMap<String, UnboundedSender<i64>>,
}

impl ConnectedClients {
    pub fn new() -> Self {
        ConnectedClients {
            senders: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, user_name: String) -> UnboundedReceiver<i64> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.senders.insert(user_name, tx);
        rx
    }

    pub fn remove_client(&mut self, user_name: &str) {
        self.senders.remove(user_name);
    }

    pub fn send_to(&self, user_name: &str, letter_id: i64) -> Result<(), ServErr> {
        match self.senders.get(user_name) {
            Some(tx) => tx.send(letter_id).map_err(|_| ServErr::ClientGone {
                user_name: user_name.to_string(),
            }),
            None => Err(ServErr::ClientGone {
                user_name: user_name.to_string(),
            }),
        }
    }

    pub fn connected_user_names(&self) -> Vec<String> {
        self.senders.keys().cloned().collect()
    }

    pub fn is_connected(&self, user_name: &str) -> bool {
        self.senders.contains_key(user_name)
    }
}
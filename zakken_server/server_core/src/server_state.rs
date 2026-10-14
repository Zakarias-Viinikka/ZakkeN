use crate::connected_clients::ConnectedClients;
use db_wrapper::android_mascot::LiveForever;

pub struct ServerState {
    pub liver: LiveForever,
    pub connected: ConnectedClients,
}

impl ServerState {
    pub fn new(liver: LiveForever) -> Self {
        ServerState {
            liver,
            connected: ConnectedClients::new(),
        }
    }
}

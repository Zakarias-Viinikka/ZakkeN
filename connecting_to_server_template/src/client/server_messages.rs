use crate::client_server_coms::{ComsHandleForClient, MessageToClient};
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use leptos::task::spawn_local;
use std::rc::Rc;

pub struct ServerMessageReceiver {
    handle: ComsHandleForClient,
}

impl ServerMessageReceiver {
    fn new(handle: ComsHandleForClient) -> Self {
        ServerMessageReceiver { handle }
    }

    fn handle(&self, msg: MessageToClient) {
        match msg {
            MessageToClient::LoveLetterFromServer(letter) => {
                crate::server_link::receiving::receive_letter(&self.handle, letter);
            }
            MessageToClient::ConnectionLost => {
                crate::server_link::conn_state::go_offline_after_connection_lost();
            }
        }
    }
}

pub fn spawn_receive_loop(
    handle: ComsHandleForClient,
    mut rx: UnboundedReceiver<MessageToClient>,
) {
    let receiver = Rc::new(ServerMessageReceiver::new(handle));
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver.handle(msg);
        }
    });
}

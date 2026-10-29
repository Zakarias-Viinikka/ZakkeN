use futures::channel::{mpsc, oneshot};

pub enum MessageToServer {
    AreYouOnline(oneshot::Sender<bool>),
    WipeYourself,
    GiveMeAllYourData,
    StartOverFromMyData,
}

pub enum MessageToClient {
    AllMyData,
}

#[derive(Clone)]
pub struct ComsHandleForClient {
    tx: mpsc::UnboundedSender<MessageToServer>,
}

impl ComsHandleForClient {
    pub async fn is_server_online(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .unbounded_send(MessageToServer::AreYouOnline(tx))
            .is_err()
        {
            return false;
        }
        rx.await.unwrap()
    }

    pub fn wipe_server(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::WipeYourself);
    }
    pub fn give_me_all_data(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::GiveMeAllYourData);
    }
    pub fn start_over_from_my_data(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::StartOverFromMyData);
    }
}

#[derive(Clone)]
pub struct ComsHandleForServer {
    tx: mpsc::UnboundedSender<MessageToClient>,
}

impl ComsHandleForServer {
    pub fn send_all_my_data(&self) {
        let _ = self.tx.unbounded_send(MessageToClient::AllMyData);
    }
}

pub fn new() -> (
    ComsHandleForClient,
    ComsHandleForServer,
    mpsc::UnboundedReceiver<MessageToClient>,
    mpsc::UnboundedReceiver<MessageToServer>,
) {
    let (client_tx, server_rx) = mpsc::unbounded();
    let (server_tx, client_rx) = mpsc::unbounded();
    (
        ComsHandleForClient { tx: client_tx },
        ComsHandleForServer { tx: server_tx },
        client_rx,
        server_rx,
    )
}

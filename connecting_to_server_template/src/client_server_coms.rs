use futures::channel::{mpsc, oneshot};
use std::fmt;

use crate::protocol::{
    AdoptPage, DatasetGenerationPayload, GetAllPagesPayload, LoveLetter, UnsentLetter,
};

pub enum MessageToServer {
    AreYouOnline(oneshot::Sender<bool>),
    Login {
        user_name: String,
        reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>,
    },
    GetAllPages {
        reply: oneshot::Sender<Result<GetAllPagesPayload, String>>,
    },
    SendLetter {
        letter: LoveLetter,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Ack {
        letter_id: i64,
    },
    GetMyUnsentLetters {
        reply: oneshot::Sender<Result<Vec<UnsentLetter>, String>>,
    },
    WipeDataset {
        reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>,
    },
    AdoptDataset {
        pages: Vec<AdoptPage>,
        reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>,
    },
}

pub enum MessageToClient {
    LoveLetterFromServer(UnsentLetter),
}

#[derive(Clone, Debug)]
pub enum ComsError {
    Transport(String),
    Server(String),
}

impl fmt::Display for ComsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComsError::Transport(s) => write!(f, "transport: {s}"),
            ComsError::Server(s) => write!(f, "server: {s}"),
        }
    }
}

impl std::error::Error for ComsError {}

#[derive(Clone)]
pub struct ComsHandleForClient {
    tx: mpsc::UnboundedSender<MessageToServer>,
}

impl ComsHandleForClient {
    async fn send_request<T, F>(&self, build: F) -> Result<T, ComsError>
    where
        F: FnOnce(oneshot::Sender<Result<T, String>>) -> MessageToServer,
    {
        let (tx, rx) = oneshot::channel();
        let msg = build(tx);
        if self.tx.unbounded_send(msg).is_err() {
            return Err(ComsError::Transport("send failed".into()));
        }
        match rx.await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(ComsError::Server(e)),
            Err(_) => Err(ComsError::Transport("reply dropped".into())),
        }
    }

    pub async fn is_server_online(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .unbounded_send(MessageToServer::AreYouOnline(tx))
            .is_err()
        {
            return false;
        }
        rx.await.unwrap_or(false)
    }

    pub async fn login(
        &self,
        user_name: impl Into<String>,
    ) -> Result<DatasetGenerationPayload, ComsError> {
        let user_name = user_name.into();
        self.send_request(|reply| MessageToServer::Login { user_name, reply })
            .await
    }

    pub async fn get_all_pages(&self) -> Result<GetAllPagesPayload, ComsError> {
        self.send_request(|reply| MessageToServer::GetAllPages { reply })
            .await
    }

    pub async fn send_letter(&self, letter: LoveLetter) -> Result<(), ComsError> {
        self.send_request(|reply| MessageToServer::SendLetter { letter, reply })
            .await
    }

    pub fn ack(&self, letter_id: i64) {
        let _ = self.tx.unbounded_send(MessageToServer::Ack { letter_id });
    }

    pub async fn get_my_unsent_letters(&self) -> Result<Vec<UnsentLetter>, ComsError> {
        self.send_request(|reply| MessageToServer::GetMyUnsentLetters { reply })
            .await
    }

    pub async fn wipe_dataset(&self) -> Result<DatasetGenerationPayload, ComsError> {
        self.send_request(|reply| MessageToServer::WipeDataset { reply })
            .await
    }

    pub async fn adopt_dataset(
        &self,
        pages: Vec<AdoptPage>,
    ) -> Result<DatasetGenerationPayload, ComsError> {
        self.send_request(|reply| MessageToServer::AdoptDataset { pages, reply })
            .await
    }
}

#[derive(Clone)]
pub struct ComsHandleForServer {
    tx: mpsc::UnboundedSender<MessageToClient>,
}

impl ComsHandleForServer {
    pub fn push_letter(&self, letter: UnsentLetter) {
        let _ = self.tx.unbounded_send(MessageToClient::LoveLetterFromServer(letter));
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

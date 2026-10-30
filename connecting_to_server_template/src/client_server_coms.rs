use futures::channel::{mpsc, oneshot};
use std::fmt;

use crate::protocol::{
    AdoptPage, DatasetGenerationPayload, GetAllPagesPayload, LoveLetter, UnsentLetter,
};

pub enum MessageToServer {
    AreYouOnline(oneshot::Sender<bool>),
    Register {
        user_name: String,
        reply: oneshot::Sender<Result<(), String>>,
    },
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
    // STAND-IN: test-bench controls (F1, F4, F6, F7 in the plan).
    TurnOn,
    TurnOff,
    ArmFault,
    PushPretendLetter {
        letter: LoveLetter,
    },
    OtherClientWipes,
    OtherClientReconciled,
    OtherClientStartedOverFromItsVersion,
}

pub enum MessageToClient {
    LoveLetterFromServer(UnsentLetter),
    // STAND-IN: the server replaced its dataset. Distinct from
    // ConnectionLost so the client knows *why* the socket dies: another
    // client wiped, or another client told the server to start over from
    // its own version. The client reacts by going offline + NeedsReconcile
    // + ServerChanged, so after reconnect + re-register the user has to
    // pick reconcile or adopt.
    ServerSaysYouAreNowLonewolf { what_happened: String },
    // STAND-IN: F3. Server signals the socket is gone.
    ConnectionLost,
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

    pub async fn register(&self, user_name: impl Into<String>) -> Result<(), ComsError> {
        let user_name = user_name.into();
        self.send_request(|reply| MessageToServer::Register { user_name, reply })
            .await
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

    pub fn turn_on_server(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::TurnOn);
    }
    pub fn turn_off_server(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::TurnOff);
    }
    pub fn arm_fault(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::ArmFault);
    }
    pub fn push_pretend_letter(&self, letter: LoveLetter) {
        let _ = self.tx.unbounded_send(MessageToServer::PushPretendLetter { letter });
    }
    pub fn other_client_wipes(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::OtherClientWipes);
    }
    pub fn other_client_reconciled(&self) {
        let _ = self.tx.unbounded_send(MessageToServer::OtherClientReconciled);
    }
    pub fn other_client_started_over_from_its_version(&self) {
        let _ = self
            .tx
            .unbounded_send(MessageToServer::OtherClientStartedOverFromItsVersion);
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

    pub fn send_server_says_you_are_now_lonewolf(&self, what_happened: impl Into<String>) {
        let _ = self
            .tx
            .unbounded_send(MessageToClient::ServerSaysYouAreNowLonewolf {
                what_happened: what_happened.into(),
            });
    }

    pub fn send_connection_lost(&self) {
        let _ = self.tx.unbounded_send(MessageToClient::ConnectionLost);
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

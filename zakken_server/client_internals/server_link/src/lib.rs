//! Kotlin-facing face of the server link.
//!
//! Caller pov: Kotlin builds a `ServerLink` with `ServerLink.connect(url,
//! listener)`, then calls the suspend functions on it. Rust owns the wire:
//! JSON framing, the websocket, request ids, matching replies to callers,
//! and bincode-decoding every reply. Kotlin owns the loops (login, send,
//! receive, apply, drift gate).
//!
//! Spontaneous server pushes arrive on `LinkListener`. Its methods are
//! called from a Rust thread and must return quickly: the Kotlin side
//! should hand the item to a channel (`trySend`) and return.
//!
//! Every enum here mirrors one in `server_protocol` / `shared_internals`
//! with an exhaustive `match`, so a new variant on the Rust side stops
//! the Android build instead of drifting silently.

use std::sync::Arc;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{
    AckPayload, DriftSolution, GetAllYrsPagesPayload, GetMyUnsentLettersPayload, LoginPayload, LoginReply,
    RegisterPayload, Request, Response, SendLoveLetterPayload, ServErr,
};
use server_protocol::{GetPageFingerprintsPayload, GetReconcileVersionPayload};
use shared_internals::{ClientCommand, ConnStatus, RouterHandle, build_request};

uniffi::setup_scaffolding!();

// ---------------------------------------------------------------- errors

/// Every failure that crosses into Kotlin carries err_msg, method, file.
///
/// - `Transport`: the socket isn't open or died before the reply came.
///   The row/letter was not confirmed; try again on the next kick.
/// - `Server`: the server answered with an error.
/// - `Local`: something on this side failed (decode, thread, platform).
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LinkError {
    #[error("transport: {err_msg} ({method}, {file})")]
    Transport {
        err_msg: String,
        method: String,
        file: String,
    },
    #[error("server: {err_msg} ({method}, {file})")]
    Server {
        err_msg: String,
        method: String,
        file: String,
    },
    #[error("local: {err_msg} ({method}, {file})")]
    Local {
        err_msg: String,
        method: String,
        file: String,
    },
}

fn transport_err(method: &str, err_msg: String) -> LinkError {
    LinkError::Transport {
        err_msg,
        method: method.to_string(),
        file: file!().to_string(),
    }
}

fn server_err(method: &str, err_msg: String) -> LinkError {
    LinkError::Server {
        err_msg,
        method: method.to_string(),
        file: file!().to_string(),
    }
}

fn local_err(method: &str, err_msg: String) -> LinkError {
    LinkError::Local {
        err_msg,
        method: method.to_string(),
        file: file!().to_string(),
    }
}

// ----------------------------------------------------------------- types

/// Mirrors `shared_internals::ConnStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum LinkStatus {
    Connecting,
    Connected,
    Failed,
}

fn link_status(status: ConnStatus) -> LinkStatus {
    match status {
        ConnStatus::Connecting => LinkStatus::Connecting,
        ConnStatus::Connected => LinkStatus::Connected,
        ConnStatus::Failed => LinkStatus::Failed,
    }
}

/// Mirrors `server_protocol::DriftSolution`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DriftFix {
    WipeSelfAndAdoptServer,
    Reconcile,
}

fn drift_fix(solution: DriftSolution) -> DriftFix {
    match solution {
        DriftSolution::WipeSelfAndAdoptServer => DriftFix::WipeSelfAndAdoptServer,
        DriftSolution::Reconcile => DriftFix::Reconcile,
    }
}

/// Mirrors `server_protocol::LoginReply`.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct LoginInfo {
    pub drift_version: String,
    pub drift_reason: String,
    pub drift_solution: Option<DriftFix>,
}

fn login_info(reply: LoginReply) -> LoginInfo {
    LoginInfo {
        drift_version: reply.drift_version,
        drift_reason: reply.drift_reason,
        drift_solution: reply.drift_solution.map(drift_fix),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum LoginOutcome {
    LoggedIn { info: LoginInfo },
    NoSuchUser,
    Other { err_msg: String },
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum RegisterOutcome {
    Registered,
    AlreadyRegistered,
    Other { err_msg: String },
}

/// One love letter from the server, stamped with the drift version the
/// server was on when it handed it over. The client compares that against
/// its own stored version and drops the letter on mismatch.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct IncomingLetter {
    pub letter_id: i64,
    pub love_letter: Vec<u8>,
    pub drift_version: String,
}

/// One page as the server holds it. Mirrors the fields of
/// `server_protocol::PageRow` that a fresh client needs.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct PageSnapshot {
    pub page_id: String,
    pub blobbed_page: Vec<u8>,
    pub page_status: Vec<u8>,
    pub is_main_menu_page: bool,
}

/// Result of sending a reconcile batch. A drift mismatch is its own case
/// because the caller tells the user to reload and start over.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ReconcileOutcome {
    Applied,
    DriftVersionMismatch,
    Other { err_msg: String },
}

/// Mirrors `server_protocol::PageFingerprint`.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct PageFingerprintInfo {
    pub page_id: String,
    pub fingerprint: String,
}

/// Mirrors `server_protocol::GetPageFingerprintsPayload`.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FingerprintsReply {
    pub drift_version: String,
    pub pages: Vec<PageFingerprintInfo>,
}

// -------------------------------------------------------------- listener

/// Implemented in Kotlin. Called from a Rust thread; return quickly.
#[uniffi::export(with_foreign)]
pub trait LinkListener: Send + Sync {
    fn on_conn_status(&self, status: LinkStatus);
    fn on_love_letter(&self, letter: IncomingLetter);
    fn on_drift_changed(&self, drift_version: String, drift_reason: String);
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
fn forward_spontaneous(
    mut rx_spontaneous: UnboundedReceiver<Response>,
    listener: Arc<dyn LinkListener>,
) {
    futures::executor::block_on(async move {
        while let Some(response) = rx_spontaneous.next().await {
            match response {
                Response::LoveLetterFromServer {
                    letter,
                    drift_version,
                } => listener.on_love_letter(IncomingLetter {
                    letter_id: letter.letter_id,
                    love_letter: letter.love_letter,
                    drift_version,
                }),
                Response::TheDriftVersionChanged {
                    drift_version,
                    drift_reason,
                } => listener.on_drift_changed(drift_version, drift_reason),
                Response::Reply { .. } => {
                    log::warn!("a Reply came down the spontaneous channel; dropping it");
                }
            }
        }
    });
}

// ----------------------------------------------------------- request glue

/// Send a request, decode the bincode `Result<T, ServErr>` reply, and
/// hand the inner result back untouched so the caller can classify the
/// server's error itself (login and register do).
async fn ask_raw<T>(
    router: &RouterHandle,
    request: Request,
    payload: Vec<u8>,
    method: &str,
) -> Result<Result<T, ServErr>, LinkError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let bytes = router
        .send_request(request, payload)
        .await
        .map_err(|e| transport_err(method, format!("{e:?}")))?;
    let outer: Result<Result<T, ServErr>, DbError> = Convert::un_payloadify(&bytes);
    outer.map_err(|e| local_err(method, format!("decode reply: {e:?}")))
}

/// Like `ask_raw`, but a server error becomes `LinkError::Server`.
async fn ask<T>(
    router: &RouterHandle,
    request: Request,
    payload: Vec<u8>,
    method: &str,
) -> Result<T, LinkError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    ask_raw::<T>(router, request, payload, method)
        .await?
        .map_err(|e| server_err(method, format!("{e:?}")))
}

// ------------------------------------------------------------ ServerLink

#[derive(uniffi::Object)]
pub struct ServerLink {
    router: RouterHandle,
}

#[uniffi::export]
impl ServerLink {
    /// Build the router and start the first connect attempt. Returns
    /// immediately; progress arrives through `listener.on_conn_status`.
    /// The url is passed in whole (`ws://host:port/ws`); Rust assumes
    /// nothing about the host.
    #[uniffi::constructor]
    pub fn connect(url: String, listener: Arc<dyn LinkListener>) -> Result<Arc<Self>, LinkError> {
        #[cfg(target_os = "android")]
        {
            use futures::channel::mpsc::unbounded;

            android_logger::init_once(
                android_logger::Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_tag("zakken"),
            );

            let (tx_spontaneous, rx_spontaneous) = unbounded::<Response>();
            let forward_to = Arc::clone(&listener);
            std::thread::Builder::new()
                .name("zakken-spontaneous".to_string())
                .spawn(move || forward_spontaneous(rx_spontaneous, forward_to))
                .map_err(|e| local_err("connect", format!("spawn forwarder thread: {e}")))?;

            let status_to = Arc::clone(&listener);
            let router = shared_internals::Router::new(url, tx_spontaneous, move |status| {
                status_to.on_conn_status(link_status(status));
            });
            router.attempt_reconnect();
            Ok(Arc::new(Self { router }))
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = (url, listener);
            Err(local_err(
                "connect",
                "ServerLink only runs on android".to_string(),
            ))
        }
    }

    /// Ask the socket task to try opening a connection. Safe to call
    /// often; extra signals are coalesced.
    pub fn attempt_reconnect(&self) {
        self.router.attempt_reconnect();
    }

    pub async fn register(&self, user_name: String) -> Result<RegisterOutcome, LinkError> {
        let payload = RegisterPayload { user_name }.to_payload();
        match ask_raw::<()>(&self.router, Request::Register, payload, "register").await? {
            Ok(()) => Ok(RegisterOutcome::Registered),
            Err(ServErr::RejectedBecause(msg)) if msg.contains("already registered") => {
                Ok(RegisterOutcome::AlreadyRegistered)
            }
            Err(e) => Ok(RegisterOutcome::Other {
                err_msg: format!("register failed: {e:?}"),
            }),
        }
    }

    pub async fn login(&self, user_name: String) -> Result<LoginOutcome, LinkError> {
        let payload = LoginPayload { user_name }.to_payload();
        match ask_raw::<LoginReply>(&self.router, Request::Login, payload, "login").await? {
            Ok(reply) => Ok(LoginOutcome::LoggedIn {
                info: login_info(reply),
            }),
            Err(ServErr::DbErrorContainer(DbError::IllegalInput(msg)))
                if msg.contains("no such user") =>
            {
                Ok(LoginOutcome::NoSuchUser)
            }
            Err(e) => Ok(LoginOutcome::Other {
                err_msg: format!("login failed: {e:?}"),
            }),
        }
    }

    /// Send one already-encoded love letter and wait for the server's ok.
    pub async fn send_love_letter(&self, love_letter: Vec<u8>) -> Result<(), LinkError> {
        let payload = SendLoveLetterPayload { love_letter }.to_payload();
        ask::<()>(
            &self.router,
            Request::SendLoveLetter,
            payload,
            "send_love_letter",
        )
        .await
    }

    /// Tell the server this client has received letter `letter_id`.
    pub async fn ack(&self, letter_id: i64) -> Result<(), LinkError> {
        let payload = AckPayload { letter_id }.to_payload();
        ask::<()>(&self.router, Request::Ack, payload, "ack").await
    }

    /// Reconcile ask: the server's drift version plus a fingerprint per page.
    /// Exempt from the drift check, so a lonewolf client may call it.
    pub async fn get_page_fingerprints(&self) -> Result<FingerprintsReply, LinkError> {
        let (request, payload) = build_request(ClientCommand::GetPageFingerprints);
        let reply: GetPageFingerprintsPayload =
            ask(&self.router, request, payload, "get_page_fingerprints").await?;
        Ok(FingerprintsReply {
            drift_version: reply.drift_version,
            pages: reply
                .pages
                .into_iter()
                .map(|p| PageFingerprintInfo {
                    page_id: p.page_id,
                    fingerprint: p.fingerprint,
                })
                .collect(),
        })
    }

    /// Reconcile ask: full data of the named pages. Unknown ids are skipped.
    pub async fn get_pages_by_ids(
        &self,
        page_ids: Vec<String>,
    ) -> Result<Vec<PageSnapshot>, LinkError> {
        let (request, payload) = build_request(ClientCommand::GetPagesByIds(page_ids));
        let reply: GetAllYrsPagesPayload =
            ask(&self.router, request, payload, "get_pages_by_ids").await?;
        Ok(reply
            .pages
            .into_iter()
            .map(|p| PageSnapshot {
                page_id: p.page_id,
                blobbed_page: p.blobbed_page,
                page_status: p.page_status,
                is_main_menu_page: p.is_main_menu_page == "true",
            })
            .collect())
    }

    /// Delete every page and letter on the server. The reply is the new
    /// drift version the server entered; the caller adopts it.
    pub async fn wipe_dataset(&self) -> Result<LoginInfo, LinkError> {
        let (request, payload) = build_request(ClientCommand::WipeDataset);
        let reply: LoginReply = ask(&self.router, request, payload, "wipe_dataset").await?;
        Ok(login_info(reply))
    }

    /// Reconcile ask: the current reconcile version.
    pub async fn get_reconcile_version(&self) -> Result<String, LinkError> {
        let (request, payload) = build_request(ClientCommand::GetReconcileVersion);
        let reply: GetReconcileVersionPayload =
            ask(&self.router, request, payload, "get_reconcile_version").await?;
        Ok(reply.reconcile_version)
    }

    /// Reconcile ask: apply a batch of encoded love letters under the
    /// claimed target versions. All or nothing on the server.
    pub async fn send_reconcile_letters(
        &self,
        target_drift_version: String,
        target_reconcile_version: String,
        letters: Vec<Vec<u8>>,
    ) -> Result<ReconcileOutcome, LinkError> {
        let (request, payload) = build_request(ClientCommand::ReconcileLetters {
            target_drift_version,
            target_reconcile_version,
            letters,
        });
        match ask_raw::<()>(&self.router, request, payload, "send_reconcile_letters").await? {
            Ok(()) => Ok(ReconcileOutcome::Applied),
            Err(ServErr::DriftVersionMismatch) => Ok(ReconcileOutcome::DriftVersionMismatch),
            Err(e) => Ok(ReconcileOutcome::Other {
                err_msg: format!("reconcile batch failed: {e:?}"),
            }),
        }
    }

    /// Every page the server has. Used once by a client with no local pages.
    pub async fn get_all_pages(&self) -> Result<Vec<PageSnapshot>, LinkError> {
        let payload: GetAllYrsPagesPayload = ask(
            &self.router,
            Request::GetAllYrsPages,
            Vec::new(),
            "get_all_pages",
        )
        .await?;
        Ok(payload
            .pages
            .into_iter()
            .map(|p| PageSnapshot {
                page_id: p.page_id,
                blobbed_page: p.blobbed_page,
                page_status: p.page_status,
                is_main_menu_page: p.is_main_menu_page == "true",
            })
            .collect())
    }

    /// Every letter the server still holds for this client, each stamped
    /// with the drift version the server is on right now.
    pub async fn pull_unsent_letters(&self) -> Result<Vec<IncomingLetter>, LinkError> {
        let pulled: GetMyUnsentLettersPayload = ask(
            &self.router,
            Request::GetMyUnsentLetters,
            Vec::new(),
            "pull_unsent_letters",
        )
        .await?;
        let drift_version = pulled.drift_version;
        Ok(pulled
            .letters
            .into_iter()
            .map(|l| IncomingLetter {
                letter_id: l.letter_id,
                love_letter: l.love_letter,
                drift_version: drift_version.clone(),
            })
            .collect())
    }
}

// ----------------------------------------------------------- drift version

/// The drift version stamped onto every outgoing request. Set it after
/// login accepts the server's version.
#[uniffi::export]
pub fn set_drift_version(version: String) {
    shared_internals::set_drift_version(version);
}

#[uniffi::export]
pub fn get_drift_version() -> Option<String> {
    shared_internals::get_drift_version()
}

// ------------------------------------------------------------ fingerprints

/// SHA-256 hex of a page's raw `blobbed_page` bytes. The same helper the
/// server uses, so both sides always agree.
#[uniffi::export]
pub fn fingerprint_of_blob(blob: Vec<u8>) -> String {
    drift_guard_shared::fingerprint(blob.as_slice())
}

/// A fresh drift version for this client to adopt.
#[uniffi::export]
pub fn new_drift_version(user_id: String, time_in_unix: String) -> String {
    drift_guard_shared::new_drift_version(&user_id, &time_in_unix)
}

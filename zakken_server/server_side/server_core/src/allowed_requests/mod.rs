pub mod ack;
pub mod get_all_yrs_pages;
pub mod login;
pub mod register;
pub mod send_love_letter;

use crate::server_state::ServerState;
use connected_clients::ConnectedClients;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::Request;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

pub struct HandlerOutcome {
    pub bytes: Vec<u8>,
    pub just_identified: Option<String>,
    pub incoming_letters: Option<UnboundedReceiver<i64>>,
}

pub fn err_outcome(e: DbError) -> HandlerOutcome {
    let wrapped: Result<(), DbError> = Err(e);
    HandlerOutcome {
        bytes: wrapped.to_payload(),
        just_identified: None,
        incoming_letters: None,
    }
}

pub async fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
    clients: &ConnectedClients,
    sender: Option<&str>,
) -> HandlerOutcome {
    let logged_in = sender.is_some();
    if !logged_in && !matches!(msg_request, Request::Register | Request::Login) {
        // A `Result<T, DbError>::Err` serializes to the same bytes no matter what T
        // is, so the gate can live here without knowing each handler's return type.
        let err: Result<(), DbError> =
            Err(DbError::IllegalInput("not logged in".to_string()));
        return HandlerOutcome {
            bytes: err.to_payload(),
            just_identified: None,
            incoming_letters: None,
        };
    }

    match msg_request {
        // Register, GetAllYrsPages and Ack only touch the db, so they lock
        // briefly, do their work, and let the guard drop at the end of the
        // arm. No await happens while the lock is held.
        Request::Register => {
            let mut locked = state.lock().unwrap();
            let (bytes, just) = register::handle(msg_content, &mut locked);
            HandlerOutcome {
                bytes,
                just_identified: just,
                incoming_letters: None,
            }
        }
        Request::GetAllYrsPages => {
            let mut locked = state.lock().unwrap();
            let (bytes, just) = get_all_yrs_pages::handle(msg_content, &mut locked);
            HandlerOutcome {
                bytes,
                just_identified: just,
                incoming_letters: None,
            }
        }
        Request::Ack => {
            let mut locked = state.lock().unwrap();
            let (bytes, just) = ack::handle(msg_content, &mut locked, sender);
            HandlerOutcome {
                bytes,
                just_identified: just,
                incoming_letters: None,
            }
        }
        // Login and SendLoveLetter do their own locking, because they need
        // to await the connections crate in between db calls.
        Request::Login => login::handle(msg_content, state, clients).await,
        Request::SendLoveLetter => {
            send_love_letter::handle(msg_content, state, clients, sender).await
        }
    }
}
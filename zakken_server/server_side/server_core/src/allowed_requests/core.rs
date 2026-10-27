use crate::server_state::ServerState;
use crate::telephone::Telephone;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::Request;
use std::sync::{Arc, Mutex};

use super::{ack, get_all_yrs_pages, get_my_unsent_letters, login, register, send_love_letter};

pub fn err_bytes(e: DbError) -> Vec<u8> {
    let wrapped: Result<(), DbError> = Err(e);
    wrapped.to_payload()
}

pub async fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
    telephone: &Telephone,
    socket_id: u64,
) -> Vec<u8> {
    let user_name = telephone.user_of(socket_id).await;
    let logged_in = user_name.is_some();

    if !logged_in && !matches!(msg_request, Request::Register | Request::Login) {
        return err_bytes(DbError::IllegalInput("not logged in".to_string()));
    }

    let sender = user_name.as_deref();

    match msg_request {
        Request::Register => {
            let mut locked = state.lock().unwrap();
            register::handle(msg_content, &mut locked)
        }
        Request::GetAllYrsPages => {
            let mut locked = state.lock().unwrap();
            get_all_yrs_pages::handle(msg_content, &mut locked)
        }
        Request::Ack => {
            let mut locked = state.lock().unwrap();
            ack::handle(msg_content, &mut locked, sender)
        }
        Request::GetMyUnsentLetters => {
            let mut locked = state.lock().unwrap();
            get_my_unsent_letters::handle(msg_content, &mut locked, sender)
        }
        Request::Nudge => err_bytes(DbError::IllegalInput(
            "Nudge is server-only; it can't be submitted as a job".to_string(),
        )),
        Request::Login => login::handle(msg_content, state, telephone, socket_id).await,
        Request::SendLoveLetter => {
            send_love_letter::handle(msg_content, state, telephone, sender).await
        }
    }
}
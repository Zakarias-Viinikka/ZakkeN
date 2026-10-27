use crate::telephone::Telephone;
use db_core::db::{operations_love_letters, operations_pages};
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{Request, UnsentLetter};
use std::sync::{Arc, Mutex};

use super::{ack, get_all_yrs_pages, get_my_unsent_letters, login, register, send_love_letter};

pub fn err_bytes(e: DbError) -> Vec<u8> {
    let wrapped: Result<(), DbError> = Err(e);
    wrapped.to_payload()
}

/// Read every letter still rowed for `user_name`, oldest id first. Used
/// by both the login notify (the socket then pulls) and the
/// GetMyUnsentLetters pull.
pub fn unsent_letters_for_user(
    liver: &LiveForever,
    user_name: &str,
) -> Result<Vec<UnsentLetter>, DbError> {
    let ids = operations_love_letters::get_unsent_letter_ids_for_user(liver, user_name)?;
    let mut letters = Vec::with_capacity(ids.len());
    for id in ids {
        let bytes = operations_love_letters::get_letter(liver, id)?;
        letters.push(UnsentLetter {
            letter_id: id,
            love_letter: bytes,
        });
    }
    Ok(letters)
}

pub async fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &Arc<Mutex<LiveForever>>,
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
            let locked = state.lock().unwrap();
            register::register_user(msg_content, &locked)
        }
        Request::GetAllYrsPages => {
            let locked = state.lock().unwrap();
            get_all_yrs_pages::send_all_pages(msg_content, &locked)
        }
        Request::Ack => {
            let locked = state.lock().unwrap();
            ack::acknowledge_letter(msg_content, &locked, sender)
        }
        Request::GetMyUnsentLetters => {
            let locked = state.lock().unwrap();
            get_my_unsent_letters::send_unsent_letters(msg_content, &locked, sender)
        }
        Request::Login => login::log_in(msg_content, state, telephone, socket_id).await,
        Request::SendLoveLetter => {
            send_love_letter::accept_letter(msg_content, state, telephone, sender).await
        }
    }
}

pub use operations_pages::StoredPage;
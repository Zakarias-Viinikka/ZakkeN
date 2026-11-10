use crate::telephone::Telephone;
use db_core::db::{db_actions_for_drift, operations_love_letters, operations_pages};
use error_stuff::{ErrorDetails, ServErr};
use queue::WhereTheRequestCameFrom;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{Request, UnsentLetter};
use std::sync::{Arc, Mutex};

use super::{
    ack, get_all_yrs_pages, get_current_drift, get_my_unsent_letters, login, register,
    send_love_letter, wipe_dataset,
};

pub fn err_bytes(e: ServErr) -> Vec<u8> {
    let wrapped: Result<(), ServErr> = Err(e);
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

/// Register and login are how a client learns the current drift version,
/// so they can't be checked against one. A request the socket submits on
/// its own behalf (the internal pull) has no client and no version to
/// check. Everything else must match or be refused.
fn refuse_if_the_client_is_on_a_stale_drift_version(
    msg_request: &Request,
    where_the_request_came_from: &WhereTheRequestCameFrom,
    state: &Arc<Mutex<LiveForever>>,
) -> Option<ServErr> {
    if matches!(
        msg_request,
        Request::Register | Request::Login | Request::GetCurrentDrift
    ) {
        return None;
    }
    let WhereTheRequestCameFrom::FromAClient(client_version) = where_the_request_came_from else {
        return None;
    };
    let current = {
        let db_wrapper = state.lock().unwrap();
        match db_actions_for_drift::get_current(&db_wrapper) {
            Ok(Some(dv)) => dv.version,
            Ok(None) => {
                return Some(ServErr::ServerError(ErrorDetails {
                    err_msg: "server has no drift_version row".to_string(),
                    file: file!().to_string(),
                    method: "core::refuse_if_the_client_is_on_a_stale_drift_version".to_string(),
                    probable_cause: "ensure_minted didn't run at startup".to_string(),
                }));
            }
            Err(e) => return Some(ServErr::DbErrorContainer(e)),
        }
    };
    if client_version != &current {
        return Some(ServErr::DriftVersionMismatch);
    }
    None
}

pub async fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &Arc<Mutex<LiveForever>>,
    telephone: &Telephone,
    socket_id: u64,
    where_the_request_came_from: WhereTheRequestCameFrom,
) -> Vec<u8> {
    let user_name = telephone.user_of(socket_id).await;
    let logged_in = user_name.is_some();

    if !logged_in && !matches!(msg_request, Request::Register | Request::Login) {
        return err_bytes(ServErr::RejectedBecause("not logged in".to_string()));
    }

    if let Some(e) =
        refuse_if_the_client_is_on_a_stale_drift_version(msg_request, &where_the_request_came_from, state)
    {
        return err_bytes(e);
    }

    let sender = user_name.as_deref();

    match msg_request {
        Request::Register => {
            let db_wrapper = state.lock().unwrap();
            register::register_user(msg_content, &db_wrapper)
        }
        Request::GetAllYrsPages => {
            let db_wrapper = state.lock().unwrap();
            get_all_yrs_pages::send_all_pages(msg_content, &db_wrapper)
        }
        Request::Ack => {
            let db_wrapper = state.lock().unwrap();
            ack::acknowledge_letter(msg_content, &db_wrapper, sender)
        }
        Request::GetMyUnsentLetters => {
            let db_wrapper = state.lock().unwrap();
            get_my_unsent_letters::send_unsent_letters(msg_content, &db_wrapper, sender)
        }
        Request::Login => login::log_in(msg_content, state, telephone, socket_id).await,
        Request::SendLoveLetter => {
            send_love_letter::accept_letter(msg_content, state, telephone, sender).await
        }
        Request::WipeDataset => {
            wipe_dataset::wipe_dataset(msg_content, state, telephone, socket_id).await
        }
        Request::GetCurrentDrift => {
            let locked = state.lock().unwrap();
            get_current_drift::get_current_drift(msg_content, &locked)
        }
    }
}

pub use operations_pages::StoredPage;
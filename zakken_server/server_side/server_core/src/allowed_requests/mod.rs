pub mod ack;
pub mod get_all_yrs_pages;
pub mod login;
pub mod register;
pub mod send_love_letter;

use crate::server_state::ServerState;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::Request;

pub fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &mut ServerState,
    sender: Option<&str>,
) -> (Vec<u8>, Option<String>) {
    let logged_in = sender.is_some();
    if !logged_in && !matches!(msg_request, Request::Register | Request::Login) {
        // A `Result<T, DbError>::Err` serializes to the same bytes no matter what T
        // is, so the gate can live here without knowing each handler's return type.
        let err: Result<(), DbError> =
            Err(DbError::IllegalInput("not logged in".to_string()));
        return (err.to_payload(), None);
    }

    match msg_request {
        Request::Register => register::handle(msg_content, state),
        Request::Login => login::handle(msg_content, state),
        Request::SendLoveLetter => send_love_letter::handle(msg_content, state, sender),
        Request::GetAllYrsPages => get_all_yrs_pages::handle(msg_content, state),
        Request::Ack => ack::handle(msg_content, state, sender),
    }
}
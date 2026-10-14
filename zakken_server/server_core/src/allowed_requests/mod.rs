pub mod register;
pub mod send_love_letter;

use crate::server_state::ServerState;
use server_protocol::Request;

pub fn do_request(
    msg_request: &Request,
    msg_content: Vec<u8>,
    state: &mut ServerState,
) -> (Vec<u8>, Option<String>) {
    match msg_request {
        Request::Register => register::handle(msg_content, state),
        Request::SendLoveLetter => send_love_letter::handle(msg_content, state),
    }
}
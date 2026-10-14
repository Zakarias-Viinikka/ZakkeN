use crate::server_state::ServerState;
use db_core::db::user_registration;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::RegisterPayload;

pub fn handle(msg_content: Vec<u8>, state: &mut ServerState) -> (Vec<u8>, Option<String>) {
    let payload = match RegisterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => return (e.to_payload(), None),
    };

    match user_registration::register_user(&state.liver, &payload.user_name) {
        Ok(()) => (ok_serialized(), Some(payload.user_name)),
        Err(e) => (e.to_payload(), None),
    }
}
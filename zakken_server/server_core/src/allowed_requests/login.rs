use crate::server_state::ServerState;
use db_core::db::user_registration;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::LoginPayload;

pub fn handle(msg_content: Vec<u8>, state: &mut ServerState) -> (Vec<u8>, Option<String>) {
    let payload = match LoginPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(DbError::BadCode(format!(
                "couldn't read login payload: {e:?}"
            )));
            return (wrapped.to_payload(), None);
        }
    };

    match user_registration::user_exists(&state.liver, &payload.user_name) {
        Ok(true) => (ok_serialized(), Some(payload.user_name)),
        Ok(false) => {
            let wrapped: Result<(), DbError> = Err(DbError::IllegalInput(format!(
                "no such user: '{}'",
                payload.user_name
            )));
            (wrapped.to_payload(), None)
        }
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(e);
            (wrapped.to_payload(), None)
        }
    }
}
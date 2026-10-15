use crate::server_state::ServerState;
use db_core::db::user_registration;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::RegisterPayload;

pub fn handle(msg_content: Vec<u8>, state: &mut ServerState) -> (Vec<u8>, Option<String>) {
    let payload = match RegisterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(DbError::BadCode(format!(
                "couldn't read register payload: {e:?}"
            )));
            return (wrapped.to_payload(), None);
        }
    };

    match user_registration::create_user(&state.liver, &payload.user_name) {
        Ok(()) => (ok_serialized(), None),
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(e);
            (wrapped.to_payload(), None)
        }
    }
}
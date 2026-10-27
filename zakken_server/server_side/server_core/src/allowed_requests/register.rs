use db_core::db::user_registration;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::RegisterPayload;

pub fn register_user(msg_content: Vec<u8>, state: &LiveForever) -> Vec<u8> {
    let payload = match RegisterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(DbError::BadCode(format!(
                "couldn't read register payload: {e:?}"
            )));
            return wrapped.to_payload();
        }
    };

    match user_registration::create_user(state, &payload.user_name) {
        Ok(()) => ok_serialized(),
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(e);
            wrapped.to_payload()
        }
    }
}
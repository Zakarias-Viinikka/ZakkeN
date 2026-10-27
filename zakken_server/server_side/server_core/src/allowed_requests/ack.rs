use db_core::db::operations_love_letters;
use db_wrapper::android_mascot::LiveForever;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::AckPayload;

pub fn acknowledge_letter(
    msg_content: Vec<u8>,
    state: &LiveForever,
    sender: Option<&str>,
) -> Vec<u8> {
    let payload = match AckPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(DbError::BadCode(format!(
                "couldn't read ack payload: {e:?}"
            )));
            return wrapped.to_payload();
        }
    };

    let Some(user_name) = sender else {
        let wrapped: Result<(), DbError> = Err(DbError::IllegalInput(
            "ack without a logged-in user".to_string(),
        ));
        return wrapped.to_payload();
    };

    match operations_love_letters::delete_unsent_row(state, user_name, payload.letter_id) {
        Ok(()) => ok_serialized(),
        Err(e) => {
            let wrapped: Result<(), DbError> = Err(e);
            wrapped.to_payload()
        }
    }
}
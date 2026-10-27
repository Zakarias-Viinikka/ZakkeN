use crate::server_state::ServerState;
use db_core::db::operations_love_letters;
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{GetMyUnsentLettersPayload, UnsentLetter};

pub fn handle(
    _msg_content: Vec<u8>,
    state: &mut ServerState,
    sender: Option<&str>,
) -> Vec<u8> {
    let Some(user_name) = sender else {
        let wrapped: Result<GetMyUnsentLettersPayload, DbError> =
            Err(DbError::IllegalInput(
                "GetMyUnsentLetters without a logged-in user".to_string(),
            ));
        return wrapped.to_payload();
    };

    let ids =
        match operations_love_letters::get_unsent_letter_ids_for_user(&state.liver, user_name) {
            Ok(ids) => ids,
            Err(e) => {
                let wrapped: Result<GetMyUnsentLettersPayload, DbError> = Err(e);
                return wrapped.to_payload();
            }
        };

    let mut letters = Vec::new();
    for id in ids {
        match operations_love_letters::get_letter(&state.liver, id) {
            Ok(bytes) => letters.push(UnsentLetter {
                letter_id: id,
                love_letter: bytes,
            }),
            Err(e) => {
                let wrapped: Result<GetMyUnsentLettersPayload, DbError> = Err(e);
                return wrapped.to_payload();
            }
        }
    }

    let wrapped: Result<GetMyUnsentLettersPayload, DbError> =
        Ok(GetMyUnsentLettersPayload { letters });
    wrapped.to_payload()
}
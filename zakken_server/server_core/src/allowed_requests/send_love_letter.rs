use crate::server_state::ServerState;
use db_core::db::operations_love_letters;
use love_letter_parser::get_target_page_id;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::SendLoveLetterPayload;

pub fn handle(msg_content: Vec<u8>, state: &mut ServerState) -> (Vec<u8>, Option<String>) {
    let payload = match SendLoveLetterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => return (e.to_payload(), None),
    };

    let target_page = match get_target_page_id(payload.love_letter.clone()) {
        Ok(t) => t,
        Err(e) => {
            return (
                protocol::error::DbError::BadCode(e.to_string()).to_payload(),
                None,
            );
        }
    };

    let letter_id = match operations_love_letters::insert_letter(
        &state.liver,
        payload.love_letter,
        target_page,
    ) {
        Ok(id) => id,
        Err(e) => return (e.to_payload(), None),
    };

    let users = match operations_love_letters::get_all_registered_user_names(&state.liver) {
        Ok(u) => u,
        Err(e) => return (e.to_payload(), None),
    };

    for user_name in users {
        if state.connected.is_connected(&user_name) {
            match state.connected.send_to(&user_name, letter_id) {
                Ok(()) => {
                    if let Err(e) = operations_love_letters::delete_unsent_row(
                        &state.liver,
                        &user_name,
                        letter_id,
                    ) {
                        return (e.to_payload(), None);
                    }
                }
                Err(e) => {
                    state.connected.remove_client(&user_name);
                    return (error_stuff::ServErr::from(e).to_payload(), None);
                }
            }
        } else if let Err(e) =
            operations_love_letters::queue_unsent_for_user(&state.liver, &user_name, letter_id)
        {
            return (e.to_payload(), None);
        }
    }

    (ok_serialized(), None)
}
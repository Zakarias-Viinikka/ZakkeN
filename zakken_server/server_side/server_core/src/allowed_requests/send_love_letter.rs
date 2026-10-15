use crate::logging::log;
use crate::server_state::ServerState;
use db_core::db::apply_letter::apply_letter;
use db_core::db::operations_love_letters;
use love_letter::deserialize_love_letter;
use love_letter_parser::get_target_page_id_from_letter;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::SendLoveLetterPayload;

fn err_to_bytes(e: DbError) -> Vec<u8> {
    let wrapped: Result<(), DbError> = Err(e);
    wrapped.to_payload()
}

pub fn handle(
    msg_content: Vec<u8>,
    state: &mut ServerState,
    sender: Option<&str>,
) -> (Vec<u8>, Option<String>) {
    let payload = match SendLoveLetterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            log("server", format!("couldn't read the letter: {e:?}"));
            return (err_to_bytes(e), None);
        }
    };

    let letter = match deserialize_love_letter(payload.love_letter.clone()) {
        Ok(l) => l,
        Err(e) => {
            log("server", format!("couldn't decode the letter: {e}"));
            return (err_to_bytes(DbError::BadCode(e.to_string())), None);
        }
    };

    let target_page = get_target_page_id_from_letter(&letter);

    if let Err(e) = apply_letter(&state.liver, letter) {
        log("server", format!("couldn't apply the letter: {e:?}"));
        return (err_to_bytes(e), None);
    }
    log(
        "server",
        format!("updated its copy of page {target_page}"),
    );

    let letter_id = match operations_love_letters::insert_letter(
        &state.liver,
        payload.love_letter,
        target_page,
    ) {
        Ok(id) => id,
        Err(e) => {
            log("server", format!("couldn't save the letter: {e:?}"));
            return (err_to_bytes(e), None);
        }
    };
    log("server", format!("saved the letter ({letter_id})"));

    let users = match operations_love_letters::get_all_registered_user_names(&state.liver) {
        Ok(u) => u,
        Err(e) => {
            log("server", format!("couldn't read the user list: {e:?}"));
            return (err_to_bytes(e), None);
        }
    };

    let mut online = 0usize;
    let mut offline = 0usize;

    for user_name in users {
        if Some(user_name.as_str()) == sender {
            continue;
        }

        if state.connected.is_connected(&user_name) {
            match state.connected.send_to(&user_name, letter_id) {
                Ok(()) => {
                    if let Err(e) = operations_love_letters::delete_unsent_row(
                        &state.liver,
                        &user_name,
                        letter_id,
                    ) {
                        log(
                            "server",
                            format!("couldn't clear the queue for {user_name}: {e:?}"),
                        );
                        return (err_to_bytes(e), None);
                    }
                    online += 1;
                }
                Err(e) => {
                    state.connected.remove_client(&user_name);
                    log(
                        "server",
                        format!("lost contact with {user_name}, dropped them: {e:?}"),
                    );
                    return (
                        err_to_bytes(DbError::BadCode(format!(
                            "lost contact with {user_name}: {e:?}"
                        ))),
                        None,
                    );
                }
            }
        } else if let Err(e) =
            operations_love_letters::queue_unsent_for_user(&state.liver, &user_name, letter_id)
        {
            log(
                "server",
                format!("couldn't queue the letter for {user_name}: {e:?}"),
            );
            return (err_to_bytes(e), None);
        } else {
            offline += 1;
        }
    }

    if online == 0 && offline == 0 {
        log("server", "no one else needs this letter");
    } else {
        log(
            "server",
            format!("will reach {online} now, {offline} when they reconnect"),
        );
    }

    (ok_serialized(), None)
}
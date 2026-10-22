use crate::allowed_requests::{err_outcome, HandlerOutcome};
use crate::logging::{log, log_error};
use crate::server_state::ServerState;
use connected_clients::Telephone;
use db_core::db::apply_letter::apply_letter;
use db_core::db::operations_love_letters;
use error_stuff::{ErrorDetails, LogCategory};
use love_letter::deserialize_love_letter;
use love_letter_parser::get_target_page_id_from_letter;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::SendLoveLetterPayload;
use std::sync::{Arc, Mutex};

pub async fn handle(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
    telephone: &Telephone,
    sender: Option<&str>,
) -> HandlerOutcome {
    let payload = match SendLoveLetterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't read the letter: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::handle".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_outcome(e);
        }
    };

    let letter = match deserialize_love_letter(payload.love_letter.clone()) {
        Ok(l) => l,
        Err(e) => {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't decode the letter: {e}"),
                    file: file!().to_string(),
                    method: "send_love_letter::handle".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_outcome(DbError::BadCode(e.to_string()));
        }
    };

    let target_page = get_target_page_id_from_letter(&letter);

    // Every db write for this letter happens inside one lock: apply it,
    // store it, read the user list, and queue an unsent row for every
    // registered user except the sender. The lock is dropped before the
    // nudge, which is a call into the telephone crate.
    let letter_id = {
        let locked = state.lock().unwrap();

        if let Err(e) = apply_letter(&locked.liver, letter) {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't apply the letter: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::handle".to_string(),
                    probable_cause: "the page the letter targets couldn't be read or written".to_string(),
                },
            );
            return err_outcome(e);
        }
        log("server", format!("updated its copy of page {target_page}"));

        let letter_id = match operations_love_letters::insert_letter(
            &locked.liver,
            payload.love_letter,
            target_page,
        ) {
            Ok(id) => id,
            Err(e) => {
                log_error(
                    "server",
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("couldn't save the letter: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::handle".to_string(),
                        probable_cause: "the love_letters_from_clients table rejected the insert".to_string(),
                    },
                );
                return err_outcome(e);
            }
        };
        log("server", format!("saved the letter ({letter_id})"));

        let users = match operations_love_letters::get_all_registered_user_names(&locked.liver) {
            Ok(u) => u,
            Err(e) => {
                log_error(
                    "server",
                    LogCategory::Database,
                    &ErrorDetails {
                        err_msg: format!("couldn't read the user list: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::handle".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return err_outcome(e);
            }
        };

        for user_name in users {
            if Some(user_name.as_str()) == sender {
                continue;
            }
            // every registered user gets a row, online or not. the row is
            // deleted when that user acks the letter. if their connection
            // dies before the ack, they get it again on reconnect and dedup
            // it themselves by letter_id.
            if let Err(e) =
                operations_love_letters::queue_unsent_for_user(&locked.liver, &user_name, letter_id)
            {
                log_error(
                    "server",
                    LogCategory::Database,
                    &ErrorDetails {
                        err_msg: format!("couldn't queue the letter for {user_name}: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::handle".to_string(),
                        probable_cause: "the unsent_love_letters table rejected the row".to_string(),
                    },
                );
                return err_outcome(e);
            }
        }

        letter_id
    };
    // lock dropped

    let report = telephone
        .nudge_all(sender.map(|s| s.to_string()))
        .await;

    for name in &report.dropped {
        log_error(
            "server",
            LogCategory::Connection,
            &ErrorDetails {
                err_msg: format!("lost contact with {name} while nudging for letter {letter_id}"),
                file: file!().to_string(),
                method: "send_love_letter::handle".to_string(),
                probable_cause: "the client disconnected mid-nudge; their unsent row will reach them".to_string(),
            },
        );
    }

    let online = report.reached.len();
    if online == 0 {
        log("server", "no one else is online right now");
    } else {
        log(
            "server",
            format!("letter {letter_id}: nudged {online} client(s)"),
        );
    }

    HandlerOutcome {
        bytes: ok_serialized(),
        just_identified: None,
        incoming_letters: None,
    }
}
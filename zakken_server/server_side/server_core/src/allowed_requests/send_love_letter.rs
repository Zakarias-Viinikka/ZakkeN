use crate::logging::{log, log_error};
use crate::telephone::Telephone;
use db_core::db::apply_letter::apply_letter;
use db_core::db::operations_love_letters;
use db_wrapper::android_mascot::LiveForever;
use error_stuff::{ErrorDetails, LogCategory};
use love_letter::deserialize_love_letter;
use love_letter_parser::get_target_page_id_from_letter;
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::SendLoveLetterPayload;
use std::sync::{Arc, Mutex};

use super::core::err_bytes;

pub async fn accept_letter(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<LiveForever>>,
    telephone: &Telephone,
    sender: Option<&str>,
) -> Vec<u8> {
    let payload = match SendLoveLetterPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't read the letter: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::accept_letter".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(e);
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
                    method: "send_love_letter::accept_letter".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(DbError::BadCode(e.to_string()));
        }
    };

    let target_page = get_target_page_id_from_letter(&letter);

    // One transaction across the whole handler. If any step fails, the
    // pages update, the letter row, and the unsent rows all roll back
    // together, so a client retry starts clean instead of leaving a
    // duplicate letter id and a stray unsent row per user.
    let letter_id = {
        let locked = state.lock().unwrap();

        if let Err(e) = locked.begin_all_or_nothing() {
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't begin transaction: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::accept_letter".to_string(),
                    probable_cause: "another transaction is already open".to_string(),
                },
            );
            return err_bytes(e);
        }

        if let Err(e) = apply_letter(&locked, letter) {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't apply the letter: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::accept_letter".to_string(),
                    probable_cause: "the page the letter targets couldn't be read or written".to_string(),
                },
            );
            let _ = locked.regret_everything();
            return err_bytes(e);
        }
        log("server", format!("updated its copy of page {target_page}"));

        let id = match operations_love_letters::insert_letter(&locked, payload.love_letter, target_page) {
            Ok(id) => id,
            Err(e) => {
                log_error(
                    "server",
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("couldn't save the letter: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::accept_letter".to_string(),
                        probable_cause: "the love_letters_from_clients table rejected the insert".to_string(),
                    },
                );
                let _ = locked.regret_everything();
                return err_bytes(e);
            }
        };
        log("server", format!("saved the letter ({id})"));

        // Queue unsent rows for every registered user except the sender,
        // so login-notify and GetMyUnsentLetters have something to point
        // at, and acks have a row to delete.
        let users = match operations_love_letters::get_all_registered_user_names(&locked) {
            Ok(u) => u,
            Err(e) => {
                log_error(
                    "server",
                    LogCategory::Database,
                    &ErrorDetails {
                        err_msg: format!("couldn't read the user list: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::accept_letter".to_string(),
                        probable_cause: String::new(),
                    },
                );
                let _ = locked.regret_everything();
                return err_bytes(e);
            }
        };

        for user_name in users {
            if Some(user_name.as_str()) == sender {
                continue;
            }
            if let Err(e) = operations_love_letters::queue_unsent_for_user(&locked, &user_name, id) {
                log_error(
                    "server",
                    LogCategory::Database,
                    &ErrorDetails {
                        err_msg: format!("couldn't queue the letter for {user_name}: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::accept_letter".to_string(),
                        probable_cause: "the unsent_love_letters table rejected the row".to_string(),
                    },
                );
                let _ = locked.regret_everything();
                return err_bytes(e);
            }
        }

        if let Err(e) = locked.everything_went_perfectly() {
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't commit the transaction: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::accept_letter".to_string(),
                    probable_cause: "sqlite rejected the commit".to_string(),
                },
            );
            return err_bytes(e);
        }

        id
    };

    let report = telephone
        .notify_all(sender.map(|s| s.to_string()))
        .await;

    for name in &report.dropped {
        log_error(
            "server",
            LogCategory::Connection,
            &ErrorDetails {
                err_msg: format!("lost contact with {name} while notifying for letter {letter_id}"),
                file: file!().to_string(),
                method: "send_love_letter::accept_letter".to_string(),
                probable_cause: "the client disconnected mid-notify; their unsent row will reach them".to_string(),
            },
        );
    }

    let online = report.reached.len();
    if online == 0 {
        log("server", "no one else is online right now");
    } else {
        log(
            "server",
            format!("letter {letter_id}: notified {online} client(s)"),
        );
    }

    ok_serialized()
}
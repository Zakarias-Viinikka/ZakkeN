use crate::logging::{log, log_error};
use crate::telephone::Telephone;
use db_core::db::apply_letter::apply_letter;
use db_core::db::operations_love_letters;
use db_wrapper::android_mascot::LiveForever;
use love_letter::{deserialize_love_letter, LoveLetter};
use love_letter_parser::get_target_page_id_from_letter;

use protocol::serialization::{ok_serialized, Convert};
use server_errors::{ErrorDetails, LogCategory, ServErr};
use server_protocol::SendLoveLetterPayload;
use std::sync::{Arc, Mutex};

use super::core::err_bytes;

/// Apply, insert, and queue one already-decoded letter. No transaction
/// of its own — callers are responsible for one. Returns the new
/// letter_id. Does NOT notify.
pub(crate) fn apply_one_letter(
    db_wrapper: &LiveForever,
    letter: LoveLetter,
    raw_bytes: Vec<u8>,
    sender: Option<&str>,
) -> Result<i64, ServErr> {
    let target_page = get_target_page_id_from_letter(&letter);
    apply_letter(db_wrapper, letter).map_err(ServErr::DbErrorContainer)?;
    let letter_id = operations_love_letters::insert_letter(db_wrapper, raw_bytes, target_page)
        .map_err(ServErr::DbErrorContainer)?;
    queue_unsent_for_everyone_but_sender(db_wrapper, letter_id, sender)?;
    Ok(letter_id)
}

fn queue_unsent_for_everyone_but_sender(
    db_wrapper: &LiveForever,
    letter_id: i64,
    sender: Option<&str>,
) -> Result<(), ServErr> {
    let users = operations_love_letters::get_all_registered_user_names(db_wrapper)
        .map_err(ServErr::DbErrorContainer)?;
    for user_name in users {
        if Some(user_name.as_str()) == sender {
            continue;
        }
        operations_love_letters::queue_unsent_for_user(db_wrapper, &user_name, letter_id)
            .map_err(ServErr::DbErrorContainer)?;
    }
    Ok(())
}

/// Entry point for `Request::SendLoveLetter`.
pub async fn main_for_accepting_a_letter(
    msg_content: Vec<u8>,
    db_wrapper: &Arc<Mutex<LiveForever>>,
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
                    method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(ServErr::DbErrorContainer(e));
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
                    method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(ServErr::BadCode(e.to_string()));
        }
    };

    let letter_id = {
        let db_wrapper = db_wrapper.lock().unwrap();

        if let Err(e) = db_wrapper.begin_all_or_nothing() {
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't begin transaction: {e:?}"),
                    file: file!().to_string(),
                    method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                    probable_cause: "another transaction is already open".to_string(),
                },
            );
            return err_bytes(ServErr::DbErrorContainer(e));
        }

        match apply_one_letter(&db_wrapper, letter, payload.love_letter, sender) {
            Ok(id) => match db_wrapper.everything_went_perfectly() {
                Ok(()) => id,
                Err(e) => {
                    log_error(
                        "server",
                        LogCategory::Database,
                        &ErrorDetails {
                            err_msg: format!("couldn't commit the transaction: {e:?}"),
                            file: file!().to_string(),
                            method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                            probable_cause: "sqlite rejected the commit".to_string(),
                        },
                    );
                    return err_bytes(ServErr::DbErrorContainer(e));
                }
            },
            Err(e) => {
                let _ = db_wrapper.regret_everything();
                log_error(
                    "server",
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("letter apply failed: {e:?}"),
                        file: file!().to_string(),
                        method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return err_bytes(e);
            }
        }
    };

    let report = telephone.notify_all(sender.map(|s| s.to_string())).await;

    for name in &report.dropped {
        log_error(
            "server",
            LogCategory::Connection,
            &ErrorDetails {
                err_msg: format!("lost contact with {name} while notifying for letter {letter_id}"),
                file: file!().to_string(),
                method: "send_love_letter::main_for_accepting_a_letter".to_string(),
                probable_cause:
                    "the client disconnected mid-notify; their unsent row will reach them"
                        .to_string(),
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
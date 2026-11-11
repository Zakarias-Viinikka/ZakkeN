use std::sync::{Arc, Mutex};

use db_core::db::{db_actions_for_drift, operations_key_value};
use db_wrapper::android_mascot::LiveForever;
use love_letter::{deserialize_love_letter, LoveLetter};
use protocol::serialization::{ok_serialized, Convert};
use server_errors::{ErrorDetails, LogCategory, ServErr};
use server_protocol::ReconcileLettersPayload;

use crate::logging::{log, log_error};
use crate::telephone::Telephone;

use super::core::err_bytes;
use super::send_love_letter::apply_one_letter;

/// Entry point for `Request::ReconcileLetters`. Decodes the whole batch
/// before touching the DB, then applies every letter inside ONE outer
/// transaction, then notifies once. Does not touch the drift version.
pub async fn reconcile_letters(
    msg_content: Vec<u8>,
    db_wrapper: &Arc<Mutex<LiveForever>>,
    telephone: &Telephone,
    sender: Option<&str>,
) -> Vec<u8> {
    let payload = match read_payload(msg_content) {
        Ok(p) => p,
        Err(e) => {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't read reconcile payload: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(e);
        }
    };

    let decoded = match decode_batch(payload.letters) {
        Ok(d) => d,
        Err(e) => {
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("couldn't decode one of the batch's letters: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(e);
        }
    };

    let letter_count = decoded.len();
    let target_drift_version = payload.target_drift_version;
    let target_reconcile_version = payload.target_reconcile_version;

    {
        let db_wrapper = db_wrapper.lock().unwrap();

        if let Err(e) = db_wrapper.begin_all_or_nothing() {
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't begin transaction: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: "another transaction is already open".to_string(),
                },
            );
            return err_bytes(ServErr::DbErrorContainer(e));
        }

        if let Err(e) = check_target_drift_version(&db_wrapper, &target_drift_version) {
            let _ = db_wrapper.regret_everything();
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("drift version check failed: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: String::new(),
                },
            );
            return err_bytes(e);
        }

        if let Err(e) = check_target_reconcile_version(&db_wrapper, &target_reconcile_version) {
            let _ = db_wrapper.regret_everything();
            log_error(
                "server",
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("reconcile version check failed: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause:
                        "another reconciler already bumped the reconcile version".to_string(),
                },
            );
            return err_bytes(e);
        }

        for (letter, bytes) in decoded {
            if let Err(e) = apply_one_letter(&db_wrapper, letter, bytes, sender) {
                let _ = db_wrapper.regret_everything();
                log_error(
                    "server",
                    LogCategory::LetterDelivery,
                    &ErrorDetails {
                        err_msg: format!("reconcile batch failed: {e:?}"),
                        file: file!().to_string(),
                        method: "reconcile_letters::reconcile_letters".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return err_bytes(e);
            }
        }

        if let Err(e) = operations_key_value::bump_reconcile_version(&db_wrapper) {
            let _ = db_wrapper.regret_everything();
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't bump reconcile version: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: "key_value_storage write failed".to_string(),
                },
            );
            return err_bytes(ServErr::DbErrorContainer(e));
        }

        if let Err(e) = db_wrapper.everything_went_perfectly() {
            log_error(
                "server",
                LogCategory::Database,
                &ErrorDetails {
                    err_msg: format!("couldn't commit the transaction: {e:?}"),
                    file: file!().to_string(),
                    method: "reconcile_letters::reconcile_letters".to_string(),
                    probable_cause: "sqlite rejected the commit".to_string(),
                },
            );
            return err_bytes(ServErr::DbErrorContainer(e));
        }
    }

    notify_after_batch(telephone, sender, letter_count).await;
    ok_serialized()
}

fn read_payload(msg_content: Vec<u8>) -> Result<ReconcileLettersPayload, ServErr> {
    Convert::un_payloadify(&msg_content).map_err(ServErr::DbErrorContainer)
}

fn decode_batch(raw: Vec<Vec<u8>>) -> Result<Vec<(LoveLetter, Vec<u8>)>, ServErr> {
    let mut decoded = Vec::with_capacity(raw.len());
    for bytes in raw {
        match deserialize_love_letter(bytes.clone()) {
            Ok(l) => decoded.push((l, bytes)),
            Err(e) => return Err(ServErr::BadCode(e.to_string())),
        }
    }
    Ok(decoded)
}

fn check_target_drift_version(
    db_wrapper: &LiveForever,
    target: &str,
) -> Result<(), ServErr> {
    match db_actions_for_drift::get_current(db_wrapper)? {
        Some(dv) if dv.version == target => Ok(()),
        _ => Err(ServErr::DriftVersionMismatch),
    }
}

fn check_target_reconcile_version(
    db_wrapper: &LiveForever,
    target: &str,
) -> Result<(), ServErr> {
    let current = operations_key_value::get_reconcile_version(db_wrapper)?;
    if current == target {
        Ok(())
    } else {
        Err(ServErr::DriftVersionMismatch)
    }
}

async fn notify_after_batch(telephone: &Telephone, sender: Option<&str>, letter_count: usize) {
    let report = telephone.notify_all(sender.map(|s| s.to_string())).await;
    for name in &report.dropped {
        log_error(
            "server",
            LogCategory::Connection,
            &ErrorDetails {
                err_msg: format!("lost contact with {name} while notifying for reconcile"),
                file: file!().to_string(),
                method: "reconcile_letters::notify_after_batch".to_string(),
                probable_cause: "their unsent rows will reach them on reconnect".to_string(),
            },
        );
    }
    log(
        "server",
        format!(
            "reconcile batch of {} letter(s): notified {} client(s)",
            letter_count,
            report.reached.len()
        ),
    );
}
use crate::logging::log_error;
use crate::telephone::Telephone;
use db_core::db::{db_actions_for_drift, user_registration};
use db_wrapper::android_mascot::LiveForever;
use server_errors::{ErrorDetails, LogCategory, ServErr};
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{LoginPayload, LoginReply};
use std::sync::{Arc, Mutex};

use super::core::err_bytes;

pub async fn log_in(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<LiveForever>>,
    telephone: &Telephone,
    socket_id: u64,
) -> Vec<u8> {
    let payload = match LoginPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            return err_bytes(ServErr::BadCode(format!(
                "couldn't read login payload: {e:?}"
            )));
        }
    };

    let drift = {
        let db_wrapper = state.lock().unwrap();
        match user_registration::user_exists(&db_wrapper, &payload.user_name) {
            Ok(false) => {
                return err_bytes(ServErr::DbErrorContainer(DbError::IllegalInput(format!(
                    "no such user: '{}'",
                    payload.user_name
                ))));
            }
            Ok(true) => {}
            Err(e) => return err_bytes(ServErr::DbErrorContainer(e)),
        }
        match db_actions_for_drift::get_current(&db_wrapper) {
            Ok(Some(dv)) => dv,
            Ok(None) => {
                return err_bytes(ServErr::ServerError(ErrorDetails {
                    err_msg: "server has no drift_version row".to_string(),
                    file: file!().to_string(),
                    method: "login::log_in".to_string(),
                    probable_cause: "ensure_minted didn't run at startup".to_string(),
                }));
            }
            Err(e) => return err_bytes(ServErr::DbErrorContainer(e)),
        }
    };

    telephone
        .attach_name(socket_id, payload.user_name.clone())
        .await;

    if let Err(e) = telephone.notify_socket(socket_id).await {
        log_error(
            &payload.user_name,
            LogCategory::LetterDelivery,
            &ErrorDetails {
                err_msg: format!("login notify failed: {e:?}"),
                file: file!().to_string(),
                method: "login::log_in".to_string(),
                probable_cause: "the telephone's task is not running".to_string(),
            },
        );
    }

    let reply: Result<LoginReply, DbError> = Ok(LoginReply {
        drift_version: drift.version,
        drift_reason: drift.reason,
        drift_solution: drift.drift_solution,
    });
    reply.to_payload()
}
use crate::logging::log_error;
use crate::telephone::Telephone;
use db_core::db::{operations_drift_version, user_registration};
use db_wrapper::android_mascot::LiveForever;
use error_stuff::{ErrorDetails, LogCategory};
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
            return err_bytes(DbError::BadCode(format!(
                "couldn't read login payload: {e:?}"
            )));
        }
    };

    let drift = {
        let locked = state.lock().unwrap();
        match user_registration::user_exists(&locked, &payload.user_name) {
            Ok(false) => {
                return err_bytes(DbError::IllegalInput(format!(
                    "no such user: '{}'",
                    payload.user_name
                )));
            }
            Ok(true) => {}
            Err(e) => return err_bytes(e),
        }
        match operations_drift_version::get_current(&locked) {
            Ok(Some(pair)) => pair,
            Ok(None) => {
                return err_bytes(DbError::BadCode(
                    "server has no drift_version row".to_string(),
                ));
            }
            Err(e) => return err_bytes(e),
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

    let reply = LoginReply {
        drift_version: drift.0,
        drift_reason: drift.1,
    };
    reply.to_payload()
}
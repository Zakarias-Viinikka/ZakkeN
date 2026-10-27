use crate::logging::log_error;
use crate::server_state::ServerState;
use crate::telephone::Telephone;
use db_core::db::user_registration;
use error_stuff::{ErrorDetails, LogCategory};
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::LoginPayload;
use std::sync::{Arc, Mutex};

use super::err_bytes;

pub async fn handle(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
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

    let exists = {
        let locked = state.lock().unwrap();
        match user_registration::user_exists(&locked.liver, &payload.user_name) {
            Ok(b) => b,
            Err(e) => return err_bytes(e),
        }
    };

    if !exists {
        return err_bytes(DbError::IllegalInput(format!(
            "no such user: '{}'",
            payload.user_name
        )));
    }

    telephone
        .attach_name(socket_id, payload.user_name.clone())
        .await;

    // Nudge the just-connected socket so it writes a Nudge to the client
    // and the client pulls its unsent letters.
    if let Err(e) = telephone.nudge_socket(socket_id).await {
        log_error(
            &payload.user_name,
            LogCategory::LetterDelivery,
            &ErrorDetails {
                err_msg: format!("login nudge failed: {e:?}"),
                file: file!().to_string(),
                method: "login::handle".to_string(),
                probable_cause: "the telephone's task is not running".to_string(),
            },
        );
    }

    ok_serialized()
}
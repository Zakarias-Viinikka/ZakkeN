use crate::allowed_requests::{err_outcome, HandlerOutcome};
use crate::logging::log_error;
use crate::server_state::ServerState;
use connected_clients::Telephone;
use db_core::db::user_registration;
use error_stuff::{ErrorDetails, LogCategory};
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::LoginPayload;
use std::sync::{Arc, Mutex};

pub async fn handle(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
    telephone: &Telephone,
) -> HandlerOutcome {
    let payload = match LoginPayload::un_payloadify(&msg_content) {
        Ok(p) => p,
        Err(e) => {
            return err_outcome(DbError::BadCode(format!(
                "couldn't read login payload: {e:?}"
            )));
        }
    };

    let exists = {
        let locked = state.lock().unwrap();
        match user_registration::user_exists(&locked.liver, &payload.user_name) {
            Ok(b) => b,
            Err(e) => return err_outcome(e),
        }
    };

    if !exists {
        return err_outcome(DbError::IllegalInput(format!(
            "no such user: '{}'",
            payload.user_name
        )));
    }

    // Register this socket's nudge channel, then nudge it once so its
    // socket task writes a Nudge to the client and the client asks for
    // its unsent letters through GetMyUnsentLetters. The executor handles
    // one job at a time, so no live fan-out can slip in between the two.
    let incoming_letters = telephone.add_client(payload.user_name.clone());
    if let Err(e) = telephone.nudge(payload.user_name.clone()).await {
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

    HandlerOutcome {
        bytes: ok_serialized(),
        just_identified: Some(payload.user_name),
        incoming_letters: Some(incoming_letters),
    }
}
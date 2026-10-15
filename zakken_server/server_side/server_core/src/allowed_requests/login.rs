use crate::allowed_requests::{err_outcome, HandlerOutcome};
use crate::logging::{log, log_error};
use crate::server_state::ServerState;
use connected_clients::ConnectedClients;
use db_core::db::operations_love_letters;
use db_core::db::user_registration;
use error_stuff::{ErrorDetails, LogCategory};
use protocol::error::DbError;
use protocol::serialization::{Convert, ok_serialized};
use server_protocol::LoginPayload;
use std::sync::{Arc, Mutex};

pub async fn handle(
    msg_content: Vec<u8>,
    state: &Arc<Mutex<ServerState>>,
    clients: &ConnectedClients,
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

    // Register this socket's push channel, then drain the backlog into it.
    // The executor handles one job at a time, so no new letter can be
    // fanned out to this user between add_client and the backfill.
    let incoming_letters = clients.add_client(payload.user_name.clone());
    backfill_unsent(state, clients, &payload.user_name).await;

    HandlerOutcome {
        bytes: ok_serialized(),
        just_identified: Some(payload.user_name),
        incoming_letters: Some(incoming_letters),
    }
}

/// Push every unsent letter for this user, oldest first. The rows stay
/// until the client acks, so a failed push just means the row is still
/// there for next login.
async fn backfill_unsent(
    state: &Arc<Mutex<ServerState>>,
    clients: &ConnectedClients,
    user_name: &str,
) {
    let ids = {
        let locked = state.lock().unwrap();
        match operations_love_letters::get_unsent_letter_ids_for_user(&locked.liver, user_name) {
            Ok(ids) => ids,
            Err(e) => {
                log_error(
                    user_name,
                    LogCategory::Database,
                    &ErrorDetails {
                        err_msg: format!("backfill failed to read unsent ids: {e:?}"),
                        file: file!().to_string(),
                        method: "backfill_unsent".to_string(),
                        probable_cause: String::new(),
                    },
                );
                return;
            }
        }
    };

    if ids.is_empty() {
        return;
    }

    log(
        user_name,
        format!("backfill: {} unsent letter(s)", ids.len()),
    );

    for id in ids {
        if let Err(e) = clients.send_to(user_name.to_string(), id).await {
            log_error(
                user_name,
                LogCategory::LetterDelivery,
                &ErrorDetails {
                    err_msg: format!("backfill: push letter {id} failed: {e:?}"),
                    file: file!().to_string(),
                    method: "backfill_unsent".to_string(),
                    probable_cause: "the push channel was closed".to_string(),
                },
            );
        }
    }
}
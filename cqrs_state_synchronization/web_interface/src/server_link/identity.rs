//! Register and login against the server.
//!
//! Tries the saved `server_user_name` first. If it's missing or the
//! server says "no such user" (server db wiped), it registers under the
//! local setup name, appending -1, -2, ... until one sticks, and saves
//! the winner.

use leptos::prelude::GetUntracked;
use protocol::error::DbError;
use protocol::serialization::{Base64Bytes, Convert};
use server_protocol::{LoginPayload, Message as ServerMessage, RegisterPayload, Request};
use shared_internals_for_server::ResponseRouter;

use crate::db::key_value_storage_db_actions;

const SERVER_NAME_KEY: &str = "server_user_name";

pub async fn login(router: &ResponseRouter) -> Result<(), String> {
    let Some(setup_name) = crate::db::cached_ids::USER_ID.get_untracked() else {
        return Err("USER_ID not set".to_string());
    };

    if let Some(saved) = read_saved_name().await? {
        match try_login(router, &saved).await? {
            LoginOutcome::Ok => return Ok(()),
            LoginOutcome::NoSuchUser => {
                leptos::logging::warn!(
                    "saved server name {saved:?} doesn't exist on server; clearing and re-registering"
                );
                clear_saved_name().await?;
                // Stopgap for "server db wiped": without this the
                // persistent seen set would drop letters whose ids
                // restart at 1. A proper dataset-generation id on the
                // server replaces this later.
                if let Err(e) =
                    crate::db::db_helpers_for_web_client::clear_seen_server_letters().await
                {
                    leptos::logging::warn!("clear_seen_server_letters: {e:?}");
                }
            }
            LoginOutcome::Other(msg) => return Err(msg),
        }
    }

    for candidate in candidates(setup_name) {
        match try_register(router, &candidate).await? {
            RegisterOutcome::Ok => {
                save_name(&candidate).await?;
                return match try_login(router, &candidate).await? {
                    LoginOutcome::Ok => Ok(()),
                    other => Err(format!("login after register failed: {other:?}")),
                };
            }
            RegisterOutcome::AlreadyRegistered => continue,
            RegisterOutcome::Other(msg) => return Err(msg),
        }
    }
    Err("all candidate server names were already registered".to_string())
}

fn candidates(base: String) -> impl Iterator<Item = String> {
    std::iter::once(base.clone())
        .chain((1u32..).map(move |n| format!("{base}-{n}")))
        .take(50)
}

#[derive(Debug)]
enum LoginOutcome {
    Ok,
    NoSuchUser,
    Other(String),
}

async fn try_login(router: &ResponseRouter, name: &str) -> Result<LoginOutcome, String> {
    let payload = LoginPayload {
        user_name: name.to_string(),
    };
    let bytes = send_request(router, Request::Login, payload.to_payload()).await?;
    let result: Result<(), DbError> = Convert::un_payloadify(&bytes)
        .map_err(|e| format!("decode login reply: {e:?}"))?;
    match result {
        Ok(()) => Ok(LoginOutcome::Ok),
        Err(DbError::IllegalInput(msg)) if msg.contains("no such user") => {
            Ok(LoginOutcome::NoSuchUser)
        }
        Err(e) => Ok(LoginOutcome::Other(format!("login failed: {e:?}"))),
    }
}

#[derive(Debug)]
enum RegisterOutcome {
    Ok,
    AlreadyRegistered,
    Other(String),
}

async fn try_register(router: &ResponseRouter, name: &str) -> Result<RegisterOutcome, String> {
    let payload = RegisterPayload {
        user_name: name.to_string(),
    };
    let bytes = send_request(router, Request::Register, payload.to_payload()).await?;
    let result: Result<(), DbError> = Convert::un_payloadify(&bytes)
        .map_err(|e| format!("decode register reply: {e:?}"))?;
    match result {
        Ok(()) => Ok(RegisterOutcome::Ok),
        Err(DbError::IllegalInput(msg)) if msg.contains("already registered") => {
            Ok(RegisterOutcome::AlreadyRegistered)
        }
        Err(e) => Ok(RegisterOutcome::Other(format!("register failed: {e:?}"))),
    }
}

async fn send_request(
    router: &ResponseRouter,
    request: Request,
    payload: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let reply_rx = router
        .send_request(|id| ServerMessage {
            message_id: id,
            request,
            payload: Base64Bytes(payload),
        })
        .map_err(|e| format!("send_request: {e:?}"))?;
    reply_rx
        .await
        .map_err(|_| "reply dropped (connection lost)".to_string())
}

async fn read_saved_name() -> Result<Option<String>, String> {
    let v = key_value_storage_db_actions::read_key_value_from_db(SERVER_NAME_KEY)
        .await
        .map_err(|e| format!("read server_user_name: {e:?}"))?;
    Ok(v.filter(|s| !s.is_empty()))
}

async fn save_name(name: &str) -> Result<(), String> {
    key_value_storage_db_actions::write_key_value_to_db(SERVER_NAME_KEY, name)
        .await
        .map_err(|e| format!("save server_user_name: {e:?}"))
}

async fn clear_saved_name() -> Result<(), String> {
    key_value_storage_db_actions::write_key_value_to_db(SERVER_NAME_KEY, "")
        .await
        .map_err(|e| format!("clear server_user_name: {e:?}"))
}

//! Register and login against the server.
//!
//! Tries the saved `server_user_name` first. If it's missing or the
//! server says "no such user" (server db wiped), it tries to register
//! once under the local setup name. No auto-suffix: if the name is
//! taken, a popup asks the user to pick a different one in /setup and
//! reload.
//!
//! "Same name = same user" is intentional. There's no auth. Two devices
//! with the same name are the same user; the second one to connect
//! takes over the first one's socket.

use leptos::prelude::{GetUntracked, Set};
use protocol::error::DbError;
use protocol::serialization::Convert;
use server_protocol::{LoginPayload, RegisterPayload, Request};
use shared_internals_for_server::RouterHandle;

use crate::db::{cached_ids, key_value_storage_db_actions};
use crate::leptos_components::small_components::popup::create_popup;

const SERVER_NAME_KEY: &str = "server_user_name";
const USER_ID_KEY: &str = "user_id";

pub async fn login(router: &RouterHandle) -> Result<(), String> {
    let Some(setup_name) = cached_ids::USER_ID.get_untracked() else {
        return Err("USER_ID not set".to_string());
    };
    if setup_name.is_empty() {
        return Err("USER_ID is empty".to_string());
    }

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

    match try_register(router, &setup_name).await? {
        RegisterOutcome::Ok => {
            save_name(&setup_name).await?;
            match try_login(router, &setup_name).await? {
                LoginOutcome::Ok => Ok(()),
                other => Err(format!("login after register failed: {other:?}")),
            }
        }
        RegisterOutcome::AlreadyRegistered => {
            create_popup(format!(
                "Server says '{}' is already taken. Open /setup to pick a different name (like '{}-1'), then reload the page.",
                setup_name, setup_name
            ));
            clear_user_id().await?;
            Err(format!(
                "server name '{}' is already taken; pick a different one in /setup",
                setup_name
            ))
        }
        RegisterOutcome::Other(msg) => Err(msg),
    }
}

#[derive(Debug)]
enum LoginOutcome {
    Ok,
    NoSuchUser,
    Other(String),
}

async fn try_login(router: &RouterHandle, name: &str) -> Result<LoginOutcome, String> {
    let payload = LoginPayload {
        user_name: name.to_string(),
    };
    let bytes = router
        .send_request(Request::Login, payload.to_payload())
        .await
        .map_err(|e| format!("login request: {e:?}"))?;
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

async fn try_register(router: &RouterHandle, name: &str) -> Result<RegisterOutcome, String> {
    let payload = RegisterPayload {
        user_name: name.to_string(),
    };
    let bytes = router
        .send_request(Request::Register, payload.to_payload())
        .await
        .map_err(|e| format!("register request: {e:?}"))?;
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

async fn clear_user_id() -> Result<(), String> {
    key_value_storage_db_actions::write_key_value_to_db(USER_ID_KEY, "")
        .await
        .map_err(|e| format!("clear user_id: {e:?}"))?;
    cached_ids::USER_ID.set(None);
    Ok(())
}
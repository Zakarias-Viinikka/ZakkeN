use error_stuff::CqrsErr;
use leptos::prelude::*;
use leptos::logging::log;

use crate::db::cached_ids;
use crate::leptos_components::small_components::popup::show_popup;

use client_table_blueprints::web_migrations::main_for_bringing_the_db_to_the_newest_schema_version::bring_the_db_to_the_newest_schema_version;

pub async fn init_all() {
    if let Err(e) = bring_the_db_to_the_newest_schema_version().await {
        log!("bring db to newest schema version failed: {:?}", e);
        show_popup(format!("bring db to newest schema version failed: {e:?}"));
        return;
    }

    if let Err(e) = init_session_id().await {
        log!("init session_id failed: {:?}", e);
        show_popup(format!("init session_id failed: {e:?}"));
    }

    if let Err(e) = init_user_id().await {
        log!("init user_id failed: {:?}", e);
        show_popup(format!("init user_id failed: {e:?}"));
    }
}

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";

async fn init_session_id() -> Result<(), CqrsErr> {
    let existing =
        crate::db::key_value_storage_db_actions::read_key_value_from_db(SESSION_ID_KEY).await?;
    let next = match existing {
        Some(s) => s.parse::<u64>().map(|n| n + 1).unwrap_or(1),
        None => 0,
    };
    let next_str = next.to_string();
    crate::db::key_value_storage_db_actions::write_key_value_to_db(SESSION_ID_KEY, &next_str)
        .await?;
    cached_ids::SESSION_ID.set(Some(next_str));
    Ok(())
}

async fn init_user_id() -> Result<(), CqrsErr> {
    let existing = crate::db::key_value_storage_db_actions::read_key_value_from_db(USER_ID_KEY)
        .await?
        .filter(|s| !s.is_empty());
    cached_ids::USER_ID.set(existing);
    Ok(())
}




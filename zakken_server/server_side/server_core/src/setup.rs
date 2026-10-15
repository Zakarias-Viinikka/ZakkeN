use crate::logging::log;
use crate::server_state::ServerState;
use db_core::personal_db_wrapper;
use std::sync::{Arc, Mutex};

/// Open the db and create every table the server needs, if missing.
/// Returns None (after logging) if the db can't be opened, which is the
/// only failure the caller should refuse to start on.
pub fn open_and_prepare() -> Option<Arc<Mutex<ServerState>>> {
    let liver = match personal_db_wrapper::new_liver() {
        Ok(l) => l,
        Err(e) => {
            log("server", format!("failed to open db: {e:?}"));
            return None;
        }
    };
    if let Err(e) = personal_db_wrapper::create_table_if_not_exist(&liver) {
        log("server", format!("failed to create socket_testing_table: {e:?}"));
    }
    if let Err(e) = personal_db_wrapper::create_data_if_not_exist(&liver) {
        log("server", format!("failed to create socket_testing_table data: {e:?}"));
    }
    if let Err(e) = db_core::db::user_registration::create_table_if_not_exist(&liver) {
        log("server", format!("failed to create registered_users: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_love_letters::create_tables_if_not_exist(&liver) {
        log("server", format!("failed to create love letter tables: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_pages::create_table_if_not_exist(&liver) {
        log("server", format!("failed to create pages: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_logs::create_table_if_not_exist(&liver) {
        log("server", format!("failed to create logs: {e:?}"));
    }
    if let Err(e) = db_core::db::operations_key_value::create_table_if_not_exist(&liver) {
        log("server", format!("failed to create key_value_storage: {e:?}"));
    }

    // Bump the session id before anything else runs, so every log row
    // written this run carries it.
    match db_core::db::operations_key_value::bump_session_id(&liver) {
        Ok(id) => crate::logging::install_session_id(id.to_string()),
        Err(e) => {
            log("server", format!("couldn't bump session id: {e:?}"));
            crate::logging::install_session_id("unknown".to_string());
        }
    }

    Some(Arc::new(Mutex::new(ServerState::new(liver))))
}

#[macro_use]
pub mod db;
pub mod leptos_components;

pub fn get_time() -> String {
    ((js_sys::Date::now() / 1000.0) as u64).to_string()
}

pub mod complex_state;

pub use complex_state::conn_state::{
    plexget_conn_state, plexget_conn_state_untracked, plexset_conn_state,
};
pub use complex_state::drift_version::{
    plexget_drift_version, plexget_drift_version_from_db, plexset_drift_version,
    plexset_make_new_drift_version,
};
pub mod new_local_block_id;
pub mod page_state;
pub mod drag_reorder_container;
pub mod expose_methods_for_testing;

pub mod owner_of_interacting_with_server;
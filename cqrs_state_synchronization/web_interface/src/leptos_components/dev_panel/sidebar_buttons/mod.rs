pub mod deliberate_lonewolf_button;
pub mod delete_everything;
pub mod insert_three_pages;
pub mod reconcile_button;
pub mod reset_everything;
pub mod wipe_self_and_adopt_server;
pub mod wipe_server;

pub use deliberate_lonewolf_button::DeliberateLonewolfButton;
pub use delete_everything::DeleteEverythingButton;
pub use insert_three_pages::InsertThreePagesButton;
pub use reconcile_button::ReconcileButton;
pub use reset_everything::ResetEverythingButton;
pub use wipe_self_and_adopt_server::WipeSelfAndAdoptServerButton;
pub use wipe_server::WipeServerButton;

use leptos::logging::log;

pub fn reload_the_page() {
    if let Some(window) = web_sys::window() {
        if let Err(e) = window.location().reload() {
            log!("reload failed: {:?}", e);
        }
    }
}

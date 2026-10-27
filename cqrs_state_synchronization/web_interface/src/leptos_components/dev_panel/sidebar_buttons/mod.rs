pub mod delete_everything;
pub mod insert_three_pages;
pub mod reset_everything;

pub use delete_everything::DeleteEverythingButton;
pub use insert_three_pages::InsertThreePagesButton;
pub use reset_everything::ResetEverythingButton;

use leptos::logging::log;

pub fn reload_the_page() {
    if let Some(window) = web_sys::window() {
        if let Err(e) = window.location().reload() {
            log!("reload failed: {:?}", e);
        }
    }
}

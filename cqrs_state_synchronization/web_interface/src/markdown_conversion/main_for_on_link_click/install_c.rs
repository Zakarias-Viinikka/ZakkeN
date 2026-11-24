//! Dummy command `[[Install c|...]]`. Only logs.
use super::on_link_click_error::OnLinkClickError;

pub fn run_install_c(rest: &str) -> Result<(), OnLinkClickError> {
    leptos::logging::log!("Install c ran, rest={rest:?}");
    Ok(())
}

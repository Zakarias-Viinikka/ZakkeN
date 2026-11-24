//! Dummy command `[[Install x|...]]`. Only logs.
use super::on_link_click_error::OnLinkClickError;

pub fn run_install_x(rest: &str) -> Result<(), OnLinkClickError> {
    leptos::logging::log!("Install x ran, rest={rest:?}");
    Ok(())
}

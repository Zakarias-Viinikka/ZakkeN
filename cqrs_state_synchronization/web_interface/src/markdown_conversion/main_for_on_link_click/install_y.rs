//! Dummy command `[[Install y|...]]`. Only logs.
use super::on_link_click_error::OnLinkClickError;

pub fn run_install_y(rest: &str) -> Result<(), OnLinkClickError> {
    leptos::logging::log!("Install y ran, rest={rest:?}");
    Ok(())
}

//! Dummy command `[[Install all things|...]]`. Runs the other install commands.
use super::install_c::run_install_c;
use super::install_x::run_install_x;
use super::install_y::run_install_y;
use super::on_link_click_error::OnLinkClickError;

pub fn run_install_all_things(rest: &str) -> Result<(), OnLinkClickError> {
    leptos::logging::log!("Install all things started, rest={rest:?}");
    run_install_x(rest)?;
    run_install_y(rest)?;
    run_install_c(rest)?;
    Ok(())
}

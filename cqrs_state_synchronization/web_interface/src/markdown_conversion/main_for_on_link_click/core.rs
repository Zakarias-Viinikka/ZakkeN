//! Entry for a click on a `[[command|rest]]` link.
//!
//! Caller pov: call `main_on_link_click(command, rest)`. It finds the file for that command,
//! runs it with `rest`, and logs the error if there is one. Nothing is swallowed.
//!
//! To add a command: make a new file in this folder, add `pub mod` in mod.rs, and add one line
//! to COMMANDS below.
use super::install_all_things::run_install_all_things;
use super::install_c::run_install_c;
use super::install_x::run_install_x;
use super::install_y::run_install_y;
use super::on_link_click_error::OnLinkClickError;
use super::page_nav::run_page_nav;

type CommandAction = fn(&str) -> Result<(), OnLinkClickError>;

/// Command name (the first part of the link) -> the function that handles it.
const COMMANDS: &[(&str, CommandAction)] = &[
    ("Install x", run_install_x),
    ("Install y", run_install_y),
    ("Install c", run_install_c),
    ("Install all things", run_install_all_things),
    ("PageNav", run_page_nav),
];

pub fn main_on_link_click(command: &str, rest: &str) {
    if let Err(e) = run_matching_command(command, rest) {
        leptos::logging::error!(
            "link click failed: err_msg={:?} method={:?} filepath={:?}",
            e.err_msg,
            e.method,
            e.filepath
        );
    }
}

fn run_matching_command(command: &str, rest: &str) -> Result<(), OnLinkClickError> {
    for (name, action) in COMMANDS {
        if *name == command {
            return action(rest);
        }
    }
    Err(OnLinkClickError {
        err_msg: format!("unknown link command {command:?} (rest was {rest:?})"),
        method: "run_matching_command".to_string(),
        filepath: file!().to_string(),
    })
}

mod core;
mod identity;
pub mod merge;
pub mod receive;
pub mod send;
mod two_tab;

pub use merge::REMOTE_DISABLED_OPEN_PAGE;

pub use core::{ConnState, CONN_STATE, start_server_link};
pub use receive::{make_letters_channel, spawn_worker, MENU_TICK};
pub use send::{kick_send, spawn_sender, PENDING_COUNT};
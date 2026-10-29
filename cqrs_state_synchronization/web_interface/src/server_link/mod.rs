mod core;
mod identity;
pub mod merge;
pub mod receive;
pub mod send;
mod two_tab;

pub use core::{ConnState, CONN_STATE, start_server_link};
pub use receive::{make_letters_channel, spawn_task_for_receiving_socket_messages, MENU_TICK};
pub use send::{spawn_sender, PENDING_COUNT};
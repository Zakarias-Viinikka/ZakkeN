pub mod connecting_gate;
mod core;
mod main_for_login_and_registration;
pub mod main_for_applying_love_letter_from_server_to_app;
pub mod main_for_receiving_spontaneous_love_letters_from_server;
pub mod send_unsent_love_letters_to_server_if_there_are_any;
pub mod server_port;
mod two_tab_guard;

pub use connecting_gate::{ConnectingGate, CONNECTING_GATE};
pub use core::{ConnState, CONN_STATE, connect_to_socket};
pub use main_for_receiving_spontaneous_love_letters_from_server::{make_letters_channel, spawn_task_for_receiving_socket_messages, MENU_TICK};
pub use send_unsent_love_letters_to_server_if_there_are_any::{spawn_sender, PENDING_COUNT};
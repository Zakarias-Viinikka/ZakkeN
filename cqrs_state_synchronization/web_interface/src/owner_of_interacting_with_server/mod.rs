mod core;
mod main_for_login_and_registration;
pub mod main_for_applying_love_letter_from_server_to_app;
pub mod main_for_receiving_spontaneous_love_letters_from_server;
pub mod send_unsent_love_letters_to_server_if_there_are_any;
mod two_tab_guard;

pub use core::{ConnState, CONN_STATE, start_server_link};
pub use main_for_receiving_spontaneous_love_letters_from_server::{make_letters_channel, spawn_task_for_receiving_socket_messages, MENU_TICK};
pub use send_unsent_love_letters_to_server_if_there_are_any::{spawn_sender, PENDING_COUNT};
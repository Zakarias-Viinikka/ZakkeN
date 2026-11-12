pub mod connecting_gate;
mod core;
mod drift;
pub mod main_for_applying_love_letter_from_server_to_app;
mod main_for_login_and_registration;
pub mod main_for_reconnecting_automatically;
pub mod main_for_receiving_spontaneous_love_letters_from_server;
pub mod reconcile_requests;
pub mod send_unsent_love_letters_to_server_if_there_are_any;
pub mod server_port;
mod two_tab_guard;


pub use connecting_gate::{CONNECTING_GATE, ConnectingGate};
pub use core::{ConnState, connect_to_socket, sync_may_run};
pub use drift::enter_lonewolf;
pub use main_for_receiving_spontaneous_love_letters_from_server::{
    MENU_TICK, make_letters_channel, spawn_task_for_receiving_socket_messages,
};
pub use send_unsent_love_letters_to_server_if_there_are_any::{PENDING_COUNT, spawn_sender};

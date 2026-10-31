//! Connection lifecycle state, generation signals, counts, and the boot
//! sequence. Mirrors `cqrs_state_synchronization/web_interface/src/owner_of_interacting_with_server/`.

pub mod conn_state;
pub mod dataset_generation;
pub mod internet_switch;
pub mod locked;
pub mod pending_queue;
pub mod receiving;
pub mod restart;
pub mod sending;
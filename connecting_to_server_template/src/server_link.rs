//! Connection lifecycle state, generation signals, counts, and the boot
//! sequence. Mirrors `cqrs_state_synchronization/web_interface/src/server_link/`.

pub mod conn_state;
pub mod dataset_generation;
pub mod pending_queue;
pub mod receiving;
pub mod restart;
pub mod sending;
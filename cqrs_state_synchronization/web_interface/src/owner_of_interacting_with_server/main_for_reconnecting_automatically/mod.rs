//! Caller pov: the task that decides when to try reconnecting.
//!
//! One task, one loop, once every second. Reads `ConnState` and asks
//! the router to try opening a socket when the client is offline.
//! Installed once from `App`.

pub mod core;

pub use core::spawn_task_to_auto_reconnect;
//! The Android half of talking to zakken_server.
//!
//! Same split as `client_internals/web`: this crate owns the socket, the
//! shared crate owns the routing. On Android there is no `web_sys`; a
//! native websocket crate will do the wire work. uniffi exposes the
//! result to Kotlin.
//!
//! Not started. Left here so the folder exists and the shape is written
//! down. When Android networking begins, this is the file.

pub fn placeholder() {}

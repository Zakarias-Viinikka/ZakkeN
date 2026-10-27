//! The browser half of talking to zakken_server.
//!
//! Sketched from
//! `rust_practice/server_template_3/client_side/how_to_talk_to_server`.
//! The shapes there are the reference; this file will fill in.
//!
//! What lives here:
//! - Opening the WebSocket (`web_sys::WebSocket::new`).
//! - Attaching handlers for open / message / error / close.
//! - A task that reads `RouterHandles::outgoing` and writes each Message
//!   to the socket.
//! - A message handler that decodes JSON into a `Response` and forwards
//!   it into `RouterHandles::incoming_responses_tx`.
//!
//! What does not live here: the response routing. That's in
//! `shared_internals_for_server::ResponseRouter`.
//!
//! Not wired up yet. The web client (`cqrs_state_synchronization/
//! web_interface`) currently has no networking; when it gets some, this
//! crate is what it depends on.

pub fn placeholder() {}

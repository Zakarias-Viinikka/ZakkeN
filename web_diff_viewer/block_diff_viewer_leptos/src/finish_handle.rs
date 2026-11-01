//! Caller pov: a small Copy value the caller creates once, passes into
//! DiffViewer, and can also hold onto so any button anywhere in the app
//! can trigger "finish now". Calling the method bumps an internal
//! counter; the viewer runs an Effect on that counter and produces the
//! output. This is the owner+handle split in leptos form — the handle
//! is cheap to clone and pass around, and the caller never sees the
//! signal inside it.

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct FinishHandle {
    pub(crate) counter: RwSignal<u32>,
}

impl Default for FinishHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl FinishHandle {
    pub fn new() -> Self {
        Self {
            counter: RwSignal::new(0),
        }
    }

    pub fn finish_and_write_the_result(&self) {
        self.counter.update(|c| *c = c.wrapping_add(1));
    }
}

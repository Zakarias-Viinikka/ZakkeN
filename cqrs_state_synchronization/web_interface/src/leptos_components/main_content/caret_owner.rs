//! `<CaretOwner />`: no visible output. The one place that places carets.
//!
//! One Effect watches the pending request (`complex_state/caret/caret_request.rs`) and the handler
//! registry (`complex_state/caret/caret_handlers_registry.rs`). When the request names a registered
//! block, the block's handler runs on the NEXT animation frame (callbacks change the row text
//! and ask for a caret in the same tick, and the new text needs a hop to reach the editor).
//! `At(pos)` beyond the editor's text is retried each frame up to MAX_RETRY_FRAMES, then the
//! handler clamps. A request for a block that is not registered yet just waits; the registry
//! changing re-runs the Effect. Any request still pending after DROP_AFTER_MS is dropped.
//! Key interpretation is NOT here (keys.rs); this only places carets.
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

use crate::complex_state::caret::caret_handlers_registry::{
    plexget_caret_handler, plexget_caret_handler_untracked,
};
use crate::complex_state::caret::caret_request::{
    CaretRequest, plexget_caret_request, plexget_caret_request_untracked,
    plexset_clear_caret_request_if_serial,
};

const MAX_RETRY_FRAMES: u32 = 10;
const DROP_AFTER_MS: u32 = 2000;

#[component]
pub fn CaretOwner() -> impl IntoView {
    // 0 = none (serials start at 1).
    let serial_with_expiry_armed = Arc::new(AtomicU64::new(0));
    let serial_being_placed = Arc::new(AtomicU64::new(0));

    Effect::new(move |_| {
        let Some(request) = plexget_caret_request() else {
            return;
        };

        if serial_with_expiry_armed.swap(request.serial, Ordering::Relaxed) != request.serial {
            drop_request_after_a_while(request.serial);
        }

        // Tracked read: a block registering later re-runs this Effect.
        if plexget_caret_handler(&request.block_id).is_none() {
            return;
        }
        if serial_being_placed.swap(request.serial, Ordering::Relaxed) == request.serial {
            return; // already on its way
        }
        place_on_next_frame(request, 0, serial_being_placed.clone());
    });
}

fn place_on_next_frame(request: CaretRequest, frames_used: u32, being_placed: Arc<AtomicU64>) {
    run_on_next_animation_frame(move || {
        let still_current = plexget_caret_request_untracked().map(|r| r.serial) == Some(request.serial);
        if !still_current {
            release(&being_placed, request.serial); // a newer request has its own run
            return;
        }
        let Some(handler) = plexget_caret_handler_untracked(&request.block_id) else {
            // Block vanished. Keep the request; re-registering re-runs the Effect.
            release(&being_placed, request.serial);
            return;
        };
        let is_last_try = frames_used >= MAX_RETRY_FRAMES;
        let placed = handler(request.place, is_last_try);
        if placed || is_last_try {
            plexset_clear_caret_request_if_serial(request.serial);
            release(&being_placed, request.serial);
        } else {
            place_on_next_frame(request, frames_used + 1, being_placed);
        }
    });
}

fn release(being_placed: &AtomicU64, serial: u64) {
    let _ = being_placed.compare_exchange(serial, 0, Ordering::Relaxed, Ordering::Relaxed);
}

fn drop_request_after_a_while(serial: u64) {
    spawn_local(async move {
        TimeoutFuture::new(DROP_AFTER_MS).await;
        plexset_clear_caret_request_if_serial(serial);
    });
}

fn run_on_next_animation_frame(f: impl FnOnce() + 'static) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let cb = Closure::once_into_js(f);
    let _ = window.request_animation_frame(cb.unchecked_ref());
}

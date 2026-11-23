//! Complex state: the reactive registry "block_id -> handler that places the caret".
//!
//! Who touches it:
//! - The markdown converter (`markdown_conversion/`, wired in step 3) registers a handler once
//!   its editor is ready, and unregisters on cleanup with `..._if_still_mine`, so a late cleanup
//!   of an old mount cannot remove a newer registration made under the same block id.
//! - `leptos_components/main_content/caret_owner.rs` (`CaretOwner`) reads it (tracked in its
//!   Effect, so a block registering re-runs the owner; untracked inside animation frames).
//!
//! The handler gets the place and `is_last_try`. It returns true when the caret was placed.
//! For `At(pos)` with `pos` beyond the editor's text it returns false, unless `is_last_try` is
//! true, in which case it clamps and returns true.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use leptos::prelude::*;

use super::caret_request::CaretPlace;

pub type CaretHandler = Arc<dyn Fn(CaretPlace, bool) -> bool + Send + Sync>;

/// Proof of which registration is yours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretHandlerToken(u64);

#[derive(Clone)]
struct Entry {
    token: u64,
    handler: CaretHandler,
}

static REGISTRY: LazyLock<ArcRwSignal<HashMap<String, Entry>>> =
    LazyLock::new(|| ArcRwSignal::new(HashMap::new()));

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);

/// Registers (or replaces) the handler for `block_id`.
pub fn plexset_register_caret_handler(block_id: String, handler: CaretHandler) -> CaretHandlerToken {
    let token = NEXT_TOKEN.fetch_add(1, Ordering::Relaxed);
    REGISTRY.update(|m| {
        m.insert(block_id, Entry { token, handler });
    });
    CaretHandlerToken(token)
}

/// Removes the entry only if it is still the one `token` registered.
pub fn plexset_unregister_caret_handler_if_still_mine(block_id: &str, token: CaretHandlerToken) {
    let still_mine = REGISTRY
        .with_untracked(|m| m.get(block_id).map(|e| e.token) == Some(token.0));
    if still_mine {
        REGISTRY.update(|m| {
            m.remove(block_id);
        });
    }
}

pub fn plexget_caret_handler(block_id: &str) -> Option<CaretHandler> {
    REGISTRY.with(|m| m.get(block_id).map(|e| e.handler.clone()))
}

pub fn plexget_caret_handler_untracked(block_id: &str) -> Option<CaretHandler> {
    REGISTRY.with_untracked(|m| m.get(block_id).map(|e| e.handler.clone()))
}

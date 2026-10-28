use leptos::prelude::Set;

use super::ui_signals::{FOCUS_BLOCK_AT_START, FOCUS_BLOCK_ID};

pub fn at_end(block_id: String) {
    FOCUS_BLOCK_ID.set(Some(block_id));
}

pub fn at_start(block_id: String) {
    FOCUS_BLOCK_AT_START.set(Some(block_id));
}
//! Caller pov: ask for a caret in a block. These only write the request that `CaretOwner`
//! handles (`caret_request.rs`). Re-exported as `complex_state::focus`, so callers still write
//! `focus::at_end(...)`.
use super::caret_request::{CaretPlace, plexset_caret_request};

pub fn at_end(block_id: String) {
    plexset_caret_request(block_id, CaretPlace::End);
}

pub fn at_start(block_id: String) {
    plexset_caret_request(block_id, CaretPlace::Start);
}

/// `utf16_pos` is a UTF-16 code unit offset, which is what the editor uses.
pub fn at_pos(block_id: String, utf16_pos: usize) {
    plexset_caret_request(block_id, CaretPlace::At(utf16_pos));
}
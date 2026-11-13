//! Caller pov: turn the viewer's per-page edits (love-letter sketches)
//! into full love-letter values with snapshots. All files here are
//! pure — no leptos, no network, no DB. The entry point is
//! `turn_all_edits_into_love_letters`.

pub mod main_for_turning_page_sketches_into_love_letters;
pub mod scratch_page;
pub mod turn_one_change_into_a_love_letter;

pub use main_for_turning_page_sketches_into_love_letters::turn_all_edits_into_love_letters;
//! Turn the server's copy of a page plus the user's chosen block list
//! into love letters. All files here are pure — no leptos, no network,
//! no DB. The entry point is `create_love_letters_out_of_reconcile`.

pub mod align_two_block_lists;
pub mod main_for_turning_a_reconcile_choice_into_love_letters;
pub mod scratch_page;
pub mod turn_one_change_into_a_love_letter;

pub use main_for_turning_a_reconcile_choice_into_love_letters::create_love_letters_out_of_reconcile;

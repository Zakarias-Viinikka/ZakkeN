pub mod moves_between_unpaired_blocks_of_text;
pub mod pairs_two_runs_by_similarity;
pub mod similarity_between_two_blocks_of_text;
pub mod the_diff_between_two_pages_of_blocks;
pub mod the_diff_row_and_its_markers;

pub use moves_between_unpaired_blocks_of_text::{
    mark_moves, EXACT_MOVE_MIN_NON_WS_CHARS, FUZZY_MOVE_MIN_NON_WS_CHARS,
    FUZZY_MOVE_MIN_SCORE, FUZZY_PAIR_CAP,
};
pub use pairs_two_runs_by_similarity::align;
pub use similarity_between_two_blocks_of_text::{
    dice_over_tokens, edit_distance, inline_spans, pairing_similarity_if_worthy,
    token_similarity, tokenize, INLINE_MAX_CHARS, PAIR_THRESHOLD_LONG, PAIR_THRESHOLD_SHORT,
};
pub use the_diff_between_two_pages_of_blocks::rows_for_two_pages;
pub use the_diff_row_and_its_markers::{
    LeftMarker, Relationship, RightMarker, Row, Slot,
};
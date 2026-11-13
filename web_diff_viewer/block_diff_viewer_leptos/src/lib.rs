//! Caller pov: a Leptos component that renders a side-by-side diff
//! between two pages of blocks. The right side is editable and rows are
//! draggable. The caller provides the two pages as signals (the server
//! side optional, for a client-only page), a `FinishHandle` for "give me
//! the result", an output signal the viewer writes to, and (optionally)
//! a `Callback` that runs after each finish.
//!
//! `ViewerOutput` carries two things:
//! - `kept_blocks`: the editable right-side blocks in visual order, each
//!   with the id/metadata/frac it came in with.
//! - `sketches`: the love-letter sketches that would turn the server's
//!   page into `kept_blocks`. Empty if `server_page` is `None`.
//!
//! Setup: link `block_diff_viewer_leptos/css/block_diff_viewer.scss`
//! in your `index.html`. That is the only required wiring.
//!
//! Internals pov: the crate is split into reactive components (in
//! `the_diff_viewer.rs`, `the_frozen_viewer_for_one_mount.rs`, and
//! `complex_state/`) and pure modules (`map_caret_across_edit.rs`,
//! `reorder_rows_by_drag_math.rs`, `segs_for_highlighting.rs`,
//! `the_edit_state_of_one_block_of_text.rs`,
//! `the_pure_function_that_builds_the_output.rs`,
//! `make_sketches_out_of_two_pages.rs`). The pure modules are
//! native-tested (`c t` at workspace root). The reactive modules are
//! not tested; the example apps are the manual test.

pub mod finish_handle;
pub mod make_sketches_out_of_two_pages;
pub mod map_caret_across_edit;
pub mod reorder_rows_by_drag_math;
pub mod segs_for_highlighting;
pub mod complex_state;
pub mod the_diff_viewer;
pub mod the_drag_session_state;
pub mod the_edit_state_of_one_block_of_text;
pub mod the_frozen_viewer_for_one_mount;
pub mod the_public_types;
pub mod the_pure_function_that_builds_the_output;

pub use finish_handle::FinishHandle;
pub use make_sketches_out_of_two_pages::make_sketches_out_of_two_pages;
pub use map_caret_across_edit::map_caret_across_edit;
pub use reorder_rows_by_drag_math::{
    apply_row_move_event, build_row_move_event,
    insertion_slot_for_dragged_center, RowMoveEvent,
};
pub use segs_for_highlighting::{segs_for_a_line, Segment};
pub use complex_state::visual_row_order;
pub use the_diff_viewer::DiffViewer;
pub use the_edit_state_of_one_block_of_text::BlockEditState;
pub use the_public_types::{
    Block, LoveLetterSketch, PageBlocksForTheViewer, PickedSide, ViewerOutput,
};
pub use the_pure_function_that_builds_the_output::output_right_side_blocks_in_visual_order;
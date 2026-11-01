//! Caller pov: a Leptos component that renders a side-by-side diff
//! between two pages of blocks. The right side is editable and rows are
//! draggable. The caller provides the two pages as signals, a
//! `FinishHandle` for "give me the result", an output signal the viewer
//! writes the result to, and (optionally) a `Callback` that runs after
//! each finish.
//!
//! Setup: link `block_diff_viewer_leptos/css/block_diff_viewer.scss`
//! in your `index.html`. That is the only required wiring. The
//! `DiffViewer` component in `the_diff_viewer.rs` has the full caller
//! contract.
//!
//! Internals pov: the crate is split into reactive components (in
//! `the_diff_viewer.rs`, `the_frozen_viewer_for_one_mount.rs`, and
//! `shared_owner/`) and pure modules (`map_caret_across_edit.rs`,
//! `reorder_rows_by_drag_math.rs`, `segs_for_highlighting.rs`,
//! `the_edit_state_of_one_block_of_text.rs`,
//! `the_pure_function_that_builds_the_output.rs`). The pure modules
//! are native-tested (`c t` at workspace root). The reactive modules
//! are not tested; the example app is the manual test.

pub mod finish_handle;
pub mod map_caret_across_edit;
pub mod reorder_rows_by_drag_math;
pub mod segs_for_highlighting;
pub mod shared_owner;
pub mod the_diff_viewer;
pub mod the_drag_session_state;
pub mod the_edit_state_of_one_block_of_text;
pub mod the_frozen_viewer_for_one_mount;
pub mod the_pure_function_that_builds_the_output;

pub use finish_handle::FinishHandle;
pub use map_caret_across_edit::map_caret_across_edit;
pub use reorder_rows_by_drag_math::{
    apply_row_move_event, build_row_move_event,
    insertion_slot_for_dragged_center, RowMoveEvent,
};
pub use segs_for_highlighting::{segs_for_a_line, Segment};
pub use shared_owner::visual_row_order;
pub use the_diff_viewer::DiffViewer;
pub use the_edit_state_of_one_block_of_text::BlockEditState;
pub use the_pure_function_that_builds_the_output::output_right_side_texts_in_visual_order;
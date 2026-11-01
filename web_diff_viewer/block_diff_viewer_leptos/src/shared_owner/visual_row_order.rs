//! Caller pov: the visual order of the rows in the viewer, as a
//! RwSignal<Vec<usize>> of row ids. Row ids are indices into the
//! frozen rows list produced at mount.
//!
//! Touchers, once the later build steps land (drag, finish Effect,
//! position memos). Each one is documented so a reader can tell at a
//! glance what modifies the order and when.
//!
//! Writers:
//! - mount — initialises the order to 0..n, once, before any row is
//!   rendered. Nothing else writes before a user drag.
//! - drag module (later step) — applies a RowMoveEvent on drop: remove
//!   the moved id, insert after above (or at index 0 when above is
//!   None). The drag module never modifies the order itself; it only
//!   reports the event.
//!
//! Readers:
//! - the keyed <For> — iterates the order to render rows. Its key is
//!   the row id, so reordering only permutes DOM nodes; textareas and
//!   per-row state are not remounted.
//! - live_right_number_by_row memo — the right gutter number and the
//!   "moved to #N" chip read the position from this order.
//! - hairline derivation — hunk separators are drawn from the visual
//!   order plus each row's hunk_index.
//! - finish Effect — collects the output by walking the order and
//!   reading each right-side edit state.

use leptos::prelude::*;

pub fn create_visual_row_order(initial: Vec<usize>) -> RwSignal<Vec<usize>> {
    RwSignal::new(initial)
}

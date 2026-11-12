//! Caller pov: the coarse, reactive signals that views read while a row
//! is being dragged. Kept coarse on purpose so only the affected cells
//! re-render.
//!
//! Writers (in the_frozen_viewer_for_one_mount.rs):
//! - the mousedown handler — sets dragged_row_id when the drag actually
//!   moves past the threshold, not on press, so a click with no
//!   movement does nothing.
//! - the mousemove handler — sets dragged_translate_y (per pointer move)
//!   and drop_indicator (only when the slot changes).
//! - the mouseup, Escape, and blur handlers — all clear every signal
//!   via `clear()`.
//!
//! Readers:
//! - the dragged row's two cells — dragged_row_id (to know whether to
//!   read translate) and dragged_translate_y (to render the lift).
//! - the drop-target row's two cells — drop_indicator (to draw the
//!   before/after edge line).
//! - the root — dragged_row_id (to switch the whole-viewer class for
//!   cursor and pointer-events).

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropEdge {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DropIndicator {
    pub target_row_id: usize,
    pub edge: DropEdge,
}

#[derive(Clone, Copy)]
pub struct DragInProgressFeedback {
    pub dragged_row_id: RwSignal<Option<usize>>,
    pub dragged_translate_y: RwSignal<f32>,
    pub drop_indicator: RwSignal<Option<DropIndicator>>,
}

impl DragInProgressFeedback {
    pub fn new() -> Self {
        Self {
            dragged_row_id: RwSignal::new(None),
            dragged_translate_y: RwSignal::new(0.0),
            drop_indicator: RwSignal::new(None),
        }
    }

    pub fn clear(&self) {
        self.dragged_row_id.set(None);
        self.dragged_translate_y.set(0.0);
        self.drop_indicator.set(None);
    }
}

impl Default for DragInProgressFeedback {
    fn default() -> Self {
        Self::new()
    }
}

//! Things shared between files inside `bbox_drag_reorder_container/` only.

/// Where the user last was in the page: which block, and where in it.
/// `Default` (empty block id, position 0) means "nothing touched yet".
#[derive(Clone, Debug, Default)]
pub struct LastEditInfo {
    pub block_id: String,
    /// UTF-16 code unit offset (what the editor uses, same unit as `CaretPlace::At`).
    pub position_last_selected: usize,
}
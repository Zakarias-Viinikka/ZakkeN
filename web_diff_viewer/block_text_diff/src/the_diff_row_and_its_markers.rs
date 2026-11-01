//! Caller pov: one row of the diff output. Holds indices into the
//! caller's input slices, not the text itself — the component that
//! renders the diff carries the text and looks it up.

/// Where one block sits in the caller's input slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub block_index: usize,
}

/// What a row means to a reader who only sees the two columns side by
/// side. Equal rows line up; LeftOnly and RightOnly have content on
/// one side only; CollapsedDeleteAndInsert pairs unrelated texts on
/// one row (that pairing came from the collapse rule, not similarity);
/// Edit is the real paired case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relationship {
    Equal,
    Edit,
    CollapsedDeleteAndInsert,
    LeftOnly,
    RightOnly,
    /// A block whose text appears on both sides but at different
    /// positions. Rendered as one row with both slots filled and both
    /// bars purple. This is the one relationship that breaks the "left
    /// slots in original order" invariant — the left slot sits at the
    /// right slot's position, so the left column no longer reads in
    /// original order past a move. The right slots stay in original
    /// order, which is what the output function relies on.
    Moved,
}

/// What the left column's gutter and bar show for this row.
/// `MovedAway` names the right block it moved to, and whether the
/// texts are identical (so the label can say "edited" otherwise).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeftMarker {
    Unchanged,
    Deleted,
    Edited,
    MovedAway {
        partner_right_block_index: usize,
        identical: bool,
    },
}

/// Mirror of `LeftMarker` for the right column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RightMarker {
    Unchanged,
    Inserted,
    Edited,
    MovedHere {
        partner_left_block_index: usize,
        identical: bool,
    },
}

/// One row of the diff output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub left: Option<Slot>,
    pub right: Option<Slot>,
    pub relationship: Relationship,
    /// None on Equal rows; equal within a hunk.
    pub hunk_index: Option<usize>,
    pub left_marker: LeftMarker,
    pub right_marker: RightMarker,
}

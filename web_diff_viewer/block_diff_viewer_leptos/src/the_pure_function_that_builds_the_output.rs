//! Caller pov: pass the frozen rows, the current visual order, the
//! original right-side block list, and a closure that returns the
//! current text of each row's right side. Returns the kept blocks in
//! visual order, top to bottom. Each entry carries the id, metadata and
//! frac from the original right-side block at that row's slot, with the
//! closure's text as its text. LeftOnly rows (no right slot) are
//! skipped.
//!
//! Pure: no DOM, no signals, no leptos. The closure is what connects
//! this to the live edit states.

use block_text_diff::Row;
use generate_love_letters::Block;

pub fn output_right_side_blocks_in_visual_order(
    rows: &[Row],
    visual_order: &[usize],
    right_blocks: &[Block],
    mut text_for_row: impl FnMut(usize) -> Option<String>,
) -> Vec<Block> {
    let mut out = Vec::new();
    for &row_id in visual_order {
        if row_id >= rows.len() {
            continue;
        }
        let Some(slot) = rows[row_id].right else {
            continue;
        };
        let Some(text) = text_for_row(row_id) else {
            continue;
        };
        let Some(original) = right_blocks.get(slot.block_index) else {
            continue;
        };
        out.push(Block {
            block_id: original.block_id.clone(),
            text,
            metadata: original.metadata.clone(),
            frac_position: original.frac_position.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_text_diff::{LeftMarker, Relationship, RightMarker, Slot};

    fn b(id: &str, text: &str) -> Block {
        Block {
            block_id: id.to_string(),
            text: text.to_string(),
            metadata: String::new(),
            frac_position: String::new(),
        }
    }

    fn row_with_both(left: usize, right: usize) -> Row {
        Row {
            left: Some(Slot { block_index: left }),
            right: Some(Slot { block_index: right }),
            relationship: Relationship::Equal,
            hunk_index: None,
            left_marker: LeftMarker::Unchanged,
            right_marker: RightMarker::Unchanged,
        }
    }

    #[test]
    fn ids_follow_visual_order_not_position() {
        let rows = vec![
            row_with_both(0, 0),
            row_with_both(1, 1),
            row_with_both(2, 2),
        ];
        let right_blocks = vec![b("a", "one"), b("b", "two"), b("c", "three")];
        // Permute: c first, then a, then b.
        let order = vec![2, 0, 1];
        let out = output_right_side_blocks_in_visual_order(
            &rows,
            &order,
            &right_blocks,
            |row_id| Some(right_blocks[row_id].text.clone()),
        );
        assert_eq!(out[0].block_id, "c");
        assert_eq!(out[1].block_id, "a");
        assert_eq!(out[2].block_id, "b");
    }

    #[test]
    fn edited_text_keeps_the_original_id() {
        let rows = vec![row_with_both(0, 0)];
        let right_blocks = vec![b("x", "original")];
        let out = output_right_side_blocks_in_visual_order(
            &rows,
            &[0],
            &right_blocks,
            |_| Some("edited".to_string()),
        );
        assert_eq!(out[0].block_id, "x");
        assert_eq!(out[0].text, "edited");
    }
}
//! Caller pov: pass the frozen rows, the current visual order, and a
//! closure that returns the current text of each row's right side.
//! Returns the right page's blocks in the visual order, top to bottom.
//! LeftOnly rows (rows with no right slot) are skipped — they don't
//! contribute to the output because there was nothing on the right to
//! begin with.
//!
//! Pure: no DOM, no signals, no leptos. The closure is what connects
//! this to the live edit states; without it, this function has no idea
//! about textareas or signals.

use block_text_diff::Row;

pub fn output_right_side_texts_in_visual_order(
    rows: &[Row],
    visual_order: &[usize],
    mut text_for_row: impl FnMut(usize) -> Option<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for &row_id in visual_order {
        if row_id >= rows.len() {
            continue;
        }
        if rows[row_id].right.is_none() {
            continue;
        }
        if let Some(text) = text_for_row(row_id) {
            out.push(text);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use block_text_diff::{LeftMarker, Relationship, RightMarker, Slot};

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
    fn left_only(left: usize) -> Row {
        Row {
            left: Some(Slot { block_index: left }),
            right: None,
            relationship: Relationship::LeftOnly,
            hunk_index: Some(0),
            left_marker: LeftMarker::Deleted,
            right_marker: RightMarker::Unchanged,
        }
    }
    fn right_only(right: usize) -> Row {
        Row {
            left: None,
            right: Some(Slot { block_index: right }),
            relationship: Relationship::RightOnly,
            hunk_index: Some(0),
            left_marker: LeftMarker::Unchanged,
            right_marker: RightMarker::Inserted,
        }
    }

    #[test]
    fn untouched_order_yields_right_page_in_order() {
        let rows = vec![row_with_both(0, 0), row_with_both(1, 1), row_with_both(2, 2)];
        let order = vec![0, 1, 2];
        let texts = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let out = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
            Some(texts[row_id].clone())
        });
        assert_eq!(out, vec!["one", "two", "three"]);
    }

    #[test]
    fn permuted_order_yields_permuted_texts() {
        let rows = vec![row_with_both(0, 0), row_with_both(1, 1), row_with_both(2, 2)];
        let order = vec![2, 0, 1];
        let texts = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let out = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
            Some(texts[row_id].clone())
        });
        assert_eq!(out, vec!["three", "one", "two"]);
    }

    #[test]
    fn left_only_rows_contribute_nothing_wherever_they_sit() {
        let rows = vec![
            row_with_both(0, 0),
            left_only(1),
            row_with_both(2, 1),
            left_only(3),
        ];
        let order = vec![0, 1, 2, 3];
        let texts = vec!["one".to_string(), "two".to_string()];
        let out = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
            match rows[row_id].right {
                Some(slot) => Some(texts[slot.block_index].clone()),
                None => None,
            }
        });
        assert_eq!(out, vec!["one", "two"]);
    }

    #[test]
    fn left_only_rows_contribute_nothing_even_when_permuted_to_the_top() {
        let rows = vec![row_with_both(0, 0), left_only(1), row_with_both(2, 1)];
        let order = vec![1, 0, 2];
        let texts = vec!["one".to_string(), "two".to_string()];
        let out = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
            match rows[row_id].right {
                Some(slot) => Some(texts[slot.block_index].clone()),
                None => None,
            }
        });
        assert_eq!(out, vec!["one", "two"]);
    }

    #[test]
    fn right_only_rows_do_contribute() {
        let rows = vec![right_only(0), row_with_both(0, 1)];
        let order = vec![0, 1];
        let texts = vec!["inserted".to_string(), "matched".to_string()];
        let out = output_right_side_texts_in_visual_order(&rows, &order, |row_id| {
            match rows[row_id].right {
                Some(_) => Some(texts[row_id].clone()),
                None => None,
            }
        });
        assert_eq!(out, vec!["inserted", "matched"]);
    }

    #[test]
    fn empty_rows_give_empty_output() {
        let out = output_right_side_texts_in_visual_order(&[], &[], |_| None);
        assert!(out.is_empty());
    }

    #[test]
    fn empty_order_gives_empty_output() {
        let rows = vec![row_with_both(0, 0)];
        let out = output_right_side_texts_in_visual_order(&rows, &[], |_| Some("x".to_string()));
        assert!(out.is_empty());
    }
}

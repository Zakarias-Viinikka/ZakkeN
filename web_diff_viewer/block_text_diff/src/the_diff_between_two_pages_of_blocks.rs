//! Caller pov: give me two pages of blocks (Vec<String> each) and I'll
//! return the ordered rows that show how they differ. Same shape every
//! time: reading the left slots top to bottom gives every left block
//! once in order; same for right. That invariant is what makes the
//! component's output-equals-the-right-page-in-order property hold.
//!
//! Internals pov: three steps. First, canonicalise blank blocks so any
//! blank equals any other blank (zap's `same_text` rule) and diff the
//! two key sequences with similar's Myers. Second, group every maximal
//! run of non-Equal ops into one hunk and align its slices. Third, walk
//! the alignment: an anchor (paired) step is one row; a run of unpaired
//! steps collapses into max(#dels, #inses) rows, with each row taking
//! one del and one insert if both exist. Every row in a hunk shares the
//! hunk_index. Finally, the moves post-pass swaps Deleted→MovedAway and
//! Inserted→MovedHere on the left-only / right-only rows that turned
//! out to be reordered blocks.

use similar::{Algorithm, DiffOp};

use crate::moves_between_unpaired_blocks_of_text::{
    mark_moves, merge_moves_into_single_rows,
};
use crate::pairs_two_runs_by_similarity::align;
use crate::the_diff_row_and_its_markers::{
    LeftMarker, Relationship, RightMarker, Row, Slot,
};

/// Sentinel key for "this block is blank". Using a value no real block
/// can equal (it contains NULs) rather than a space or an empty string
/// avoids ever colliding with actual content.
const BLANK_KEY: &str = "\u{0}blank\u{0}";

fn canonical_key(text: &str) -> &str {
    if text.trim().is_empty() {
        BLANK_KEY
    } else {
        text
    }
}

pub fn rows_for_two_pages(left: &[String], right: &[String]) -> Vec<Row> {
    let left_keys: Vec<&str> = left.iter().map(|s| canonical_key(s)).collect();
    let right_keys: Vec<&str> = right.iter().map(|s| canonical_key(s)).collect();
    let ops = similar::capture_diff_slices(Algorithm::Myers, &left_keys, &right_keys);

    let mut rows: Vec<Row> = Vec::new();
    let mut hunk_counter: usize = 0;
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DiffOp::Equal { old_index, new_index, len } => {
                for k in 0..*len {
                    rows.push(equal_row(old_index + k, new_index + k));
                }
                i += 1;
            }
            _ => {
                let start = i;
                while i < ops.len() && !matches!(&ops[i], DiffOp::Equal { .. }) {
                    i += 1;
                }
                emit_hunk(&mut rows, &ops[start..i], left, right, hunk_counter);
                hunk_counter += 1;
            }
        }
    }

    mark_moves(&mut rows, left, right);
    merge_moves_into_single_rows(rows)
}

fn equal_row(old_block_index: usize, new_block_index: usize) -> Row {
    Row {
        left: Some(Slot { block_index: old_block_index }),
        right: Some(Slot { block_index: new_block_index }),
        relationship: Relationship::Equal,
        hunk_index: None,
        left_marker: LeftMarker::Unchanged,
        right_marker: RightMarker::Unchanged,
    }
}

fn emit_hunk(
    rows: &mut Vec<Row>,
    ops: &[DiffOp],
    left: &[String],
    right: &[String],
    hunk_index: usize,
) {
    // Ops in a hunk are contiguous by construction, so the slices are
    // just [first.old_index .. last.old_index + last.old_len] and the
    // mirror for new.
    let first = &ops[0];
    let last = &ops[ops.len() - 1];
    let old_start = op_old_index(first);
    let old_end = op_old_index(last) + op_old_len(last);
    let new_start = op_new_index(first);
    let new_end = op_new_index(last) + op_new_len(last);

    let old_texts: Vec<&str> = left[old_start..old_end]
        .iter()
        .map(|s| s.as_str())
        .collect();
    let new_texts: Vec<&str> = right[new_start..new_end]
        .iter()
        .map(|s| s.as_str())
        .collect();

    let steps = align(&old_texts, &new_texts);

    let mut si = 0;
    while si < steps.len() {
        match steps[si] {
            (Some(l), Some(r)) => {
                rows.push(Row {
                    left: Some(Slot { block_index: old_start + l }),
                    right: Some(Slot { block_index: new_start + r }),
                    relationship: Relationship::Edit,
                    hunk_index: Some(hunk_index),
                    left_marker: LeftMarker::Edited,
                    right_marker: RightMarker::Edited,
                });
                si += 1;
            }
            _ => {
                // Collect the entire unpaired run between anchors,
                // separating dels from inses so the collapse rule can
                // pair them rank-for-rank.
                let mut dels: Vec<usize> = Vec::new();
                let mut inses: Vec<usize> = Vec::new();
                while si < steps.len() && !matches!(steps[si], (Some(_), Some(_))) {
                    match steps[si] {
                        (Some(l), None) => dels.push(l),
                        (None, Some(r)) => inses.push(r),
                        _ => unreachable!(),
                    }
                    si += 1;
                }
                let count = dels.len().max(inses.len());
                for r in 0..count {
                    let l_slot = dels
                        .get(r)
                        .map(|&bi| Slot { block_index: old_start + bi });
                    let r_slot = inses
                        .get(r)
                        .map(|&bi| Slot { block_index: new_start + bi });
                    let (relationship, left_marker, right_marker) = match (l_slot, r_slot) {
                        (Some(_), Some(_)) => (
                            Relationship::CollapsedDeleteAndInsert,
                            LeftMarker::Deleted,
                            RightMarker::Inserted,
                        ),
                        (Some(_), None) => (
                            Relationship::LeftOnly,
                            LeftMarker::Deleted,
                            RightMarker::Unchanged,
                        ),
                        (None, Some(_)) => (
                            Relationship::RightOnly,
                            LeftMarker::Unchanged,
                            RightMarker::Inserted,
                        ),
                        (None, None) => unreachable!(),
                    };
                    rows.push(Row {
                        left: l_slot,
                        right: r_slot,
                        relationship,
                        hunk_index: Some(hunk_index),
                        left_marker,
                        right_marker,
                    });
                }
            }
        }
    }
}

fn op_old_index(op: &DiffOp) -> usize {
    match op {
        DiffOp::Equal { old_index, .. } => *old_index,
        DiffOp::Delete { old_index, .. } => *old_index,
        DiffOp::Insert { old_index, .. } => *old_index,
        DiffOp::Replace { old_index, .. } => *old_index,
    }
}
fn op_old_len(op: &DiffOp) -> usize {
    match op {
        DiffOp::Equal { len, .. } => *len,
        DiffOp::Delete { old_len, .. } => *old_len,
        DiffOp::Insert { .. } => 0,
        DiffOp::Replace { old_len, .. } => *old_len,
    }
}
fn op_new_index(op: &DiffOp) -> usize {
    match op {
        DiffOp::Equal { new_index, .. } => *new_index,
        DiffOp::Delete { new_index, .. } => *new_index,
        DiffOp::Insert { new_index, .. } => *new_index,
        DiffOp::Replace { new_index, .. } => *new_index,
    }
}
fn op_new_len(op: &DiffOp) -> usize {
    match op {
        DiffOp::Equal { len, .. } => *len,
        DiffOp::Delete { .. } => 0,
        DiffOp::Insert { new_len, .. } => *new_len,
        DiffOp::Replace { new_len, .. } => *new_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows_of(left: &[&str], right: &[&str]) -> Vec<Row> {
        let l: Vec<String> = left.iter().map(|s| s.to_string()).collect();
        let r: Vec<String> = right.iter().map(|s| s.to_string()).collect();
        rows_for_two_pages(&l, &r)
    }

    fn lefts(rows: &[Row]) -> Vec<usize> {
        rows.iter()
            .filter_map(|r| r.left.map(|s| s.block_index))
            .collect()
    }
    fn rights(rows: &[Row]) -> Vec<usize> {
        rows.iter()
            .filter_map(|r| r.right.map(|s| s.block_index))
            .collect()
    }

    fn assert_ordering(rows: &[Row], left_len: usize, right_len: usize) {
        assert_eq!(
            rights(rows),
            (0..right_len).collect::<Vec<_>>(),
            "right order broke"
        );
        let has_moves = rows.iter().any(|r| r.relationship == Relationship::Moved);
        if !has_moves {
            assert_eq!(
                lefts(rows),
                (0..left_len).collect::<Vec<_>>(),
                "left order broke (but no moves detected)"
            );
        } else {
            // A move relocates the left slot to the right position, so
            // the left slots are no longer in original order past a
            // move. They must still be a permutation of 0..left_len.
            let mut sorted = lefts(rows);
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                (0..left_len).collect::<Vec<_>>(),
                "left slots not a permutation"
            );
        }
    }

    fn kinds(rows: &[Row]) -> Vec<&'static str> {
        rows.iter()
            .map(|r| match r.relationship {
                Relationship::Equal => "equal",
                Relationship::Edit => "edit",
                Relationship::CollapsedDeleteAndInsert => "collapsed",
                Relationship::LeftOnly => "left",
                Relationship::RightOnly => "right",
                Relationship::Moved => "moved",
            })
            .collect()
    }

    #[test]
    fn identical_pages_produce_all_equal_rows_and_no_hunks() {
        let rows = rows_of(&["a", "b", "c"], &["a", "b", "c"]);
        assert_eq!(kinds(&rows), vec!["equal", "equal", "equal"]);
        assert!(rows.iter().all(|r| r.hunk_index.is_none()));
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn similar_replacement_gives_one_edit_row() {
        let rows = rows_of(&["intro", "color", "outro"], &["intro", "colour", "outro"]);
        assert_eq!(kinds(&rows), vec!["equal", "edit", "equal"]);
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn dissimilar_replacement_gives_one_collapsed_row() {
        let rows = rows_of(
            &["intro", "banana", "outro"],
            &["intro", "kangaroo", "outro"],
        );
        assert_eq!(kinds(&rows), vec!["equal", "collapsed", "equal"]);
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn four_deleted_vs_two_unrelated_inserted_gives_four_rows() {
        let rows = rows_of(
            &["D1", "D2", "D3", "D4"],
            &[
                "I1 is a long standalone paragraph",
                "I2 is another long standalone paragraph",
            ],
        );
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].relationship, Relationship::CollapsedDeleteAndInsert);
        assert_eq!(rows[1].relationship, Relationship::CollapsedDeleteAndInsert);
        assert_eq!(rows[2].relationship, Relationship::LeftOnly);
        assert_eq!(rows[3].relationship, Relationship::LeftOnly);
        assert_eq!(rows[2].right, None);
        assert_eq!(rows[3].right, None);
        assert_ordering(&rows, 4, 2);
    }

    #[test]
    fn stack_then_edit_keeps_the_edit_aligned() {
        let rows = rows_of(
            &[
                "Shared opener",
                "X is a completely unrelated line",
                "Y is another completely unrelated line",
                "The ending that pairs well",
            ],
            &["Shared opener", "The ending that pairs well indeed"],
        );
        assert_eq!(
            kinds(&rows),
            vec!["equal", "left", "left", "edit"],
            "rows: {:?}",
            kinds(&rows)
        );
        assert_ordering(&rows, 4, 2);
    }

    #[test]
    fn stack_then_edit_with_more_inserts_than_deletes() {
        let rows = rows_of(
            &["Shared opener", "The ending that pairs well indeed"],
            &[
                "Shared opener",
                "I1 is a completely unrelated line",
                "I2 is another completely unrelated line",
                "The ending that pairs well",
            ],
        );
        assert_eq!(
            kinds(&rows),
            vec!["equal", "right", "right", "edit"],
            "rows: {:?}",
            kinds(&rows)
        );
        assert_ordering(&rows, 2, 4);
    }

    #[test]
    fn hunk_index_is_shared_within_and_different_between() {
        let rows = rows_of(
            &[
                "alpha opener",
                "banana",
                "middle",
                "cherry",
                "omega closer",
            ],
            &[
                "alpha opener",
                "kangaroo",
                "middle",
                "strawberry",
                "omega closer",
            ],
        );
        assert_eq!(
            kinds(&rows),
            vec!["equal", "collapsed", "equal", "collapsed", "equal"]
        );
        assert_eq!(rows[0].hunk_index, None);
        assert_eq!(rows[1].hunk_index, Some(0));
        assert_eq!(rows[2].hunk_index, None);
        assert_eq!(rows[3].hunk_index, Some(1));
        assert_eq!(rows[4].hunk_index, None);
    }

    #[test]
    fn empty_left_gives_only_right_rows() {
        let rows = rows_of(&[], &["a", "b"]);
        assert_eq!(kinds(&rows), vec!["right", "right"]);
        assert_ordering(&rows, 0, 2);
    }

    #[test]
    fn empty_right_gives_only_left_rows() {
        let rows = rows_of(&["a", "b"], &[]);
        assert_eq!(kinds(&rows), vec!["left", "left"]);
        assert_ordering(&rows, 2, 0);
    }

    #[test]
    fn both_empty_gives_no_rows() {
        let rows = rows_of(&[], &[]);
        assert!(rows.is_empty());
    }

    #[test]
    fn blank_blocks_equal_each_other_regardless_of_whitespace() {
        let rows = rows_of(&["a", "", "b"], &["a", "   ", "b"]);
        assert_eq!(kinds(&rows), vec!["equal", "equal", "equal"]);
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn blank_does_not_equal_non_blank() {
        let rows = rows_of(&["a", "", "b"], &["a", "hello there", "b"]);
        assert_eq!(kinds(&rows), vec!["equal", "collapsed", "equal"]);
        assert_eq!(rows[1].left.unwrap().block_index, 1);
        assert_eq!(rows[1].right.unwrap().block_index, 1);
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn insertion_in_the_middle_keeps_edges_paired() {
        let rows = rows_of(
            &["one", "two", "three"],
            &["one", "two", "inserted block here now", "three"],
        );
        assert_eq!(kinds(&rows), vec!["equal", "equal", "right", "equal"]);
        assert_ordering(&rows, 3, 4);
    }

    #[test]
    fn delete_and_insert_interleaved_still_reads_in_order() {
        let rows = rows_of(
            &["a", "gone-away", "b"],
            &["a", "arrived-here", "b"],
        );
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn unicode_content_does_not_break_the_walk() {
        let rows = rows_of(
            &["åäö först", "mellan", "sist"],
            &["åäö först", "annorlunda", "sist"],
        );
        assert_ordering(&rows, 3, 3);
    }

    #[test]
    fn reordered_long_paragraph_gets_moved_markers() {
        let a = "Alpha paragraph that starts the document and provides context.";
        let b = "Bravo paragraph that describes something in detail, long enough for a move.";
        let c = "Charlie paragraph that appears after the second one, also long enough.";
        // Reorder: B moves from middle to end (equivalently, C moves
        // from end to middle — the diff is symmetric). Myers has two
        // minimal paths here, "Delete B then Insert B" or "Insert C
        // then Delete C", and either is valid; similar picks one based
        // on its internal tie-breaking. So this test asserts only that
        // a move was detected and that the two markers name each
        // other's block indices, not which of the two blocks was
        // labelled as the mover.
        let rows = rows_of(&[a, b, c], &[a, c, b]);
        assert_ordering(&rows, 3, 3);

        let moved_rows: Vec<&Row> = rows
            .iter()
            .filter(|r| r.relationship == Relationship::Moved)
            .collect();
        assert_eq!(moved_rows.len(), 1, "expected exactly one Moved row");
        let m = moved_rows[0];
        let left_block = m.left.expect("Moved row without a left slot").block_index;
        let right_block = m.right.expect("Moved row without a right slot").block_index;
        assert!(left_block != right_block, "expected the moved block to differ");

        let (left_partner, left_identical) = match m.left_marker {
            LeftMarker::MovedAway {
                partner_right_block_index,
                identical,
            } => (partner_right_block_index, identical),
            _ => panic!("Moved row has a non-MovedAway left marker"),
        };
        let (right_partner, right_identical) = match m.right_marker {
            RightMarker::MovedHere {
                partner_left_block_index,
                identical,
            } => (partner_left_block_index, identical),
            _ => panic!("Moved row has a non-MovedHere right marker"),
        };
        assert_eq!(left_partner, right_block);
        assert_eq!(right_partner, left_block);
        assert!(left_identical);
        assert!(right_identical);
    }

    #[test]
    fn property_left_and_right_slot_sequences_are_permutations() {
        let mut state: u64 = 0x9e3779b97f4a7c15;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        let pool = [
            "alpha one two three",
            "banana split with cream",
            "cherry on top of cake",
            "date palm by the sea",
            "elderberry jam today",
            "fig leaf in the wind",
        ];
        for _ in 0..50 {
            let left_len = (next() % 6) as usize;
            let right_len = (next() % 6) as usize;
            let left: Vec<&str> = (0..left_len)
                .map(|_| pool[(next() as usize) % pool.len()])
                .collect();
            let right: Vec<&str> = (0..right_len)
                .map(|_| pool[(next() as usize) % pool.len()])
                .collect();
            let rows = rows_of(&left, &right);
            assert_ordering(&rows, left_len, right_len);
        }
    }
}
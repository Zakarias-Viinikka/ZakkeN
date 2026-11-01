//! Caller pov: pass me the rows (already laid out) plus the two pages of
//! text, and I'll swap Deleted→MovedAway and Inserted→MovedHere on the
//! left-only / right-only rows that turned out to be moves rather than
//! genuine delete+insert. Row layout doesn't change.
//!
//! Internals pov: exact-text matches come first (a reordered paragraph
//! is the common case) using a hash map. Remaining candidates go through
//! fuzzy token similarity. Size gates keep tiny "yes"/"}"-style blocks
//! from being called moves. Matching is one-to-one and greedy, best
//! score first.
//!
//! A `CollapsedDeleteAndInsert` row is a real pairing made by the
//! collapse rule, not a candidate for a move. The plan explicitly
//! accepts that the collapse rule can pair unrelated texts, and the fix
//! (if it ever feels wrong) would be splitting collapsed rows at drag
//! start — not here.

use std::collections::HashMap;

use crate::similarity_between_two_blocks_of_text::token_similarity;
use crate::the_diff_row_and_its_markers::{
    LeftMarker, Relationship, RightMarker, Row,
};

/// A block with fewer non-whitespace characters than this can't be
/// classified as an exact-text move. Stops "yes", "}", "ok" and other
/// tiny one-word blocks from being called moves just because the same
/// word appears twice.
pub const EXACT_MOVE_MIN_NON_WS_CHARS: usize = 8;

/// Stricter gate for fuzzy moves, because fuzzy matching is noisier
/// than exact equality.
pub const FUZZY_MOVE_MIN_NON_WS_CHARS: usize = 24;

/// Fuzzy similarity floor. Below this, the pair isn't considered a
/// move even if both sides pass the size gate.
pub const FUZZY_MOVE_MIN_SCORE: f32 = 0.75;

/// Cap on D×I fuzzy candidate pairs. Above this, only exact matches run
/// — the quadratic cost isn't worth it for pathological pages.
pub const FUZZY_PAIR_CAP: usize = 400;

pub fn mark_moves(rows: &mut [Row], left: &[String], right: &[String]) {
    // Collect candidates from the laid-out rows. Each entry is
    // (row_index, text). Only LeftOnly and RightOnly rows are
    // candidates; a CollapsedDeleteAndInsert row is already paired.
    let mut left_candidates: Vec<(usize, &str)> = Vec::new();
    let mut right_candidates: Vec<(usize, &str)> = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        if r.relationship == Relationship::LeftOnly {
            if let Some(slot) = r.left {
                if let Some(text) = left.get(slot.block_index) {
                    left_candidates.push((i, text.as_str()));
                }
            }
        } else if r.relationship == Relationship::RightOnly {
            if let Some(slot) = r.right {
                if let Some(text) = right.get(slot.block_index) {
                    right_candidates.push((i, text.as_str()));
                }
            }
        }
    }
    if left_candidates.is_empty() || right_candidates.is_empty() {
        return;
    }

    let mut left_used = vec![false; left_candidates.len()];
    let mut right_used = vec![false; right_candidates.len()];
    // (left_candidates index, right_candidates index, identical)
    let mut matches: Vec<(usize, usize, bool)> = Vec::new();

    // Pass 1: exact text equality, gated by size.
    let mut exact_map: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, (_, text)) in left_candidates.iter().enumerate() {
        if non_ws_chars(text) >= EXACT_MOVE_MIN_NON_WS_CHARS {
            exact_map.entry(*text).or_default().push(i);
        }
    }
    for (j, (_, text)) in right_candidates.iter().enumerate() {
        if non_ws_chars(text) < EXACT_MOVE_MIN_NON_WS_CHARS {
            continue;
        }
        if let Some(bucket) = exact_map.get_mut(*text) {
            if let Some(i) = bucket.pop() {
                left_used[i] = true;
                right_used[j] = true;
                matches.push((i, j, true));
            }
        }
    }

    // Pass 2: fuzzy. Only if under the cap.
    let remaining_lefts: Vec<usize> = (0..left_candidates.len())
        .filter(|&i| !left_used[i])
        .collect();
    let remaining_rights: Vec<usize> = (0..right_candidates.len())
        .filter(|&j| !right_used[j])
        .collect();
    if !remaining_lefts.is_empty()
        && !remaining_rights.is_empty()
        && remaining_lefts.len() * remaining_rights.len() <= FUZZY_PAIR_CAP
    {
        let mut fuzzy_candidates: Vec<(f32, usize, usize)> = Vec::new();
        for &i in &remaining_lefts {
            if non_ws_chars(left_candidates[i].1) < FUZZY_MOVE_MIN_NON_WS_CHARS {
                continue;
            }
            for &j in &remaining_rights {
                if non_ws_chars(right_candidates[j].1) < FUZZY_MOVE_MIN_NON_WS_CHARS {
                    continue;
                }
                let score = token_similarity(left_candidates[i].1, right_candidates[j].1);
                if score >= FUZZY_MOVE_MIN_SCORE {
                    fuzzy_candidates.push((score, i, j));
                }
            }
        }
        fuzzy_candidates
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, i, j) in fuzzy_candidates {
            if left_used[i] || right_used[j] {
                continue;
            }
            left_used[i] = true;
            right_used[j] = true;
            let identical = left_candidates[i].1 == right_candidates[j].1;
            matches.push((i, j, identical));
        }
    }

    // Apply to rows.
    for (i, j, identical) in matches {
        let left_row_idx = left_candidates[i].0;
        let right_row_idx = right_candidates[j].0;
        let left_block_index = rows[left_row_idx].left.unwrap().block_index;
        let right_block_index = rows[right_row_idx].right.unwrap().block_index;
        rows[left_row_idx].left_marker = LeftMarker::MovedAway {
            partner_right_block_index: right_block_index,
            identical,
        };
        rows[right_row_idx].right_marker = RightMarker::MovedHere {
            partner_left_block_index: left_block_index,
            identical,
        };
    }
}

fn non_ws_chars(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

/// Second pass: collapse each pair of LeftOnly (marked MovedAway) and
/// RightOnly (marked MovedHere) rows into a single `Moved` row. The
/// merged row is emitted at the RightOnly row's original position — the
/// "landed" position — and its LeftOnly partner is skipped. The right
/// slots stay in original order (what the output function relies on);
/// the left slots go out of order past a move, the same trade the plan
/// already accepts for drags.
pub fn merge_moves_into_single_rows(rows: Vec<Row>) -> Vec<Row> {
    use std::collections::HashMap;

    let right_block_to_row: HashMap<usize, usize> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            if r.relationship == Relationship::RightOnly {
                r.right.map(|s| (s.block_index, i))
            } else {
                None
            }
        })
        .collect();

    let mut left_row_to_right_row: HashMap<usize, usize> = HashMap::new();
    let mut right_row_to_left_row: HashMap<usize, usize> = HashMap::new();
    for (i, row) in rows.iter().enumerate() {
        if let LeftMarker::MovedAway { partner_right_block_index, .. } = row.left_marker {
            if let Some(&j) = right_block_to_row.get(&partner_right_block_index) {
                left_row_to_right_row.insert(i, j);
                right_row_to_left_row.insert(j, i);
            }
        }
    }
    if left_row_to_right_row.is_empty() {
        return rows;
    }

    let mut merged_for_right: HashMap<usize, Row> = HashMap::new();
    for (&j, &i) in right_row_to_left_row.iter() {
        let left_row = &rows[i];
        let right_row = &rows[j];
        let (left_block_index, identical) = match left_row.left_marker {
            LeftMarker::MovedAway { identical, .. } => {
                (left_row.left.map(|s| s.block_index).unwrap_or(0), identical)
            }
            _ => continue,
        };
        let right_block_index = right_row.right.map(|s| s.block_index).unwrap_or(0);
        merged_for_right.insert(
            j,
            Row {
                left: left_row.left,
                right: right_row.right,
                relationship: Relationship::Moved,
                hunk_index: right_row.hunk_index,
                left_marker: LeftMarker::MovedAway {
                    partner_right_block_index: right_block_index,
                    identical,
                },
                right_marker: RightMarker::MovedHere {
                    partner_left_block_index: left_block_index,
                    identical,
                },
            },
        );
    }

    let mut out: Vec<Row> = Vec::with_capacity(rows.len() - left_row_to_right_row.len());
    for (i, row) in rows.into_iter().enumerate() {
        if left_row_to_right_row.contains_key(&i) {
            continue;
        }
        if let Some(merged) = merged_for_right.remove(&i) {
            out.push(merged);
        } else {
            out.push(row);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::the_diff_row_and_its_markers::Slot;

    fn left_only(block_index: usize) -> Row {
        Row {
            left: Some(Slot { block_index }),
            right: None,
            relationship: Relationship::LeftOnly,
            hunk_index: Some(0),
            left_marker: LeftMarker::Deleted,
            right_marker: RightMarker::Unchanged,
        }
    }
    fn right_only(block_index: usize) -> Row {
        Row {
            left: None,
            right: Some(Slot { block_index }),
            relationship: Relationship::RightOnly,
            hunk_index: Some(1),
            left_marker: LeftMarker::Unchanged,
            right_marker: RightMarker::Inserted,
        }
    }

    #[test]
    fn exact_match_marks_both_sides_as_moves() {
        let mut rows = vec![left_only(0), right_only(0)];
        let long = "A paragraph long enough to pass the exact size gate.";
        let left = vec![long.to_string()];
        let right = vec![long.to_string()];
        mark_moves(&mut rows, &left, &right);
        assert!(matches!(
            rows[0].left_marker,
            LeftMarker::MovedAway { partner_right_block_index: 0, identical: true }
        ));
        assert!(matches!(
            rows[1].right_marker,
            RightMarker::MovedHere { partner_left_block_index: 0, identical: true }
        ));
    }

    #[test]
    fn short_blocks_are_not_marked_as_moves() {
        let mut rows = vec![left_only(0), right_only(0)];
        let left = vec!["yes".to_string()];
        let right = vec!["yes".to_string()];
        mark_moves(&mut rows, &left, &right);
        assert_eq!(rows[0].left_marker, LeftMarker::Deleted);
        assert_eq!(rows[1].right_marker, RightMarker::Inserted);
    }

    #[test]
    fn fuzzy_match_for_edited_move() {
        let mut rows = vec![left_only(0), right_only(0)];
        let left = vec![
            "The quick brown fox jumps over the lazy dog and keeps on running.".to_string(),
        ];
        let right = vec![
            "The quick brown fox jumps over the lazy cat and keeps on running.".to_string(),
        ];
        mark_moves(&mut rows, &left, &right);
        assert!(matches!(
            rows[0].left_marker,
            LeftMarker::MovedAway { partner_right_block_index: 0, identical: false }
        ));
        assert!(matches!(
            rows[1].right_marker,
            RightMarker::MovedHere { partner_left_block_index: 0, identical: false }
        ));
    }

    #[test]
    fn fuzzy_below_threshold_is_ignored() {
        let mut rows = vec![left_only(0), right_only(0)];
        let left = vec![
            "The quick brown fox jumps over the lazy dog and keeps on running.".to_string(),
        ];
        let right = vec![
            "A radically different paragraph with no overlap whatsoever at all.".to_string(),
        ];
        mark_moves(&mut rows, &left, &right);
        assert_eq!(rows[0].left_marker, LeftMarker::Deleted);
        assert_eq!(rows[1].right_marker, RightMarker::Inserted);
    }

    #[test]
    fn paired_rows_are_never_touched() {
        let mut rows = vec![Row {
            left: Some(Slot { block_index: 0 }),
            right: Some(Slot { block_index: 0 }),
            relationship: Relationship::Edit,
            hunk_index: Some(0),
            left_marker: LeftMarker::Edited,
            right_marker: RightMarker::Edited,
        }];
        let text = "A paragraph long enough to pass any size gate at all.".to_string();
        let left = vec![text.clone()];
        let right = vec![text];
        mark_moves(&mut rows, &left, &right);
        assert_eq!(rows[0].left_marker, LeftMarker::Edited);
        assert_eq!(rows[0].right_marker, RightMarker::Edited);
    }

    #[test]
    fn one_to_one_matching_takes_the_exact_first() {
        // Two lefts with the same text, one right with that text. Only
        // one left can be marked.
        let mut rows = vec![
            left_only(0),
            left_only(0),
            right_only(0),
        ];
        let text = "A long paragraph long enough to pass the exact size gate.".to_string();
        let left = vec![text.clone(), text.clone()];
        let right = vec![text];
        mark_moves(&mut rows, &left, &right);
        let marked = rows
            .iter()
            .filter(|r| matches!(r.left_marker, LeftMarker::MovedAway { .. }))
            .count();
        assert_eq!(marked, 1);
        assert!(matches!(
            rows[2].right_marker,
            RightMarker::MovedHere { .. }
        ));
    }

    #[test]
    fn two_exact_pairs_mark_both() {
        let mut rows = vec![
            left_only(0),
            right_only(0),
            left_only(1),
            right_only(1),
        ];
        let a = "First paragraph that is long enough for the exact gate.".to_string();
        let b = "Second paragraph that is long enough for the exact gate.".to_string();
        let left = vec![a.clone(), b.clone()];
        let right = vec![b, a];
        mark_moves(&mut rows, &left, &right);
        // Every candidate pair is marked because both texts are in both
        // sides.
        let left_marks = rows
            .iter()
            .filter(|r| matches!(r.left_marker, LeftMarker::MovedAway { .. }))
            .count();
        let right_marks = rows
            .iter()
            .filter(|r| matches!(r.right_marker, RightMarker::MovedHere { .. }))
            .count();
        assert_eq!(left_marks, 2);
        assert_eq!(right_marks, 2);
    }

    #[test]
    fn empty_pages_return_early_without_changes() {
        let mut rows = vec![left_only(0)];
        mark_moves(&mut rows, &["hi".to_string()], &[]);
        assert_eq!(rows[0].left_marker, LeftMarker::Deleted);
    }
}

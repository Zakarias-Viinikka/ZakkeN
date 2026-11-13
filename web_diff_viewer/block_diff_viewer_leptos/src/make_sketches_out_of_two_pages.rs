//! Caller pov: turn two pages — the server's truth and the client's
//! kept side in its current visual order — into love-letter sketches.
//! Used by `DiffViewer` on finish, and available to the caller directly
//! for pairs that were never opened (e.g. auto-picked by a default).
//!
//! Internals pov: alive filter, then LIS-based frac assignment (so only
//! blocks that are out of order get new fracs), then
//! `generate_sketches`. Pure; no DOM, no signals, no leptos.
//!
//! `generate_sketches` errors on duplicate block ids. That means the
//! caller handed in malformed pages; it's a programming bug, so this
//! module panics rather than returning a Result. The letter-builder's
//! self-check downstream is the real safety net for bad heuristics.

use std::collections::{HashMap, HashSet};

use generate_love_letters::{generate_sketches, Block, LoveLetterSketch};
use smart_frac_index::frac_core::{key_after, key_at_start, key_before, key_between};

use crate::the_public_types::PageBlocksForTheViewer;

pub fn make_sketches_out_of_two_pages(
    server_page: &PageBlocksForTheViewer,
    kept_blocks: &[Block],
) -> Vec<LoveLetterSketch> {
    let alive = alive_filter(server_page, kept_blocks);
    let alive_with_fracs = assign_fracs_so_only_out_of_order_blocks_move(server_page, &alive);
    generate_sketches(&server_page.page_id, &server_page.blocks, &alive_with_fracs)
        .expect("generate_sketches: duplicate block id (should be impossible)")
}

/// Drop blocks that don't survive the reconcile: a kept block is alive
/// unless it's a server block that was blanked (kept text empty, server
/// text non-empty), or a brand-new block with empty text. Non-blank
/// kept blocks with no server counterpart are alive (creates).
fn alive_filter(server_page: &PageBlocksForTheViewer, kept_blocks: &[Block]) -> Vec<Block> {
    let server_by_id: HashMap<&str, &Block> = server_page
        .blocks
        .iter()
        .map(|b| (b.block_id.as_str(), b))
        .collect();
    let title_id: Option<&str> = server_page
        .blocks
        .first()
        .map(|b| b.block_id.as_str());

    kept_blocks
        .iter()
        .filter(|kb| {
            if Some(kb.block_id.as_str()) == title_id {
                return true;
            }
            match server_by_id.get(kb.block_id.as_str()) {
                Some(server_block) => {
                    !(kb.text.is_empty() && !server_block.text.is_empty())
                }
                None => !kb.text.is_empty(),
            }
        })
        .cloned()
        .collect()
}

/// New fracs only for survivors whose position among the other survivors
/// isn't in the longest strictly-increasing subsequence of their server
/// indices. Everything else keeps its server frac. New blocks (not on
/// the server) keep their placeholder frac — `generate_sketches` ignores
/// it for `CreateNewBlockWithId`.
fn assign_fracs_so_only_out_of_order_blocks_move(
    server_page: &PageBlocksForTheViewer,
    alive: &[Block],
) -> Vec<Block> {
    let server_index: HashMap<&str, usize> = server_page
        .blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.block_id.as_str(), i))
        .collect();

    let survivor_positions: Vec<(usize, usize)> = alive
        .iter()
        .enumerate()
        .filter_map(|(i, b)| server_index.get(b.block_id.as_str()).map(|&si| (i, si)))
        .collect();

    let survivor_seq: Vec<usize> = survivor_positions.iter().map(|(_, si)| *si).collect();
    let lis_indices: Vec<usize> = longest_strictly_increasing_subsequence(&survivor_seq);
    let stable_alive_indices: HashSet<usize> = lis_indices
        .into_iter()
        .map(|i| survivor_positions[i].0)
        .collect();

    let mut result: Vec<Block> = alive.to_vec();
    let mut last_survivor_frac: Option<String> = None;

    for i in 0..alive.len() {
        let block = &alive[i];
        let Some(&si) = server_index.get(block.block_id.as_str()) else {
            continue;
        };
        if stable_alive_indices.contains(&i) {
            result[i].frac_position = server_page.blocks[si].frac_position.clone();
            last_survivor_frac = Some(result[i].frac_position.clone());
        } else {
            let right_frac: Option<String> = (i + 1..alive.len())
                .find(|&j| stable_alive_indices.contains(&j))
                .and_then(|j| {
                    server_index
                        .get(alive[j].block_id.as_str())
                        .map(|&s| server_page.blocks[s].frac_position.clone())
                });
            result[i].frac_position = match (last_survivor_frac.clone(), right_frac) {
                (Some(l), Some(r)) => {
                    key_between(&l, &r).expect("smart_frac_index::key_between failed")
                }
                (Some(l), None) => {
                    key_after(&l).expect("smart_frac_index::key_after failed")
                }
                (None, Some(r)) => {
                    key_before(&r).expect("smart_frac_index::key_before failed")
                }
                (None, None) => key_at_start(),
            };
            last_survivor_frac = Some(result[i].frac_position.clone());
        }
    }
    result
}

/// Patience sorting with predecessor links. Returns the indices into
/// `seq` that form the longest strictly-increasing subsequence.
fn longest_strictly_increasing_subsequence(seq: &[usize]) -> Vec<usize> {
    let n = seq.len();
    if n == 0 {
        return Vec::new();
    }
    let mut tails: Vec<usize> = Vec::new();
    let mut prev: Vec<Option<usize>> = vec![None; n];
    for i in 0..n {
        let x = seq[i];
        let pos = tails
            .iter()
            .position(|&j| seq[j] >= x)
            .unwrap_or(tails.len());
        if pos > 0 {
            prev[i] = Some(tails[pos - 1]);
        }
        if pos == tails.len() {
            tails.push(i);
        } else {
            tails[pos] = i;
        }
    }
    let mut result = Vec::new();
    let mut cur = tails.last().copied();
    while let Some(i) = cur {
        result.push(i);
        cur = prev[i];
    }
    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(id: &str, text: &str, frac: &str) -> Block {
        Block {
            block_id: id.to_string(),
            text: text.to_string(),
            metadata: String::new(),
            frac_position: frac.to_string(),
        }
    }

    fn page(id: &str, blocks: Vec<Block>) -> PageBlocksForTheViewer {
        PageBlocksForTheViewer {
            page_id: id.to_string(),
            is_main_menu_page: false,
            blocks,
        }
    }

    #[test]
    fn identical_pages_produce_zero_sketches() {
        let server = page(
            "p",
            vec![block("a", "hello", "80"), block("b", "world", "81")],
        );
        let kept = server.blocks.clone();
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        assert!(sketches.is_empty());
    }

    #[test]
    fn text_edit_produces_one_edit_block() {
        let server = page(
            "p",
            vec![block("a", "hello", "80"), block("b", "world", "81")],
        );
        let kept = vec![block("a", "hi", "80"), block("b", "world", "81")];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        assert_eq!(sketches.len(), 1);
        assert!(matches!(
            sketches[0],
            LoveLetterSketch::EditBlock { .. }
        ));
    }

    #[test]
    fn swapped_two_blocks_produces_at_least_one_move() {
        let server = page(
            "p",
            vec![block("a", "hello", "80"), block("b", "world", "81")],
        );
        let kept = vec![block("b", "world", "81"), block("a", "hello", "80")];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        let moves = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::MoveBlock { .. }))
            .count();
        assert!(moves >= 1);
    }

    #[test]
    fn new_block_produces_create_new_block_with_id() {
        let server = page("p", vec![block("a", "hello", "80")]);
        let kept = vec![block("a", "hello", "80"), block("new", "surprise", "")];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        let creates = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::CreateNewBlockWithId { .. }))
            .count();
        assert_eq!(creates, 1);
    }

    #[test]
    fn blank_kept_text_on_non_empty_server_block_produces_remove() {
        let server = page("p", vec![block("a", "hello", "80")]);
        let kept = vec![block("a", "", "80")];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        let removes = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::RemoveBlock { .. }))
            .count();
        assert_eq!(removes, 1);
    }

    #[test]
    fn blank_kept_text_on_blank_server_block_is_a_no_op() {
        let server = page("p", vec![block("a", "", "80")]);
        let kept = vec![block("a", "", "80")];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        assert!(sketches.is_empty());
    }

    #[test]
    fn blank_kept_text_on_the_first_block_is_not_a_remove() {
        let server = page(
            "p",
            vec![block("title", "My Title", "80"), block("body", "hello", "81")],
        );
        let kept = vec![
            block("title", "", "80"),
            block("body", "hello", "81"),
        ];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        let removes = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::RemoveBlock { .. }))
            .count();
        assert_eq!(removes, 0, "title must never be removed");
    }

    #[test]
    fn move_one_block_to_top_produces_one_move() {
        let server = page(
            "p",
            vec![
                block("a", "one", "80"),
                block("b", "two", "81"),
                block("c", "three", "82"),
            ],
        );
        let kept = vec![
            block("c", "three", "82"),
            block("a", "one", "80"),
            block("b", "two", "81"),
        ];
        let sketches = make_sketches_out_of_two_pages(&server, &kept);
        let moves = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::MoveBlock { .. }))
            .count();
        assert_eq!(moves, 1);
    }

    #[test]
    fn lis_of_empty_is_empty() {
        assert!(longest_strictly_increasing_subsequence(&[]).is_empty());
    }

    #[test]
    fn lis_of_sorted_returns_all() {
        let seq = [0usize, 1, 2, 3];
        let out = longest_strictly_increasing_subsequence(&seq);
        assert_eq!(out, vec![0, 1, 2, 3]);
    }

    #[test]
    fn lis_of_reversed_takes_one() {
        let seq = [3usize, 2, 1, 0];
        let out = longest_strictly_increasing_subsequence(&seq);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn lis_of_mixed() {
        // Standard example: [10, 9, 2, 5, 3, 7, 101, 18] -> [2, 3, 7, 18]
        let seq = [10usize, 9, 2, 5, 3, 7, 101, 18];
        let out = longest_strictly_increasing_subsequence(&seq);
        // LIS length is 4.
        assert_eq!(out.len(), 4);
        // Indices 2, 4, 5, 7 in the original sequence.
        let values: Vec<usize> = out.iter().map(|&i| seq[i]).collect();
        assert_eq!(values, vec![2, 3, 7, 18]);
    }
}

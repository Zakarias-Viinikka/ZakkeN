//! Caller pov: turn one page's edits (the viewer's sketches) into full
//! love-letter values, by applying them to a scratch copy of the
//! server's page. Returns the letters. No self-check — correctness is
//! covered by tests.
//!
//! Two passes because creates can't be placed until survivors have
//! their final positions:
//! - Pass 1: edits, deletes, moves. Order among them doesn't matter.
//! - Pass 2: creates, in target order. Each create is placed relative
//!   to the target block before it (which exists — either a survivor or
//!   an earlier create). Insert uses the target id as the doc id, so
//!   the resulting letter stays `CreateNewBlockWithId`.

use std::collections::HashSet;
use std::sync::Arc;

use generate_love_letters::Block;
use love_letter::{LoveLetter, LoveLetterSketch};
use my_yrs_lib::BossOfYrs;

use super::scratch_page::{
    open_a_scratch_copy_of_the_server_page, position_for_insert,
};
use super::turn_one_change_into_a_love_letter::{
    edit_to_love_letter, insert_variant_to_love_letter,
};

pub fn turn_all_edits_into_love_letters(
    server_page: Arc<BossOfYrs>,
    edits: &[LoveLetterSketch],
    target_blocks: &[Block],
) -> Result<Vec<LoveLetter>, String> {
    let page_id = server_page.clone().page_id();
    let scratch = open_a_scratch_copy_of_the_server_page(server_page)?;

    let mut letters = Vec::new();

    for edit in edits {
        if matches!(
            edit,
            LoveLetterSketch::CreateNewBlockWithId { .. }
                | LoveLetterSketch::CreateNewBlock { .. }
        ) {
            continue;
        }
        letters.push(edit_to_love_letter(&scratch, &page_id, edit)?);
    }

    let create_ids: HashSet<&str> = edits
        .iter()
        .filter_map(|e| match e {
            LoveLetterSketch::CreateNewBlockWithId { block_id, .. } => {
                Some(block_id.as_str())
            }
            _ => None,
        })
        .collect();

    for (i, block) in target_blocks.iter().enumerate() {
        if !create_ids.contains(block.block_id.as_str()) {
            continue;
        }
        // Every create has a previous block in target order: the title
        // sits at index 0 and can never be deleted, so no create lands
        // at index 0.
        let above_id: &str = target_blocks
            .get(i.wrapping_sub(1))
            .map(|b| b.block_id.as_str())
            .expect("create at index 0 — the title block must be preserved");
        let position = position_for_insert(&scratch, Some(above_id))?;
        letters.push(insert_variant_to_love_letter(
            &scratch,
            &page_id,
            &block.block_id,
            block.text.clone(),
            position,
        )?);
    }

    Ok(letters)
}

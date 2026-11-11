//! Caller pov: `create_love_letters_out_of_reconcile(server_page,
//! target_blocks)` produces the love letters that turn the server's
//! page into the target block list. The target is only texts — no ids —
//! because that's what the diff viewer outputs.
//!
//! Internals pov: load the server's page into a fresh scratch doc with
//! a unique actor name, align the initial and target block texts, walk
//! the hunks left to right turning each into a letter and applying each
//! to the scratch doc so later letters build on earlier ones, then
//! self-check that the scratch doc's texts equal the target. If the
//! self-check fails, return an error and send nothing.

use std::sync::Arc;

use love_letter::LoveLetter;
use my_yrs_lib::BossOfYrs;

use super::align_two_block_lists::{align_blocks, Hunk};
use super::scratch_page::{
    open_a_scratch_copy_of_the_server_page, position_for_insert,
    read_blocks_with_ids,
};
use super::turn_one_change_into_a_love_letter::{
    delete_and_capture, edit_and_capture, insert_and_capture, text_edit_for,
};

pub fn create_love_letters_out_of_reconcile(
    server_page: Arc<BossOfYrs>,
    target_blocks: Vec<String>,
) -> Result<Vec<LoveLetter>, String> {
    let page_id = server_page.clone().page_id();
    let scratch = open_a_scratch_copy_of_the_server_page(server_page)?;

    let initial = read_blocks_with_ids(&scratch)?;
    let initial_texts: Vec<String> = initial.iter().map(|(_, t)| t.clone()).collect();
    let initial_ids: Vec<String> = initial.iter().map(|(id, _)| id.clone()).collect();

    let hunks = align_blocks(&initial_texts, &target_blocks);

    let mut letters = Vec::new();
    let mut last_placed_id: Option<String> = None;

    for hunk in hunks {
        match hunk {
            Hunk::Equal { a_start, len } => {
                for i in 0..len {
                    last_placed_id = Some(initial_ids[a_start + i].clone());
                }
            }
            Hunk::Replace {
                a_start,
                a_len,
                b_start,
                b_len,
            } => {
                let pairs = a_len.min(b_len);
                for j in 0..pairs {
                    let block_id = initial_ids[a_start + j].clone();
                    let old_text = initial_texts[a_start + j].clone();
                    let new_text = target_blocks[b_start + j].clone();
                    let edit = text_edit_for(&old_text, &new_text)?;
                    let letter = edit_and_capture(&scratch, &page_id, &block_id, edit)?;
                    letters.push(letter);
                    last_placed_id = Some(block_id);
                }
                for j in pairs..a_len {
                    let block_id = initial_ids[a_start + j].clone();
                    let letter = delete_and_capture(&scratch, &page_id, &block_id)?;
                    letters.push(letter);
                }
                for j in pairs..b_len {
                    let new_text = target_blocks[b_start + j].clone();
                    let position = position_for_insert(&scratch, last_placed_id.as_deref())?;
                    let (letter, new_id) =
                        insert_and_capture(&scratch, &page_id, new_text, position)?;
                    letters.push(letter);
                    last_placed_id = Some(new_id);
                }
            }
        }
    }

    let final_texts: Vec<String> = read_blocks_with_ids(&scratch)?
        .into_iter()
        .map(|(_, t)| t)
        .collect();
    if final_texts != target_blocks {
        return Err(format!(
            "self-check failed: got {:?} target {:?}",
            final_texts, target_blocks
        ));
    }

    Ok(letters)
}

//! Caller pov: turn one edit into one love letter, by applying the edit
//! to a scratch copy of the server's page and wrapping sketch + diff
//! snapshot into a letter.
//!
//! Depth 2: `edit_to_love_letter` matches on the edit and dispatches.
//! Depth 3: one `*_variant_to_love_letter` per sketch variant.
//! Depth 4: `update_doc_so_next_edits_are_aware` — the shared
//! bookmark → apply → snapshot pattern each variant uses.
//!
//! Insert is applied via `insert_new_block_with_preexisting_id`, so the
//! viewer's target block id becomes the doc id. No id bubbles back up,
//! and the letter stays `CreateNewBlockWithId`. New-block metadata is
//! currently passed empty — the plan treats metadata on creates as out
//! of scope.

use std::sync::Arc;

use love_letter::{build_a_love_letter, LoveLetter, LoveLetterSketch};
use my_yrs_lib::{
    yrs_wrapper::{create_bookmark, generate_diff_snapshot, PositionToInsert},
    BossOfYrs, EditTarget,
};

/// Depth 2. Take one edit, return its love letter.
pub(crate) fn edit_to_love_letter(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    edit: &LoveLetterSketch,
) -> Result<LoveLetter, String> {
    match edit {
        LoveLetterSketch::EditBlock {
            block_id,
            text_edit,
            edit_target,
            ..
        } => edit_variant_to_love_letter(
            scratch,
            page_id,
            block_id,
            text_edit,
            edit_target.clone(),
        ),
        LoveLetterSketch::RemoveBlock { block_id, .. } => {
            delete_variant_to_love_letter(scratch, page_id, block_id)
        }
        LoveLetterSketch::MoveBlock {
            block_id,
            new_frac_position,
            ..
        } => move_variant_to_love_letter(
            scratch,
            page_id,
            block_id,
            new_frac_position,
        ),
        LoveLetterSketch::CreateNewBlockWithId { .. }
        | LoveLetterSketch::CreateNewBlock { .. } => Err(
            "creates must go through turn_all_edits_into_love_letters pass 2, not edit_to_love_letter"
                .to_string(),
        ),
        LoveLetterSketch::DisablePage { .. }
        | LoveLetterSketch::CreateNewPage { .. } => Err(format!(
            "unexpected page-level sketch in per-page letter builder: {edit:?}"
        )),
    }
}

fn edit_variant_to_love_letter(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    block_id: &str,
    text_edit: &my_yrs_lib::TextEdit,
    edit_target: EditTarget,
) -> Result<LoveLetter, String> {
    let text_edit_for_apply = text_edit.clone();
    let block_for_apply = block_id.to_string();
    let edit_target_for_apply = edit_target.clone();
    let snapshot = update_doc_so_next_edits_are_aware(scratch, || {
        scratch
            .clone()
            .edit_text_block(block_for_apply, text_edit_for_apply, edit_target_for_apply)
            .map_err(|e| format!("edit_text_block: {e:?}"))
    })?;
    let sketch = LoveLetterSketch::EditBlock {
        text_edit: text_edit.clone(),
        edit_target,
        target_page_id: page_id.to_string(),
        block_id: block_id.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

fn delete_variant_to_love_letter(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    block_id: &str,
) -> Result<LoveLetter, String> {
    let block_for_apply = block_id.to_string();
    let snapshot = update_doc_so_next_edits_are_aware(scratch, || {
        scratch
            .clone()
            .delete_block(block_for_apply)
            .map_err(|e| format!("delete_block: {e:?}"))
    })?;
    let sketch = LoveLetterSketch::RemoveBlock {
        block_id: block_id.to_string(),
        target_page_id: page_id.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

fn move_variant_to_love_letter(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    block_id: &str,
    new_frac_position: &str,
) -> Result<LoveLetter, String> {
    let block_for_apply = block_id.to_string();
    let frac_for_apply = new_frac_position.to_string();
    let snapshot = update_doc_so_next_edits_are_aware(scratch, || {
        scratch
            .clone()
            .set_frac_position(block_for_apply, frac_for_apply)
            .map_err(|e| format!("set_frac_position: {e:?}"))
    })?;
    let sketch = LoveLetterSketch::MoveBlock {
        block_id: block_id.to_string(),
        target_page_id: page_id.to_string(),
        new_frac_position: new_frac_position.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

pub(crate) fn insert_variant_to_love_letter(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    target_id: &str,
    text: String,
    position: PositionToInsert,
) -> Result<LoveLetter, String> {
    let id_for_apply = target_id.to_string();
    let position_for_apply = position.clone();
    let snapshot = update_doc_so_next_edits_are_aware(scratch, || {
        scratch
            .clone()
            .insert_new_block_with_preexisting_id(
                id_for_apply,
                text,
                String::new(),
                position_for_apply,
            )
            .map_err(|e| format!("insert_new_block_with_preexisting_id: {e:?}"))
    })?;
    let sketch = LoveLetterSketch::CreateNewBlockWithId {
        block_id: target_id.to_string(),
        position_to_insert: position,
        target_page_id: page_id.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

/// Depth 4. Bookmark the scratch, run `apply`, diff, return the
/// snapshot. Used by all four variants above.
fn update_doc_so_next_edits_are_aware(
    scratch: &Arc<BossOfYrs>,
    apply: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<u8>, String> {
    let bookmark =
        create_bookmark(scratch.clone()).map_err(|e| format!("bookmark: {e:?}"))?;
    apply()?;
    generate_diff_snapshot(scratch.clone(), bookmark)
        .map_err(|e| format!("generate_diff_snapshot: {e:?}"))
}
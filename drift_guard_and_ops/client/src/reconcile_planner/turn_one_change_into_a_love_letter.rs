//! Caller pov: take a scratch doc + one change, get back the letter that
//! captures that change. Three shapes: edit a block's text, delete a
//! block, insert a new block. Each applies the change to the scratch
//! doc, snapshots the diff, wraps sketch + snapshot into a LoveLetter,
//! and returns it. Insert also returns the new block's id so the caller
//! knows where to place the next insert.

use std::sync::Arc;

use love_letter::{build_a_love_letter, LoveLetter, LoveLetterSketch};
use my_yrs_lib::{
    yrs_wrapper::{create_bookmark, generate_diff_snapshot, PositionToInsert},
    BossOfYrs, EditTarget, TextEdit,
};
use text_diff::diff_logic::{get_diff, DiffResult};

/// Convert the difference between two block texts into a `TextEdit` the
/// yrs crate can apply. Errors only if the two texts are identical — a
/// replace hunk shouldn't contain an unchanged block.
pub fn text_edit_for(old: &str, new: &str) -> Result<TextEdit, String> {
    match get_diff(old, new) {
        DiffResult::NoDiff => Err(format!(
            "no diff between {old:?} and {new:?} inside a replace hunk"
        )),
        DiffResult::Insert(text, position) => Ok(TextEdit::Insert { text, position }),
        DiffResult::Delete(text, position) => Ok(TextEdit::Delete { text, position }),
        DiffResult::Replace {
            old_text,
            new_text,
            position,
        } => Ok(TextEdit::Replace {
            old_text,
            new_text,
            position,
        }),
    }
}

pub fn edit_and_capture(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    block_id: &str,
    edit: TextEdit,
) -> Result<LoveLetter, String> {
    let bookmark = create_bookmark(scratch.clone()).map_err(|e| format!("bookmark: {e:?}"))?;
    scratch
        .clone()
        .edit_text_block(block_id.to_string(), edit.clone(), EditTarget::Text)
        .map_err(|e| format!("edit_text_block: {e:?}"))?;
    let snapshot = generate_diff_snapshot(scratch.clone(), bookmark)
        .map_err(|e| format!("generate_diff_snapshot: {e:?}"))?;
    let sketch = LoveLetterSketch::EditBlock {
        text_edit: edit,
        edit_target: EditTarget::Text,
        target_page_id: page_id.to_string(),
        block_id: block_id.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

pub fn delete_and_capture(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    block_id: &str,
) -> Result<LoveLetter, String> {
    let bookmark = create_bookmark(scratch.clone()).map_err(|e| format!("bookmark: {e:?}"))?;
    scratch
        .clone()
        .delete_block(block_id.to_string())
        .map_err(|e| format!("delete_block: {e:?}"))?;
    let snapshot = generate_diff_snapshot(scratch.clone(), bookmark)
        .map_err(|e| format!("generate_diff_snapshot: {e:?}"))?;
    let sketch = LoveLetterSketch::RemoveBlock {
        block_id: block_id.to_string(),
        target_page_id: page_id.to_string(),
    };
    Ok(build_a_love_letter(sketch, snapshot))
}

pub fn insert_and_capture(
    scratch: &Arc<BossOfYrs>,
    page_id: &str,
    content: String,
    position: PositionToInsert,
) -> Result<(LoveLetter, String), String> {
    let bookmark = create_bookmark(scratch.clone()).map_err(|e| format!("bookmark: {e:?}"))?;
    let new_id = scratch
        .clone()
        .insert_new_block(content, String::new(), position.clone())
        .map_err(|e| format!("insert_new_block: {e:?}"))?;
    let snapshot = generate_diff_snapshot(scratch.clone(), bookmark)
        .map_err(|e| format!("generate_diff_snapshot: {e:?}"))?;
    let sketch = LoveLetterSketch::CreateNewBlock {
        position_to_insert: position,
        target_page_id: page_id.to_string(),
    };
    Ok((build_a_love_letter(sketch, snapshot), new_id))
}

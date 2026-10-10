use error_stuff::cqrs_err::CqrsErr;
use love_letter::{LoveLetterSketch, sketch_to_bytes};
use my_yrs_lib::{
    BossOfYrs, EditTarget, TextEdit, YrsError,
    yrs_error::ErrorInfo,
    yrs_wrapper::{self},
};
use std::sync::Arc;
use text_diff::diff_logic::DiffResult;

use crate::{
    diff_result_to_text_edit_conversion::diff_result_to_text_edit,
    shared::{PagesUpdateCtx, UncommitedDiffsInsertCtx},
};

pub struct EverythingForEditBlock {
    pub pages_update: PagesUpdateCtx,
    pub every_block_in_existence_update: EveryBlockInExistenceUpdateCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub struct EveryBlockInExistenceUpdateCtx {
    pub block_id: String,
    pub new_content: String,
}

pub fn edit_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    diff: DiffResult,
    session_id: String,
) -> Result<Option<EverythingForEditBlock>, CqrsErr> {
    let text_edit = match diff_result_to_text_edit(diff.clone()) {
        Some(t) => t,
        None => return Ok(None),
    };

    let bookmark_before_edit = yrs_wrapper::create_bookmark(yrs.clone())?;

    yrs.clone()
        .edit_text_block(block_id.clone(), text_edit.clone(), EditTarget::Text)?;

    let snapshot_of_edit = yrs_wrapper::generate_diff_snapshot(yrs.clone(), bookmark_before_edit)?;

    let new_text = yrs.clone().read_block(block_id.clone())?.ok_or_else(|| {
        CqrsErr::YrsErrorContainer(YrsError::GenericError {
            info: ErrorInfo {
                error_msg: format!("no block with id: {block_id}"),
                file: file!().to_string(),
                method: "edit_block".to_string(),
            },
        })
    })?;

    let yrs_blob = yrs.clone().snapshot()?;

    let page_id = yrs.clone().page_id();
    let love_letter_sketch =
        make_love_letter_sketch_for_editing_block(text_edit, page_id.clone(), block_id.clone())?;

    Ok(Some(EverythingForEditBlock {
        pages_update: PagesUpdateCtx {
            page_id: page_id.clone(),
            yrs_blob,
        },
        every_block_in_existence_update: EveryBlockInExistenceUpdateCtx {
            block_id,
            new_content: new_text,
        },
        uncommitted_diffs: UncommitedDiffsInsertCtx {
            snapshot_of_edit,
            love_letter_sketch,
            session_id,
            target_id: page_id,
        },
    }))
}

fn make_love_letter_sketch_for_editing_block(
    text_edit: TextEdit,
    target_page_id: String,
    block_id: String,
) -> Result<Vec<u8>, CqrsErr> {
    let sketch = LoveLetterSketch::EditBlock {
        text_edit,
        edit_target: EditTarget::Text,
        target_page_id,
        block_id,
    };

    let bytes = sketch_to_bytes(&sketch)?;
    Ok(bytes)
}

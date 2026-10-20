use error_stuff::cqrs_err::CqrsErr;
use love_letter::{LoveLetterSketch, sketch_to_bytes};
use my_yrs_lib::{
    BossOfYrs, YrsError,
    yrs_error::ErrorInfo,
    yrs_wrapper::{self, PositionToInsert},
};
use std::sync::Arc;

use crate::shared::{ClientLoveLetterInsertCtx, PagesUpdateCtx};

#[derive(uniffi::Record)]
pub struct EverythingForAddBlock {
    pub pages_update: PagesUpdateCtx,
    pub every_block_in_existence_insert: EveryBlockInExistenceInsertCtx,
    pub client_love_letter: ClientLoveLetterInsertCtx,
}

#[derive(uniffi::Record)]
pub struct EveryBlockInExistenceInsertCtx {
    pub my_id_as_given_by_yrs: String,
    pub content: String,
    pub id_of_page_i_belong_to: String,
    /// The block's fractional index — a hex string. Known before the
    /// row is inserted because yrs computes it when the block is made.
    pub frac_position: String,
    pub is_part_of_main_menu_page: bool,
}

fn insert_into_yrs(
    yrs: Arc<BossOfYrs>,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    insert_with_existing_block_id: Option<String>,
) -> Result<(String, String), CqrsErr> {
    let block_id = match insert_with_existing_block_id {
        Some(existing_id) => {
            yrs.clone().insert_new_block_with_preexisting_id(
                existing_id.clone(),
                block_content,
                block_meta_data,
                position,
            )?;
            existing_id
        }
        None => yrs
            .clone()
            .insert_new_block(block_content, block_meta_data, position)?,
    };
    let frac = yrs.clone().get_frac_position_of_block(block_id.clone())?;
    Ok((block_id, frac))
}

pub fn add_block(
    yrs: Arc<BossOfYrs>,
    block_content: String,
    block_meta_data: String,
    position: PositionToInsert,
    is_part_of_main_menu_page: bool,
    session_id: String,
    insert_with_existing_block_id: Option<String>,
) -> Result<EverythingForAddBlock, CqrsErr> {
    let bookmark_before_edit = yrs_wrapper::create_bookmark(yrs.clone())?;
    let position_for_love_letter = position.clone();
    let is_create_with_id = insert_with_existing_block_id.is_some();

    let (block_id, new_frac_position) = insert_into_yrs(
        yrs.clone(),
        block_content,
        block_meta_data,
        position,
        insert_with_existing_block_id,
    )?;

    let snapshot_of_edit = yrs_wrapper::generate_diff_snapshot(yrs.clone(), bookmark_before_edit)?;

    let new_content = yrs.clone().read_block(block_id.clone())?.ok_or_else(|| {
        CqrsErr::YrsErrorContainer(YrsError::GenericError {
            info: ErrorInfo {
                error_msg: format!("no block with id: {block_id}"),
                file: file!().to_string(),
                method: "add_block".to_string(),
            },
        })
    })?;

    let yrs_blob = yrs.clone().snapshot()?;
    let page_id = yrs.clone().page_id();

    let love_letter_sketch = if is_create_with_id {
        LoveLetterSketch::CreateNewBlockWithId {
            block_id: block_id.clone(),
            position_to_insert: position_for_love_letter,
            target_page_id: page_id.clone(),
        }
    } else {
        LoveLetterSketch::CreateNewBlock {
            position_to_insert: position_for_love_letter,
            target_page_id: page_id.clone(),
        }
    };
    let love_letter_sketch_bytes = sketch_to_bytes(&love_letter_sketch)?;

    Ok(EverythingForAddBlock {
        pages_update: PagesUpdateCtx {
            page_id: page_id.clone(),
            yrs_blob,
        },
        every_block_in_existence_insert: EveryBlockInExistenceInsertCtx {
            my_id_as_given_by_yrs: block_id,
            content: new_content,
            id_of_page_i_belong_to: page_id.clone(),
            frac_position: new_frac_position,
            is_part_of_main_menu_page,
        },
        client_love_letter: ClientLoveLetterInsertCtx {
            snapshot_of_edit,
            love_letter_sketch: love_letter_sketch_bytes,
            session_id,
            target_id: page_id,
        },
    })
}

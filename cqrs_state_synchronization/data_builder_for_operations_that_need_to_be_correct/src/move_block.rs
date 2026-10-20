use error_stuff::cqrs_err::CqrsErr;
use love_letter::{LoveLetterSketch, sketch_to_bytes};
use my_yrs_lib::{BossOfYrs, yrs_wrapper};
use std::sync::Arc;

use crate::shared::{ClientLoveLetterInsertCtx, PagesUpdateCtx};

#[derive(uniffi::Record)]
pub struct EverythingForMoveBlock {
    pub pages_update: PagesUpdateCtx,
    pub block_id: String,
    pub new_frac_position: String,
    pub client_love_letter: ClientLoveLetterInsertCtx,
}

pub fn move_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    block_id_of_above: Option<String>,
    block_id_of_below: Option<String>,
    session_id: String,
) -> Result<EverythingForMoveBlock, CqrsErr> {
    let bookmark_before_edit = yrs_wrapper::create_bookmark(yrs.clone())?;
    yrs.clone()
        .move_block(block_id.clone(), block_id_of_above, block_id_of_below)?;
    let snapshot_of_edit = yrs_wrapper::generate_diff_snapshot(yrs.clone(), bookmark_before_edit)?;

    let new_frac_position = yrs.clone().get_frac_position_of_block(block_id.clone())?;
    let yrs_blob = yrs.clone().snapshot()?;
    let page_id = yrs.clone().page_id();

    let love_letter_sketch = LoveLetterSketch::MoveBlock {
        block_id: block_id.clone(),
        target_page_id: page_id.clone(),
        new_frac_position: new_frac_position.clone(),
    };
    let love_letter_sketch_bytes = sketch_to_bytes(&love_letter_sketch)?;

    Ok(EverythingForMoveBlock {
        pages_update: PagesUpdateCtx {
            page_id: page_id.clone(),
            yrs_blob,
        },
        block_id,
        new_frac_position,
        client_love_letter: ClientLoveLetterInsertCtx {
            snapshot_of_edit,
            love_letter_sketch: love_letter_sketch_bytes,
            session_id,
            target_id: page_id,
        },
    })
}

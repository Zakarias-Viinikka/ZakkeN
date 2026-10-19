use error_stuff::cqrs_err::CqrsErr;
use love_letter::{LoveLetterSketch, sketch_to_bytes};
use my_yrs_lib::{BossOfYrs, yrs_wrapper};
use std::sync::Arc;

use crate::shared::{ClientLoveLetterInsertCtx, PagesUpdateCtx};

#[derive(uniffi::Record)]
pub struct EverythingForDeleteBlock {
    pub pages_update: PagesUpdateCtx,
    pub block_id: String,
    pub client_love_letter: ClientLoveLetterInsertCtx,
}

pub fn delete_block(
    yrs: Arc<BossOfYrs>,
    block_id: String,
    session_id: String,
) -> Result<EverythingForDeleteBlock, CqrsErr> {
    let bookmark_before_edit = yrs_wrapper::create_bookmark(yrs.clone())?;
    yrs.clone().delete_block(block_id.clone())?;
    let snapshot_of_edit = yrs_wrapper::generate_diff_snapshot(yrs.clone(), bookmark_before_edit)?;

    let yrs_blob = yrs.clone().snapshot()?;
    let page_id = yrs.clone().page_id();

    let love_letter_sketch = LoveLetterSketch::RemoveBlock {
        block_id: block_id.clone(),
        target_page_id: page_id.clone(),
    };
    let love_letter_sketch_bytes = sketch_to_bytes(&love_letter_sketch)?;

    Ok(EverythingForDeleteBlock {
        pages_update: PagesUpdateCtx {
            page_id: page_id.clone(),
            yrs_blob,
        },
        block_id,
        client_love_letter: ClientLoveLetterInsertCtx {
            snapshot_of_edit,
            love_letter_sketch: love_letter_sketch_bytes,
            session_id,
            target_id: page_id,
        },
    })
}

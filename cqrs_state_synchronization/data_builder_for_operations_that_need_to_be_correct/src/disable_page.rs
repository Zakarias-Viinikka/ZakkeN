use error_stuff::cqrs_err::CqrsErr;
use love_letter::{LoveLetterSketch, sketch_to_bytes};
use my_yrs_lib::yrs_active_pages;
use std::sync::Arc;

use crate::shared::{PagesUpdateCtx, UncommitedDiffsInsertCtx};

pub struct EverythingForDisablePage {
    pub pages_update: PagesUpdateCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub fn disable_page(
    page_id: String,
    session_id: String,
    active_pages_serialized_form: Vec<u8>,
) -> Result<EverythingForDisablePage, CqrsErr> {
    let boss_of_active_pages = Arc::new(yrs_active_pages::YrsActivePages::new(
        active_pages_serialized_form,
    )?);
    let bookmark = boss_of_active_pages.clone().create_bookmark()?;
    boss_of_active_pages
        .clone()
        .mark_page_deleted(page_id.clone())?;
    let snapshot_of_edit = boss_of_active_pages
        .clone()
        .generate_diff_snapshot(bookmark)?;
    let target_id = page_id.clone();
    let love_letter_sketch = LoveLetterSketch::DisablePage {
        page_id: page_id.clone(),
    };

    let love_letter_sketch_bytes = sketch_to_bytes(&love_letter_sketch)?;

    let new_active_pages_blob = boss_of_active_pages.clone().snapshot()?;

    let uncommitted_diffs_ctx = UncommitedDiffsInsertCtx {
        snapshot_of_edit,
        love_letter_sketch: love_letter_sketch_bytes,
        session_id,
        target_id,
    };

    let pages_update = PagesUpdateCtx {
        page_id,
        yrs_blob: new_active_pages_blob,
    };

    Ok(EverythingForDisablePage {
        pages_update,
        uncommitted_diffs: uncommitted_diffs_ctx,
    })
}

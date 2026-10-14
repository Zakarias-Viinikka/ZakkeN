use crate::db::operations_pages;
use db_wrapper::android_mascot::LiveForever;
use love_letter::LoveLetter;
use my_yrs_lib::yrs_wrapper::create_bookmark;
use my_yrs_lib::{BossOfYrs, doc_from_snapshot, yrs_active_pages::YrsActivePages};
use protocol::error::DbError;
use std::sync::Arc;

// Mirrors cqrs's FAKE_TIME. The server's docs are only ever opened to apply a
// letter, never to generate new IDs, so the time string is unused.
const FAKE_TIME: &str = "";

pub fn apply_letter(liver: &LiveForever, letter: LoveLetter) -> Result<(), DbError> {
    match letter {
        LoveLetter::CreateNewPage {
            page_id,
            is_main_menu_page,
            snapshot_of_edit,
        } => create_page(liver, page_id, is_main_menu_page, snapshot_of_edit),

        LoveLetter::EditBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        } => merge_into_page_blob(liver, target_page_id, snapshot_of_edit),

        LoveLetter::CreateNewBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        } => merge_into_page_blob(liver, target_page_id, snapshot_of_edit),

        LoveLetter::RemoveBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        } => merge_into_page_blob(liver, target_page_id, snapshot_of_edit),

        LoveLetter::DisablePage {
            page_id,
            snapshot_of_edit,
        } => merge_into_page_status(liver, page_id, snapshot_of_edit),
    }
}

fn create_page(
    liver: &LiveForever,
    page_id: String,
    is_main_menu_page: bool,
    snapshot_of_edit: Vec<u8>,
) -> Result<(), DbError> {
    if operations_pages::page_exists(liver, &page_id)? {
        // CreateNewPage for a page that already exists — merge it like an edit
        // rather than insert a duplicate.
        return merge_into_page_blob(liver, page_id, snapshot_of_edit);
    }

    let boss = Arc::new(
        doc_from_snapshot(
            snapshot_of_edit.clone(),
            "server".to_string(),
            page_id.clone(),
            FAKE_TIME.to_string(),
        )
        .map_err(|e| DbError::BadCode(format!("create_page: decode snapshot: {e}")))?,
    );

    let version = create_bookmark(Arc::clone(&boss))
        .map_err(|e| DbError::BadCode(format!("create_page: bookmark: {e}")))?;

    let active_pages = Arc::new(YrsActivePages::new_empty());
    let page_status = active_pages
        .snapshot()
        .map_err(|e| DbError::BadCode(format!("create_page: page_status: {e}")))?;

    operations_pages::insert_page(
        liver,
        &page_id,
        snapshot_of_edit,
        page_status,
        version,
        if is_main_menu_page { "true" } else { "false" },
    )
}

fn merge_into_page_blob(
    liver: &LiveForever,
    page_id: String,
    snapshot_of_edit: Vec<u8>,
) -> Result<(), DbError> {
    let stored = operations_pages::get_page(liver, &page_id)?
        .ok_or_else(|| DbError::IllegalInput(format!("no such page: '{page_id}'")))?;

    let boss = Arc::new(
        doc_from_snapshot(
            stored.blobbed_page,
            "server".to_string(),
            page_id.clone(),
            FAKE_TIME.to_string(),
        )
        .map_err(|e| DbError::BadCode(format!("merge_into_page_blob: decode: {e}")))?,
    );

    BossOfYrs::merge_with_snapshot(Arc::clone(&boss), snapshot_of_edit)
        .map_err(|e| DbError::BadCode(format!("merge_into_page_blob: merge: {e}")))?;

    let new_blob = BossOfYrs::snapshot(Arc::clone(&boss))
        .map_err(|e| DbError::BadCode(format!("merge_into_page_blob: snapshot: {e}")))?;

    operations_pages::update_page_blob(liver, &page_id, new_blob)
}

fn merge_into_page_status(
    liver: &LiveForever,
    page_id: String,
    snapshot_of_edit: Vec<u8>,
) -> Result<(), DbError> {
    let stored = operations_pages::get_page(liver, &page_id)?
        .ok_or_else(|| DbError::IllegalInput(format!("no such page: '{page_id}'")))?;

    let active = Arc::new(
        YrsActivePages::new(stored.page_status)
            .map_err(|e| DbError::BadCode(format!("merge_into_page_status: decode: {e}")))?,
    );

    YrsActivePages::merge_with_snapshot(Arc::clone(&active), snapshot_of_edit)
        .map_err(|e| DbError::BadCode(format!("merge_into_page_status: merge: {e}")))?;

    let new_status = YrsActivePages::snapshot(Arc::clone(&active))
        .map_err(|e| DbError::BadCode(format!("merge_into_page_status: snapshot: {e}")))?;

    operations_pages::update_page_status(liver, &page_id, new_status)
}
//! Caller pov: replace everything in the local db with the pages from an
//! export zip on the server's disk, then go lonewolf.
//!
//! Local only. No love letters are written, because the imported pages
//! are new docs that share no history with the server's copies. The
//! wipe and every page write happen in one transaction, so a failed
//! request midway leaves the old data untouched.

use data_builder_for_operations_that_need_to_be_correct::build_import_page;
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::{
    apply_merge_plan, run_in_transaction,
};
use client_table_blueprints::{
    tbl_client_love_letter, tbl_every_block_in_existence, tbl_pages, tbl_seen_server_letters,
};
use leptos::prelude::Update;
use server_protocol::{ImportedPage, ServErr};

use super::import_requests::fetch_import_pages;
use crate::db::cached_ids::current_user_id;
use crate::db::dev_actions::{enter_lone_wolf, wipe_tables};
use crate::db::write_lock::WRITE_LOCK;
use crate::db::yrs_errors::yrs_error_at;
use crate::get_time;

const PAGES_PER_REQUEST: usize = 50;

pub async fn import_pages_from_export_zip(zip_path: String) -> Result<(), CqrsErr> {
    // write_lock.rs: flush before taking the lock, never while holding it.
    crate::complex_state::batcher_handle::flush_now().await?;

    let guard = WRITE_LOCK.acquire().await;
    run_in_transaction(|| async move {
        wipe_tables(&[
            tbl_pages::TABLE_NAME,
            tbl_every_block_in_existence::TABLE_NAME,
            tbl_client_love_letter::TABLE_NAME,
            tbl_seen_server_letters::TABLE_NAME,
        ])
        .await?;
        apply_every_page_of_the_zip(zip_path).await
    })
    .await?;
    drop(guard);

    crate::owner_of_interacting_with_server::MENU_TICK.update(|n| *n = n.wrapping_add(1));
    enter_lone_wolf().await
}

async fn apply_every_page_of_the_zip(zip_path: String) -> Result<(), CqrsErr> {
    let user_id = current_user_id();
    let mut after_page_id: Option<String> = None;
    loop {
        let reply = fetch_import_pages(zip_path.clone(), after_page_id.clone(), PAGES_PER_REQUEST)
            .await
            .map_err(cqrs_err_from_server_err)?;
        let has_more = reply.has_more;
        let Some(last_page_id) = reply.pages.last().map(|p| p.page_id.clone()) else {
            break;
        };
        for page in reply.pages {
            apply_one_imported_page(page, user_id.clone()).await?;
        }
        if !has_more {
            break;
        }
        after_page_id = Some(last_page_id);
    }
    Ok(())
}

async fn apply_one_imported_page(page: ImportedPage, user_id: String) -> Result<(), CqrsErr> {
    let mut blocks = page.blocks;
    blocks.sort_by_key(|b| b.frac_index);
    let blocks_in_order = blocks
        .into_iter()
        .map(|b| my_yrs_lib::Block {
            text: b.content,
            metadata: b.metadata,
            id_in_yrs: b.block_id,
        })
        .collect();
    // The export doesn't carry is_main_menu_page, so every imported page
    // is treated as one.
    let plan = build_import_page(page.page_id, blocks_in_order, true, user_id, get_time())?;
    apply_merge_plan(plan).await
}

/// Stopgap: dressed as a yrs error because I haven't seen whether
/// CqrsErr has a server variant. If it does, swap it in here.
fn cqrs_err_from_server_err(e: ServErr) -> CqrsErr {
    yrs_error_at(
        format!("import: server refused a page request: {e:?}"),
        "apply_every_page_of_the_zip",
        file!(),
    )
    .into()
}

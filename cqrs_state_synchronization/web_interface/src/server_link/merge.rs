//! Apply a love letter received from the server.
//!
//! Reads what the builder needs from the db (or reuses the open page's
//! boss), runs the pure builder to get a plan, then runs the executor
//! under `run_exclusive` so no local flush or entry-point write can
//! interleave with the merge.
//!
//! Also updates `open.page1.blocks` when the merge touched the open
//! page, so the UI reflects the change without a reload.

use std::sync::{Arc, LazyLock};

use data_builder_for_operations_that_need_to_be_correct::{
    build_merge_blob, build_merge_disable, build_merge_new_page, FinalBlock,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::apply_merge_plan;
use leptos::prelude::{ArcRwSignal, GetUntracked, Set};
use love_letter::LoveLetter;
use my_yrs_lib::BossOfYrs;
use protocol::error::DbError;

use crate::complicated_signal_flow::{run_exclusive, BatcherHandle, BlockRow, OpenPages};
use crate::db::{cached_ids, db_helpers_for_web_client, ui_actions_helper};

/// Set when a DisablePage letter arrives for the page currently open in
/// the viewer. A route-level effect watches this and navigates to the
/// menu with a popup.
pub static REMOTE_DISABLED_OPEN_PAGE: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

/// Why an apply failed, so the worker can decide whether to halt or
/// just leave the letter unacked.
pub enum ApplyError {
    /// The letter targets a page that doesn't exist locally and isn't
    /// a CreateNewPage. The worker leaves it unacked; a following
    /// CreateNewPage for that page will make a re-pull succeed.
    UnknownPage(String),
    /// Anything else. The worker shows the "copy your changes, reload"
    /// popup and stops, because state is now suspect.
    Fatal(CqrsErr),
}

pub struct ApplyOutcome {
    pub page_id: String,
    pub changed: bool,
}

pub async fn apply_letter(
    batcher: &BatcherHandle,
    open: OpenPages,
    letter: LoveLetter,
) -> Result<ApplyOutcome, ApplyError> {
    match letter {
        LoveLetter::CreateNewPage {
            page_id,
            is_main_menu_page,
            snapshot_of_edit,
        } => {
            let exists = db_helpers_for_web_client::page_exists(page_id.clone())
                .await
                .map_err(|e| ApplyError::Fatal(CqrsErr::DbErrorContainer(e)))?;
            if exists {
                apply_blob(batcher, open, page_id, snapshot_of_edit).await
            } else {
                apply_new_page(batcher, open, page_id, is_main_menu_page, snapshot_of_edit).await
            }
        }
        LoveLetter::EditBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        }
        | LoveLetter::CreateNewBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        }
        | LoveLetter::CreateNewBlockWithId {
            target_page_id,
            snapshot_of_edit,
            ..
        }
        | LoveLetter::MoveBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        }
        | LoveLetter::RemoveBlock {
            target_page_id,
            snapshot_of_edit,
            ..
        } => apply_blob(batcher, open, target_page_id, snapshot_of_edit).await,
        LoveLetter::DisablePage {
            page_id,
            snapshot_of_edit,
        } => apply_disable(batcher, open, page_id, snapshot_of_edit).await,
    }
}

async fn apply_blob(
    batcher: &BatcherHandle,
    open: OpenPages,
    page_id: String,
    snapshot_of_edit: Vec<u8>,
) -> Result<ApplyOutcome, ApplyError> {
    let Some(boss) = pick_boss_for_page(open, &page_id).await.map_err(ApplyError::Fatal)? else {
        return Err(ApplyError::UnknownPage(page_id));
    };

    let existing_blocks =
        db_helpers_for_web_client::get_existing_blocks_for_page(page_id.clone())
            .await
            .map_err(|e| ApplyError::Fatal(CqrsErr::DbErrorContainer(e)))?;

    let is_main_menu_page = ui_actions_helper::is_main_menu_page(page_id.clone())
        .await
        .map_err(|e| ApplyError::Fatal(e))?;

    let pid_for_plan = page_id.clone();
    let pid_for_refresh = page_id.clone();
    let changed = run_exclusive(batcher, move || async move {
        let plan = build_merge_blob(
            pid_for_plan,
            snapshot_of_edit,
            existing_blocks,
            is_main_menu_page,
            boss,
        )?;
        let final_out = plan.final_blocks.clone();
        apply_merge_plan(plan).await?;
        // Refresh inside the lock: a keystroke landing between the
        // lock release and the UI refresh would push a diff against
        // the pre-merge content to the batcher, and that diff would
        // apply to the wrong offsets on the next flush.
        Ok(refresh_open_page_if_matches(open, &pid_for_refresh, final_out))
    })
    .await
    .map_err(ApplyError::Fatal)?;

    Ok(ApplyOutcome { page_id, changed })
}

async fn apply_new_page(
    batcher: &BatcherHandle,
    open: OpenPages,
    page_id: String,
    is_main_menu_page: bool,
    snapshot_of_edit: Vec<u8>,
) -> Result<ApplyOutcome, ApplyError> {
    let user_id = cached_ids::USER_ID.get_untracked().ok_or_else(|| {
        ApplyError::Fatal(CqrsErr::DbErrorContainer(DbError::IllegalInput(
            "USER_ID not set".to_string(),
        )))
    })?;
    let time = crate::get_time();

    let pid_for_plan = page_id.clone();
    let final_blocks = run_exclusive(batcher, move || async move {
        let plan = build_merge_new_page(
            pid_for_plan,
            is_main_menu_page,
            snapshot_of_edit,
            user_id,
            time,
        )?;
        let final_out = plan.final_blocks.clone();
        apply_merge_plan(plan).await?;
        Ok(final_out)
    })
    .await
    .map_err(ApplyError::Fatal)?;

    // A fresh page always changes the menu, whether or not it's the one
    // currently open. The `refresh_open_page_if_matches` return is
    // discarded on purpose: it's only true if the page was open AND
    // the blocks differed, which won't be true for a new page that
    // isn't the one being viewed.
    let _ = refresh_open_page_if_matches(open, &page_id, final_blocks);
    Ok(ApplyOutcome {
        page_id,
        changed: true,
    })
}

async fn apply_disable(
    batcher: &BatcherHandle,
    open: OpenPages,
    page_id: String,
    snapshot_of_edit: Vec<u8>,
) -> Result<ApplyOutcome, ApplyError> {
    let exists = db_helpers_for_web_client::page_exists(page_id.clone())
        .await
        .map_err(|e| ApplyError::Fatal(CqrsErr::DbErrorContainer(e)))?;
    if !exists {
        return Err(ApplyError::UnknownPage(page_id));
    }

    let status = ui_actions_helper::get_active_pages_blob(page_id.clone())
        .await
        .map_err(|e| ApplyError::Fatal(e))?;

    let pid_for_plan = page_id.clone();
    run_exclusive(batcher, move || async move {
        let plan = build_merge_disable(pid_for_plan, snapshot_of_edit, status)?;
        apply_merge_plan(plan).await
    })
    .await
    .map_err(ApplyError::Fatal)?;

    if open.page1.page_id.get_untracked() == page_id {
        REMOTE_DISABLED_OPEN_PAGE.set(Some(page_id.clone()));
    }

    Ok(ApplyOutcome {
        page_id,
        changed: true,
    })
}

/// If the merged page is the one currently open, update its blocks so
/// the UI shows the change. Returns whether anything actually changed.
fn refresh_open_page_if_matches(
    open: OpenPages,
    page_id: &str,
    final_blocks: Vec<FinalBlock>,
) -> bool {
    if open.page1.page_id.get_untracked() != page_id {
        return false;
    }
    let new_rows: Vec<BlockRow> = final_blocks
        .into_iter()
        .map(|b| BlockRow {
            block_id: b.block_id,
            page_id: page_id.to_string(),
            content: b.content,
            is_title: b.is_title,
        })
        .collect();
    let unchanged = {
        let current = open.page1.blocks.get_untracked();
        if current.len() != new_rows.len() {
            false
        } else {
            current
                .iter()
                .zip(new_rows.iter())
                .all(|(a, b)| a.block_id == b.block_id && a.content == b.content)
        }
    };
    if !unchanged {
        open.page1.blocks.set(new_rows);
        true
    } else {
        false
    }
}

pub(crate) async fn pick_boss_for_page(
    open: OpenPages,
    page_id: &str,
) -> Result<Option<Arc<BossOfYrs>>, CqrsErr> {
    if open.page1.page_id.get_untracked() == page_id {
        if let Some(boss) = open.page1.boss_of_yrs.get_untracked() {
            return Ok(Some(boss));
        }
    }

    if !db_helpers_for_web_client::page_exists(page_id.to_string())
        .await
        .map_err(CqrsErr::DbErrorContainer)?
    {
        return Ok(None);
    }

    let user_id = cached_ids::USER_ID.get_untracked().ok_or_else(|| {
        CqrsErr::DbErrorContainer(DbError::IllegalInput("USER_ID not set".to_string()))
    })?;
    let boss = db_helpers_for_web_client::get_yrs_unblobbed(
        page_id.to_string(),
        user_id,
        crate::get_time(),
    )
    .await
    .map_err(CqrsErr::DbErrorContainer)?;
    Ok(Some(boss))
}
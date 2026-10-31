use std::sync::Arc;

use data_builder_for_operations_that_need_to_be_correct::{
    build_merge_blob, build_merge_disable, build_merge_new_page, FinalBlock,
};
use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::apply_merge_plan;
use leptos::prelude::{GetUntracked, Set};
use love_letter::LoveLetter;
use my_yrs_lib::BossOfYrs;
use protocol::error::DbError;

use crate::shared_owners::BlockRow;
use crate::db::batcher::{run_exclusive, BatcherHandle};
use crate::db::cached_ids::{self, try_current_user_id};
use crate::db::reads::{blocks as reads_blocks, pages as reads_pages};
use crate::page_state::OpenPages;

pub enum ApplyError {
    UnknownPage(String),
    Fatal(CqrsErr),
}

impl From<DbError> for ApplyError {
    fn from(e: DbError) -> Self {
        ApplyError::Fatal(CqrsErr::DbErrorContainer(e))
    }
}

impl From<CqrsErr> for ApplyError {
    fn from(e: CqrsErr) -> Self {
        ApplyError::Fatal(e)
    }
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
            let exists = reads_pages::page_exists(page_id.clone()).await?;
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
    let Some(boss) = pick_boss_for_page(open, &page_id).await? else {
        return Err(ApplyError::UnknownPage(page_id));
    };

    let existing_blocks = reads_blocks::get_existing_blocks_for_page(page_id.clone()).await?;

    let is_main_menu_page = reads_pages::is_main_menu_page(page_id.clone()).await?;

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
        Ok(refresh_open_page_if_matches(open, &pid_for_refresh, final_out))
    })
    .await?;

    Ok(ApplyOutcome { page_id, changed })
}

async fn apply_new_page(
    batcher: &BatcherHandle,
    open: OpenPages,
    page_id: String,
    is_main_menu_page: bool,
    snapshot_of_edit: Vec<u8>,
) -> Result<ApplyOutcome, ApplyError> {
    let user_id = try_current_user_id()?;
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
    .await?;

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
    let exists = reads_pages::page_exists(page_id.clone()).await?;
    if !exists {
        return Err(ApplyError::UnknownPage(page_id));
    }

    let status = reads_pages::get_active_pages_blob(page_id.clone()).await?;

    let pid_for_plan = page_id.clone();
    run_exclusive(batcher, move || async move {
        let plan = build_merge_disable(pid_for_plan, snapshot_of_edit, status)?;
        apply_merge_plan(plan).await
    })
    .await?;

    if open.page1.page_id.get_untracked() == page_id {
        crate::leptos_components::small_components::popup::create_popup(
            "This page was disabled from another client. You can keep working on it, but it won't appear in the menu anymore."
                .to_string(),
        );
    }

    Ok(ApplyOutcome {
        page_id,
        changed: true,
    })
}

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
    if let Some(boss) = open.boss_for_page(page_id) {
        return Ok(Some(boss));
    }

    if !reads_pages::page_exists(page_id.to_string())
        .await
        .map_err(CqrsErr::DbErrorContainer)?
    {
        return Ok(None);
    }

    let user_id = cached_ids::try_current_user_id()?;
    let boss = reads_pages::get_yrs_unblobbed(
        page_id.to_string(),
        user_id,
        crate::get_time(),
    )
    .await
    .map_err(CqrsErr::DbErrorContainer)?;
    Ok(Some(boss))
}
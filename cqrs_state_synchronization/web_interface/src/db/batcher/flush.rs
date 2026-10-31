use std::sync::Arc;

use error_stuff::CqrsErr;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::run_in_transaction;
use leptos::prelude::*;
use my_yrs_lib::BossOfYrs;
use text_diff::diff_batching::combine_list_of_getdiff_results::combine_getdiff_results;
use text_diff::diff_logic::DiffResult;

use crate::shared_owners::APP_LOCKED;
use crate::db::cached_ids::current_session_id;
use crate::db::main_send_unsent_love_letters::main_send_unsent_love_letters;
use crate::db::writes;

use super::handle::PendingWrite;

pub(crate) async fn flush_pending_writes(
    pending: &mut Vec<PendingWrite>,
) -> Result<(), CqrsErr> {
    if pending.is_empty() {
        return Ok(());
    }

    let batch = std::mem::take(pending);

    let _write_guard = crate::db::write_lock::WRITE_LOCK.acquire().await;

    let session_id = current_session_id();

    let result = run_in_transaction(move || async move {
        write_pending_in_order(batch, &session_id).await
    })
    .await;

    match result {
        Ok(()) => {
            main_send_unsent_love_letters();
            Ok(())
        }
        Err(e) => {
            lock_app_after_failed_flush();
            Err(e)
        }
    }
}

async fn write_pending_in_order(
    batch: Vec<PendingWrite>,
    session_id: &str,
) -> Result<(), CqrsErr> {
    let mut run: Vec<(String, Arc<BossOfYrs>, Vec<DiffResult>)> = Vec::new();
    for op in batch {
        match op {
            PendingWrite::Edit {
                boss,
                block_id,
                diff,
            } => {
                if let Some(existing) = run.iter_mut().find(|(bid, _, _)| bid == &block_id) {
                    existing.2.push(diff);
                } else {
                    run.push((block_id, boss, vec![diff]));
                }
            }
            PendingWrite::AddBlock {
                boss,
                block_id,
                content,
                position,
                is_part_of_main_menu_page,
                session_id: add_session_id,
            } => {
                write_edit_run(std::mem::take(&mut run), session_id).await?;
                writes::add_block_unflushed(
                    boss,
                    block_id,
                    content,
                    String::new(),
                    position,
                    is_part_of_main_menu_page,
                    add_session_id,
                )
                .await?;
            }
        }
    }
    write_edit_run(run, session_id).await
}

async fn write_edit_run(
    run: Vec<(String, Arc<BossOfYrs>, Vec<DiffResult>)>,
    session_id: &str,
) -> Result<(), CqrsErr> {
    for (block_id, boss, diffs) in run {
        for diff in combine_getdiff_results(diffs) {
            writes::edit_block(boss.clone(), block_id.clone(), diff, session_id.to_string())
                .await?;
        }
    }
    Ok(())
}

fn lock_app_after_failed_flush() {
    if !APP_LOCKED.get_untracked() {
        APP_LOCKED.set(true);
        crate::leptos_components::small_components::popup::create_popup(
            "Data corrupted. Copy any recent changes you need, then refresh the page.".to_string(),
        );
    }
}
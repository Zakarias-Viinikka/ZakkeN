use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::shared_owners::focus;
use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids::current_session_id;
use crate::leptos_components::small_components::popup::show_popup;
use crate::page_state::OpenPages;

use super::on_delete_request::block_above;

pub fn install(batcher: BatcherHandle, open: OpenPages) -> Callback<(String, String)> {
    Callback::new(move |(block_id, current_text): (String, String)| {
        let page = open.page1;
        let blocks = page.blocks.get_untracked();
        let Some(above) = block_above(&blocks, &block_id) else {
            return;
        };
        if above.is_title {
            return;
        }
        let above_id = above.block_id.clone();
        let above_content = above.content.clone();
        let Some(boss) = page.boss_of_yrs.get_untracked() else {
            leptos::logging::error!("on_merge_with_above: no boss for block {}", block_id);
            return;
        };
        let batcher = batcher.clone();
        let session_id = current_session_id();

        spawn_local(async move {
            if above_content.is_empty() {
                delete_above_and_clear_row(batcher, boss, above_id, page, session_id).await;
                return;
            }
            append_into_above_and_delete_this(
                batcher,
                boss,
                above_id,
                above_content,
                block_id,
                current_text,
                page,
                session_id,
            )
            .await;
        });
    })
}

async fn delete_above_and_clear_row(
    batcher: BatcherHandle,
    boss: std::sync::Arc<my_yrs_lib::BossOfYrs>,
    above_id: String,
    page: crate::page_state::Page,
    session_id: String,
) {
    if let Err(e) =
        crate::db::ui_ops::delete_block(batcher, boss, above_id.clone(), session_id).await
    {
        leptos::logging::error!("merge: delete empty above failed: {:?}", e);
        show_popup(format!("merge: delete empty above failed: {e:?}"));
        return;
    }
    page.blocks.update(|rows| {
        rows.retain(|b| b.block_id != above_id);
    });
}

#[allow(clippy::too_many_arguments)]
async fn append_into_above_and_delete_this(
    batcher: BatcherHandle,
    boss: std::sync::Arc<my_yrs_lib::BossOfYrs>,
    above_id: String,
    above_content: String,
    block_id: String,
    current_text: String,
    page: crate::page_state::Page,
    session_id: String,
) {
    let new_above = format!("{}{}", above_content, current_text);
    let diff = text_diff::diff_logic::get_diff(&above_content, &new_above);
    batcher.push_edit(boss.clone(), above_id.clone(), diff);

    if let Err(e) =
        crate::db::ui_ops::delete_block(batcher, boss, block_id.clone(), session_id).await
    {
        leptos::logging::error!("merge: delete current failed: {:?}", e);
        show_popup(format!("merge: delete current failed: {e:?}"));
        return;
    }

    page.blocks.update(|rows| {
        rows.retain(|b| b.block_id != block_id);
        if let Some(b) = rows.iter_mut().find(|b| b.block_id == above_id) {
            b.content = new_above;
        }
    });

    focus::at_end(above_id);
}

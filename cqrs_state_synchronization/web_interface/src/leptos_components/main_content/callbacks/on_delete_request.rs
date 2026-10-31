use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::shared_owners::{focus, BlockRow};
use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids::current_session_id;
use crate::leptos_components::small_components::popup::create_popup;
use crate::page_state::OpenPages;

pub fn install(batcher: BatcherHandle, open: OpenPages) -> Callback<String> {
    Callback::new(move |block_id: String| {
        let page = open.page1;
        let blocks = page.blocks.get_untracked();
        let normal_count = blocks.iter().filter(|b| !b.is_title).count();
        if normal_count <= 1 {
            return;
        }
        let above_id = block_above(&blocks, &block_id).map(|b| b.block_id.clone());
        let Some(boss) = page.boss_of_yrs.get_untracked() else {
            leptos::logging::error!("on_delete_request: no boss for block {}", block_id);
            return;
        };
        let batcher = batcher.clone();
        let session_id = current_session_id();
        spawn_local(async move {
            if let Err(e) =
                crate::db::ui_ops::delete_block(batcher, boss, block_id.clone(), session_id)
                    .await
            {
                leptos::logging::error!("delete block failed: {:?}", e);
                create_popup(format!("delete block failed: {e:?}"));
                return;
            }
            page.blocks.update(|rows| {
                rows.retain(|b| b.block_id != block_id);
            });
            if let Some(aid) = above_id {
                focus::at_end(aid);
            }
        });
    })
}

pub(crate) fn block_above<'a>(blocks: &'a [BlockRow], block_id: &str) -> Option<&'a BlockRow> {
    let idx = blocks.iter().position(|b| b.block_id == block_id)?;
    idx.checked_sub(1).and_then(|i| blocks.get(i))
}

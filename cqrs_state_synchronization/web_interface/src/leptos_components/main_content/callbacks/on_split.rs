use leptos::prelude::*;

use crate::shared_owners::{focus, BlockRow};
use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids::current_session_id;
use crate::leptos_components::small_components::popup::create_popup;
use crate::new_local_block_id::new_local_block_id;
use crate::page_state::OpenPages;

pub fn install(batcher: BatcherHandle, open: OpenPages) -> Callback<(String, String, String)> {
    Callback::new(move |(block_id, before, after): (String, String, String)| {
        let Some((boss, page)) = open.page_and_boss_for_block(&block_id) else {
            leptos::logging::error!("on_split: no page/boss for block {}", block_id);
            create_popup(format!("split failed: no open page has block {block_id}"));
            return;
        };

        let session_id = current_session_id();
        let rows = page.blocks.get_untracked();
        let Some(idx) = rows.iter().position(|b| b.block_id == block_id) else {
            return;
        };
        let old_text = rows[idx].content.clone();
        let block_id_of_below = rows.get(idx + 1).map(|b| b.block_id.clone());
        let page_id = page.page_id.get_untracked();
        let new_block_id = new_local_block_id();

        if old_text != before {
            let split_diff = text_diff::diff_logic::get_diff(&old_text, &before);
            batcher.push_edit(boss.clone(), block_id.clone(), split_diff);
        }
        batcher.push_add_block(
            boss,
            new_block_id.clone(),
            after.clone(),
            my_yrs_lib::yrs_wrapper::PositionToInsert::Between {
                block_id_of_above: Some(block_id.clone()),
                block_id_of_below,
            },
            false,
            session_id,
        );

        insert_row_after(&page, &block_id, before, after, new_block_id.clone(), page_id);
        focus::at_start(new_block_id);
    })
}

fn insert_row_after(
    page: &crate::page_state::Page,
    block_id: &str,
    before: String,
    after: String,
    new_block_id: String,
    page_id: String,
) {
    page.blocks.update(|rows| {
        if let Some(i) = rows.iter().position(|b| b.block_id == block_id) {
            rows[i].content = before;
            rows.insert(
                i + 1,
                BlockRow {
                    block_id: new_block_id,
                    page_id,
                    content: after,
                    is_title: false,
                },
            );
        }
    });
}

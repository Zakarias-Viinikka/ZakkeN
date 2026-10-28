use leptos::prelude::*;

use crate::complicated_signal_flow::BlockInputEvent;
use crate::db::batcher::BatcherHandle;
use crate::leptos_components::small_components::popup::create_popup;
use crate::page_state::OpenPages;

pub fn install(batcher: BatcherHandle, open: OpenPages) -> Callback<BlockInputEvent> {
    Callback::new(move |evt: BlockInputEvent| {
        let block_id = evt.block_id.clone();
        let Some((boss, page)) = open.page_and_boss_for_block(&block_id) else {
            leptos::logging::error!("on_block_input: no page/boss for block {}", block_id);
            create_popup(format!("edit failed: no open page has block {block_id}"));
            return;
        };

        let diff = text_diff::diff_logic::get_diff(&evt.old_text, &evt.new_text);

        let new_text = evt.new_text.clone();
        batcher.push_edit(boss, block_id.clone(), diff);
        page.blocks.update(|rows| {
            if let Some(b) = rows.iter_mut().find(|b| b.block_id == block_id) {
                b.content = new_text;
            }
        });
    })
}

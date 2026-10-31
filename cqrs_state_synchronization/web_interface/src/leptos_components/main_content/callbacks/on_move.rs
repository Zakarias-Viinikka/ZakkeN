use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids::current_session_id;
use crate::drag_reorder_container::MoveEvent;
use crate::leptos_components::small_components::popup::show_popup;
use crate::page_state::OpenPages;

pub fn install(batcher: BatcherHandle, open: OpenPages) -> Callback<MoveEvent> {
    Callback::new(move |ev: MoveEvent| {
        let session_id = current_session_id();

        if ev.from_page_id == ev.to_page_id {
            let batcher = batcher.clone();
            spawn_local(async move {
                if let Err(e) = crate::db::ui_ops::move_block(
                    batcher,
                    ev.to_page_id,
                    ev.moved_block_id,
                    ev.above,
                    ev.below,
                    session_id,
                )
                .await
                {
                    leptos::logging::error!("move block failed: {:?}", e);
                    show_popup(format!("move block failed: {e:?}"));
                }
            });
            return;
        }

        let old_boss = open.boss_for_page(&ev.from_page_id);
        let new_boss = open.boss_for_page(&ev.to_page_id);
        let (Some(old_boss), Some(new_boss)) = (old_boss, new_boss) else {
            leptos::logging::error!(
                "on_move: no boss for page {} or {}",
                ev.from_page_id,
                ev.to_page_id,
            );
            show_popup(format!(
                "cross-page move failed: no open page with id {} or {}",
                ev.from_page_id, ev.to_page_id
            ));
            return;
        };

        let batcher = batcher.clone();
        spawn_local(async move {
            if let Err(e) = crate::db::ui_ops::move_block_to_another_page(
                batcher,
                old_boss,
                new_boss,
                ev.moved_block_id,
                ev.above,
                ev.below,
                session_id,
            )
            .await
            {
                leptos::logging::error!("move block to another page failed: {:?}", e);
                show_popup(format!("move block to another page failed: {e:?}"));
            }
        });
    })
}

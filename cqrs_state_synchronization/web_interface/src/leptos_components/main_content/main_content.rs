use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use my_yrs_lib::BossOfYrs;

use crate::complicated_signal_flow::{BatcherHandle, BlockInputEvent, OpenPages};
use crate::page_state::{Page, PageLoadState};
use crate::db::db_helpers_for_web_client;
use crate::drag_reorder_container::{DragReorderContainer, MoveEvent, PageInput};
use crate::leptos_components::small_components::popup::create_popup;

use super::background_activity_indicator::BackgroundActivityIndicator;
use super::connection_indicator::ConnectionIndicator;
use super::page_loading_logic;

#[component]
pub fn MainContent() -> impl IntoView {
    let open = expect_context::<OpenPages>();
    let page1 = open.page1;
    // Two-page view isn't built yet. page2 stays None; when the UI
    // exists, this reads open.page2.get() reactively.
    let page2: Option<Page> = None;

    let batcher_handle = expect_context::<BatcherHandle>();
    let batcher_handle_for_load = batcher_handle.clone();

    // Tracks both page_id and page_state. page_id changes when the
    // route changes; page_state changes as the load sequence advances.
    Effect::new(move |_| {
        if page1.page_id.get().is_empty() {
            return;
        }
        match page1.page_state.get() {
            PageLoadState::Loading => {
                page_loading_logic::preload_pages(page1);
            }
            PageLoadState::PreloadLoaded(_) => {
                let batcher_handle = batcher_handle_for_load.clone();
                spawn_local(async move {
                    page_loading_logic::load_offline(page1, batcher_handle).await;
                });
            }
            PageLoadState::Loaded => {
                spawn_local(async move {
                    page_loading_logic::load_from_server(page1).await;
                });
            }
            _ => {}
        }
    });

    Effect::new(move |_| {
        if let Some(blocks) = page1.preload_response.get() {
            page1.preload_response.set(None);
            if matches!(page1.page_state.get_untracked(), PageLoadState::Loading) {
                let count = blocks.len();
                page1.blocks.set(blocks);
                page1.page_state.set(PageLoadState::PreloadLoaded(count));
            }
        }
    });

    let pages_for_ui = Signal::derive(move || {
        let pid = page1.page_id.get();
        if pid.is_empty() {
            Vec::new()
        } else {
            vec![PageInput { page_id: pid }]
        }
    });

    let blocks_for_ui = Signal::derive(move || page1.blocks.get());

    let batcher_handle_for_move = batcher_handle.clone();

    let on_move = Callback::new(move |ev: MoveEvent| {
        let batcher_handle = batcher_handle_for_move.clone();
        let session_id = crate::db::cached_ids::SESSION_ID
            .get_untracked()
            .expect(crate::db::cached_ids::INIT_ERR);

        if ev.from_page_id == ev.to_page_id {
            let batcher_handle = batcher_handle.clone();
            spawn_local(async move {
                if let Err(e) = crate::db::ui_actions::move_block(
                    batcher_handle,
                    ev.to_page_id,
                    ev.moved_block_id,
                    ev.above,
                    ev.below,
                    session_id,
                )
                .await
                {
                    leptos::logging::error!("move block failed: {:?}", e);
                    create_popup(format!("move block failed: {e:?}"));
                }
            });
            return;
        }

        let old_boss = get_boss_from_page_id(page1, page2, &ev.from_page_id);
        let new_boss = get_boss_from_page_id(page1, page2, &ev.to_page_id);
        let (Some(old_boss), Some(new_boss)) = (old_boss, new_boss) else {
            leptos::logging::error!(
                "on_move: no boss for page {} or {}",
                ev.from_page_id,
                ev.to_page_id,
            );
            create_popup(format!(
                "cross-page move failed: no open page with id {} or {}",
                ev.from_page_id, ev.to_page_id
            ));
            return;
        };

        spawn_local(async move {
            if let Err(e) = crate::db::ui_actions::move_block_to_another_page(
                batcher_handle,
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
                create_popup(format!("move block to another page failed: {e:?}"));
            }
        });
    });

    let batcher_handle_for_delete = batcher_handle.clone();

    let on_block_input = Callback::new(move |evt: BlockInputEvent| {
        let block_id = evt.block_id.clone();
        let Some((boss, page)) = find_page_and_boss(page1, page2, &block_id) else {
            leptos::logging::error!("on_block_input: no page/boss for block {}", block_id);
            create_popup(format!("edit failed: no open page has block {block_id}"));
            return;
        };
        let batcher_handle = batcher_handle.clone();

        let diff = text_diff::diff_logic::get_diff(&evt.old_text, &evt.new_text);

        if check_if_should_make_new_block(&evt) {
            // Push the \n\n that triggered the split to the batcher before
            // the block is created, so yrs sees the text the user typed.
            // add_block flushes first, so yrs has the new content before
            // the new block is inserted.
            batcher_handle.push_edit(boss.clone(), block_id.clone(), diff);
            let session_id = crate::db::cached_ids::SESSION_ID
                .get_untracked()
                .expect(crate::db::cached_ids::INIT_ERR);
            spawn_local(async move {
                split_block_at_end(page, boss, block_id, batcher_handle, session_id).await;
            });
            return;
        }

        batcher_handle.push_edit(boss, block_id, diff);
    });

    let on_delete_request = Callback::new(move |block_id: String| {
        let page = page1;
        let normal_count = page
            .blocks
            .get_untracked()
            .iter()
            .filter(|b| !b.is_title)
            .count();
        if normal_count <= 1 {
            return;
        }
        let Some(boss) = page.boss_of_yrs.get_untracked() else {
            leptos::logging::error!("on_delete_request: no boss for block {}", block_id);
            return;
        };
        let batcher_handle = batcher_handle_for_delete.clone();
        let session_id = crate::db::cached_ids::SESSION_ID
            .get_untracked()
            .expect(crate::db::cached_ids::INIT_ERR);
        spawn_local(async move {
            if let Err(e) =
                crate::db::ui_actions::delete_block(batcher_handle, boss, block_id.clone(), session_id)
                    .await
            {
                leptos::logging::error!("delete block failed: {:?}", e);
                create_popup(format!("delete block failed: {e:?}"));
                return;
            }
            page.blocks.update(|rows| {
                rows.retain(|b| b.block_id != block_id);
            });
        });
    });

    let state_1 = Signal::derive(move || page1.page_state.get());
    let state_2 = Signal::derive(move || None::<PageLoadState>);

    view! {
        <div style=move || {
            if crate::complicated_signal_flow::APP_LOCKED.get() {
                "background-color: #4a1a1a; color: #ffffff; min-height: 100vh;"
            } else {
                "min-height: 100vh;"
            }
        }>
            <ConnectionIndicator state_1=state_1 state_2=state_2 />
            <BackgroundActivityIndicator />
            <DragReorderContainer
                pages=pages_for_ui
                blocks=blocks_for_ui
                on_move=on_move
                on_block_input=on_block_input
                on_delete_request=on_delete_request
            />
        </div>
    }
}

// ============================================================
// HELPERS
// ============================================================

fn get_boss_from_page_id(
    page1: Page,
    page2: Option<Page>,
    page_id: &str,
) -> Option<Arc<BossOfYrs>> {
    if page1.page_id.get_untracked() == page_id {
        return page1.boss_of_yrs.get_untracked();
    }
    if let Some(p2) = page2 {
        if p2.page_id.get_untracked() == page_id {
            return p2.boss_of_yrs.get_untracked();
        }
    }
    None
}

fn page_for_block(
    page1: Page,
    page2: Option<Page>,
    block_id: &str,
) -> Option<Page> {
    if page1
        .blocks
        .get_untracked()
        .iter()
        .any(|b| b.block_id == block_id)
    {
        return Some(page1);
    }
    if let Some(p2) = page2 {
        if p2
            .blocks
            .get_untracked()
            .iter()
            .any(|b| b.block_id == block_id)
        {
            return Some(p2);
        }
    }
    None
}

fn find_page_and_boss(
    page1: Page,
    page2: Option<Page>,
    block_id: &str,
) -> Option<(Arc<BossOfYrs>, Page)> {
    let page = page_for_block(page1, page2, block_id)?;
    let boss = page.boss_of_yrs.get_untracked()?;
    Some((boss, page))
}

fn check_if_should_make_new_block(evt: &BlockInputEvent) -> bool {
    if !evt.was_enter || evt.old_text.is_empty() {
        return false;
    }
    if !evt.new_text.ends_with("\n\n") {
        return false;
    }
    evt.cursor_pos == evt.new_text.encode_utf16().count()
}

async fn split_block_at_end(
    page: Page,
    boss: Arc<BossOfYrs>,
    block_id: String,
    batcher_handle: BatcherHandle,
    session_id: String,
) {
    let blocks = page.blocks.get_untracked();
    let block_id_of_below = blocks
        .iter()
        .position(|b| b.block_id == block_id)
        .and_then(|i| blocks.get(i + 1))
        .map(|b| b.block_id.clone());

    let position = my_yrs_lib::yrs_wrapper::PositionToInsert::Between {
        block_id_of_above: Some(block_id),
        block_id_of_below,
    };

    match crate::db::ui_actions::add_block(
        batcher_handle,
        boss,
        String::new(),
        String::new(),
        position,
        false,
        session_id,
    )
    .await
    {
        Ok(new_block_id) => {
            leptos::logging::log!("new block created: {}", new_block_id);
            let page_id = page.page_id.get_untracked();
            if let Ok(rows) = db_helpers_for_web_client::get_all_blocks_for_page(page_id).await {
                page.blocks.set(rows);
            }
            crate::complicated_signal_flow::FOCUS_BLOCK_ID.set(Some(new_block_id));
        }
        Err(e) => {
            leptos::logging::error!("add block failed: {:?}", e);
            create_popup(format!("add block failed: {e:?}"));
        }
    }
}
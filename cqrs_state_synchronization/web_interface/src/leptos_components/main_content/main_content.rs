use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use my_yrs_lib::BossOfYrs;

use crate::complicated_signal_flow::{BatcherHandle, BlockInputEvent, BlockRow, OpenPages};
use crate::page_state::{Page, PageLoadState};

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
    let batcher_handle_for_merge = batcher_handle.clone();
    let batcher_handle_for_split = batcher_handle.clone();

    let on_block_input = Callback::new(move |evt: BlockInputEvent| {
        let block_id = evt.block_id.clone();
        let Some((boss, page)) = find_page_and_boss(page1, page2, &block_id) else {
            leptos::logging::error!("on_block_input: no page/boss for block {}", block_id);
            create_popup(format!("edit failed: no open page has block {block_id}"));
            return;
        };
        let batcher_handle = batcher_handle.clone();

        let diff = text_diff::diff_logic::get_diff(&evt.old_text, &evt.new_text);

        let new_text = evt.new_text.clone();
        batcher_handle.push_edit(boss, block_id.clone(), diff);
        page.blocks.update(|rows| {
            if let Some(b) = rows.iter_mut().find(|b| b.block_id == block_id) {
                b.content = new_text;
            }
        });
    });

    // Enter in a non-empty block. `before` stays in this block, `after`
    // becomes the new block. The UI makes the new block right away with
    // an id it chose itself. The DB writes are queued in the batcher,
    // in order, and nothing here waits for them.
    let on_split = Callback::new(move |(block_id, before, after): (String, String, String)| {
        let Some((boss, page)) = find_page_and_boss(page1, page2, &block_id) else {
            leptos::logging::error!("on_split: no page/boss for block {}", block_id);
            create_popup(format!("split failed: no open page has block {block_id}"));
            return;
        };
        let batcher_handle = batcher_handle_for_split.clone();
        let session_id = crate::db::cached_ids::SESSION_ID
            .get_untracked()
            .expect(crate::db::cached_ids::INIT_ERR);
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
            batcher_handle.push_edit(boss.clone(), block_id.clone(), split_diff);
        }
        batcher_handle.push_add_block(
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

        // One update: the old block is trimmed and the new row sits
        // directly after it, so the drag component sees a pure add.
        let new_id_for_row = new_block_id.clone();
        page.blocks.update(|rows| {
            if let Some(i) = rows.iter().position(|b| b.block_id == block_id) {
                rows[i].content = before;
                rows.insert(
                    i + 1,
                    BlockRow {
                        block_id: new_id_for_row,
                        page_id,
                        content: after,
                        is_title: false,
                    },
                );
            }
        });
        crate::complicated_signal_flow::FOCUS_BLOCK_AT_START.set(Some(new_block_id));
    });

    // Up/Down at the edge of a block. Down lands at the start of the
    // block below, Up at the end of the block above.
    let on_navigate = Callback::new(move |(block_id, go_down): (String, bool)| {
        let blocks = page1.blocks.get_untracked();
        let Some(idx) = blocks.iter().position(|b| b.block_id == block_id) else {
            return;
        };
        if go_down {
            if let Some(next) = blocks.get(idx + 1) {
                crate::complicated_signal_flow::FOCUS_BLOCK_AT_START.set(Some(next.block_id.clone()));
            }
        } else if idx > 0 {
            crate::complicated_signal_flow::FOCUS_BLOCK_ID.set(Some(blocks[idx - 1].block_id.clone()));
        }
    });

    let on_delete_request = Callback::new(move |block_id: String| {
        let page = page1;
        let blocks = page.blocks.get_untracked();
        let normal_count = blocks.iter().filter(|b| !b.is_title).count();
        if normal_count <= 1 {
            return;
        }
        let above_id = get_block_above(&blocks, &block_id).map(|b| b.block_id.clone());
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
            if let Some(aid) = above_id {
                crate::complicated_signal_flow::FOCUS_BLOCK_ID.set(Some(aid));
            }
        });
    });

    let on_merge_with_above = Callback::new(move |(block_id, current_text): (String, String)| {
        let page = page1;
        let blocks = page.blocks.get_untracked();
        let Some(above) = get_block_above(&blocks, &block_id) else {
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
        let batcher_handle = batcher_handle_for_merge.clone();
        let session_id = crate::db::cached_ids::SESSION_ID
            .get_untracked()
            .expect(crate::db::cached_ids::INIT_ERR);

        spawn_local(async move {
            if above_content.is_empty() {
                // Above is empty; just delete it. Current block stays
                // focused at position 0 (unchanged).
                if let Err(e) = crate::db::ui_actions::delete_block(
                    batcher_handle,
                    boss,
                    above_id.clone(),
                    session_id,
                )
                .await
                {
                    leptos::logging::error!("merge: delete empty above failed: {:?}", e);
                    create_popup(format!("merge: delete empty above failed: {e:?}"));
                    return;
                }
                page.blocks.update(|rows| {
                    rows.retain(|b| b.block_id != above_id);
                });
                return;
            }

            // Append current_text to above, delete this block.
            let new_above = format!("{}{}", above_content, current_text);
            let diff = text_diff::diff_logic::get_diff(&above_content, &new_above);
            batcher_handle.push_edit(boss.clone(), above_id.clone(), diff);

            if let Err(e) = crate::db::ui_actions::delete_block(
                batcher_handle,
                boss,
                block_id.clone(),
                session_id,
            )
            .await
            {
                leptos::logging::error!("merge: delete current failed: {:?}", e);
                create_popup(format!("merge: delete current failed: {e:?}"));
                return;
            }

            page.blocks.update(|rows| {
                rows.retain(|b| b.block_id != block_id);
                if let Some(b) = rows.iter_mut().find(|b| b.block_id == above_id) {
                    b.content = new_above.clone();
                }
            });

            crate::complicated_signal_flow::FOCUS_BLOCK_ID.set(Some(above_id));
        });
    });

    view! {
        <div style=move || {
            if crate::complicated_signal_flow::APP_LOCKED.get() {
                "background-color: #4a1a1a; color: #ffffff; min-height: 100vh;"
            } else {
                "min-height: 100vh;"
            }
        }>
            <ConnectionIndicator />
            <BackgroundActivityIndicator />
            <DragReorderContainer
                pages=pages_for_ui
                blocks=blocks_for_ui
                on_move=on_move
                on_block_input=on_block_input
                on_delete_request=on_delete_request
                on_merge_with_above=on_merge_with_above
                on_split=on_split
                on_navigate=on_navigate
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

fn get_block_above<'a>(blocks: &'a [BlockRow], block_id: &str) -> Option<&'a BlockRow> {
    let idx = blocks.iter().position(|b| b.block_id == block_id)?;
    idx.checked_sub(1).and_then(|i| blocks.get(i))
}

/// A fresh block id, made in the UI so the block can exist before the
/// DB has heard of it. 128 random bits as hex.
fn new_local_block_id() -> String {
    let mut s = String::with_capacity(32);
    for _ in 0..4 {
        let n = (js_sys::Math::random() * 4294967296.0) as u32;
        s.push_str(&format!("{:08x}", n));
    }
    s
}
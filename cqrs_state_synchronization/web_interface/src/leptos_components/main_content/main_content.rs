use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;

use my_yrs_lib::BossOfYrs;

use crate::complicated_signal_flow::{BlockRow, Page, PageLoadState};
use crate::db::db_helpers_for_web_client;
use crate::leptos_components::small_components::popup::create_popup;
use crate::drag_reorder_container::{DragReorderContainer, MoveEvent, PageInput};

use super::background_activity_indicator::BackgroundActivityIndicator;
use super::connection_indicator::ConnectionIndicator;
use super::page_loading_logic;

#[component]
pub fn MainContent(page_id_1: String, page_id_2: Option<String>) -> impl IntoView {
    let page1 = Page::new(page_id_1);
    let page2 = page_id_2.map(Page::new);

    // page1 state effect — advances one step per state change
    Effect::new(move |_| match page1.page_state.get() {
        PageLoadState::Loading => {
            page_loading_logic::preload_pages(page1);
        }
        PageLoadState::PreloadLoaded(_) => {
            spawn_local(async move {
                page_loading_logic::load_offline(page1).await;
            });
        }
        PageLoadState::Loaded => {
            spawn_local(async move {
                page_loading_logic::load_from_server(page1).await;
            });
        }
        _ => {}
    });

    // page1 response effect — applies blocks when the response signal
    // flips to Some. Only transitions if still Loading, so a late
    // response can't clobber a page that's already loaded offline.
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

    if let Some(p2) = page2 {
        Effect::new(move |_| match p2.page_state.get() {
            PageLoadState::Loading => {
                page_loading_logic::preload_pages(p2);
            }
            PageLoadState::PreloadLoaded(_) => {
                spawn_local(async move {
                    page_loading_logic::load_offline(p2).await;
                });
            }
            PageLoadState::Loaded => {
                spawn_local(async move {
                    page_loading_logic::load_from_server(p2).await;
                });
            }
            _ => {}
        });

        Effect::new(move |_| {
            if let Some(blocks) = p2.preload_response.get() {
                p2.preload_response.set(None);
                if matches!(p2.page_state.get_untracked(), PageLoadState::Loading) {
                    let count = blocks.len();
                    p2.blocks.set(blocks);
                    p2.page_state.set(PageLoadState::PreloadLoaded(count));
                }
            }
        });
    }

    let pages_for_ui = Signal::derive(move || {
        let mut ps = vec![PageInput {
            page_id: page1.page_id.get(),
        }];
        if let Some(p2) = page2 {
            ps.push(PageInput {
                page_id: p2.page_id.get(),
            });
        }
        ps
    });

    let blocks_for_ui = Signal::derive(move || {
        let mut all: Vec<BlockRow> = Vec::new();
        all.extend(page1.blocks.get());
        if let Some(p2) = page2 {
            all.extend(p2.blocks.get());
        }
        all
    });

    let on_move = Callback::new(move |ev: MoveEvent| {
        let session_id = crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR);

        if ev.from_page_id == ev.to_page_id {
            spawn_local(async move {
                if let Err(e) = crate::db::ui_actions::move_block(
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
                "on_move: could not resolve bosses for cross-page move ({} -> {})",
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

    let page1_for_input = page1;
    let page2_for_input = page2;
    let on_block_input = Callback::new(move |(block_id, new_content): (String, String)| {
        let boss_opt = {
            let in_p1 = page1_for_input
                .blocks
                .get_untracked()
                .iter()
                .any(|b| b.block_id == block_id);
            if in_p1 {
                page1_for_input.boss_of_yrs.get_untracked()
            } else if let Some(p2) = page2_for_input {
                let in_p2 = p2
                    .blocks
                    .get_untracked()
                    .iter()
                    .any(|b| b.block_id == block_id);
                if in_p2 {
                    p2.boss_of_yrs.get_untracked()
                } else {
                    None
                }
            } else {
                None
            }
        };

        let Some(boss) = boss_opt else {
            leptos::logging::error!("on_block_input: no boss found for block_id {}", block_id);
            create_popup(format!("edit failed: no open page has block {block_id}"));
            return;
        };

        let session_id = crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR);
        spawn_local(async move {
            if let Err(e) = db_helpers_for_web_client::edit_title_for_page(
                boss,
                block_id,
                new_content,
                session_id,
            )
            .await
            {
                leptos::logging::error!("edit block failed: {:?}", e);
                create_popup(format!("edit block failed: {e:?}"));
            }
        });
    });

    let state_1 = Signal::derive(move || page1.page_state.get());
    let state_2 = Signal::derive(move || page2.map(|p| p.page_state.get()));

    view! {
        <ConnectionIndicator state_1=state_1 state_2=state_2 />
        <BackgroundActivityIndicator />
        <DragReorderContainer
            pages=pages_for_ui
            blocks=blocks_for_ui
            on_move=on_move
            on_block_input=on_block_input
        />
    }
}

// ============================================================
// HELPERS
// ============================================================

/// Gives back the yrs doc for whichever of the two visible pages
/// has that id. None if neither does.
fn get_boss_from_page_id(page1: Page, page2: Option<Page>, page_id: &str) -> Option<Arc<BossOfYrs>> {
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

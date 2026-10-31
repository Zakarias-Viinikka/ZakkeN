use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::db::batcher::BatcherHandle;
use crate::db::cached_ids::current_user_id;
use crate::db::reads_wrapper::{blocks as reads_blocks, pages as reads_pages};
use crate::leptos_components::small_components::popup::show_popup;
use crate::page_state::{Page, PageLoadState};

pub fn install_load_effects(page1: Page, batcher: BatcherHandle) {
    Effect::new(move |_| {
        if page1.page_id.get().is_empty() {
            return;
        }
        match page1.page_state.get() {
            PageLoadState::Loading => preload_pages(page1),
            PageLoadState::PreloadLoaded(_) => {
                let batcher = batcher.clone();
                spawn_local(async move {
                    load_offline(page1, batcher).await;
                });
            }
            PageLoadState::Loaded => {
                spawn_local(async move {
                    load_from_server(page1).await;
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
}

pub fn preload_pages(page: Page) {
    let pid = page.page_id.get_untracked();
    page.preload_request.set(Some(pid));
    page.preload_response.set(Some(Vec::new()));
}

pub async fn load_offline(page: Page, batcher: BatcherHandle) {
    if let Err(e) = batcher.flush_now().await {
        leptos::logging::error!("flush before load_offline failed: {:?}", e);
    }

    let pid = page.page_id.get_untracked();

    let boss = match reads_pages::get_yrs_unblobbed(
        pid.clone(),
        current_user_id(),
        crate::get_time(),
    )
    .await
    {
        Ok(b) => b,
        Err(e) => {
            leptos::logging::error!("load boss failed: {:?}", e);
            show_popup(format!("load page failed: {e:?}"));
            page.page_state.set(PageLoadState::LoadFailed);
            return;
        }
    };
    page.boss_of_yrs.set(Some(boss));

    match reads_blocks::get_all_blocks_for_page(pid).await {
        Ok(rows) => {
            page.blocks.set(rows);
            page.page_state.set(PageLoadState::Loaded);
        }
        Err(e) => {
            leptos::logging::error!("load blocks failed: {:?}", e);
            show_popup(format!("load blocks failed: {e:?}"));
            page.page_state.set(PageLoadState::LoadFailed);
        }
    }
}

pub async fn load_from_server(page: Page) {
    let _ = page;
}
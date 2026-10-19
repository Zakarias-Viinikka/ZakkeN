use leptos::prelude::*;

use crate::complicated_signal_flow::{Page, PageLoadState};
use crate::db::db_helpers_for_web_client;

/// Step 1. Ask for preloaded blocks.
///
/// Writes the page id into preload_request. When the preloader exists,
/// it will read this, fetch whatever it has, and write the result into
/// preload_response. The page's response effect will then seed the
/// blocks and transition to PreloadLoaded(count).
///
/// For now nothing watches preload_request, so we seed
/// preload_response with an empty Vec directly to drive the effect.
/// Remove that line once the preloader exists.
pub fn preload_pages(page: Page) {
    let pid = page.page_id.get_untracked();
    page.preload_request.set(Some(pid));
    // TODO: remove when the preloader exists — currently the only
    // thing that ever writes to preload_response
    page.preload_response.set(Some(Vec::new()));
}

/// Step 2. Read the yrs blob from db, build the BossOfYrs, read every
/// block row for this page, seed the live block signal. Set Loaded on
/// success, LoadFailed on error.
pub async fn load_offline(page: Page) {
    let pid = page.page_id.get_untracked();

    let boss = match db_helpers_for_web_client::get_yrs_unblobbed(
        pid.clone(),
        crate::FAKE_USER_ID.to_string(),
        crate::FAKE_TIME.to_string(),
    )
    .await
    {
        Ok(b) => b,
        Err(e) => {
            leptos::logging::error!("load boss failed: {:?}", e);
            page.page_state.set(PageLoadState::LoadFailed);
            return;
        }
    };
    page.boss_of_yrs.set(Some(boss));

    match db_helpers_for_web_client::get_all_blocks_for_page(pid).await {
        Ok(rows) => {
            page.blocks.set(rows);
            page.page_state.set(PageLoadState::Loaded);
        }
        Err(e) => {
            leptos::logging::error!("load blocks failed: {:?}", e);
            page.page_state.set(PageLoadState::LoadFailed);
        }
    }
}

/// Step 3. Ask the server whether our copy matches.
///
/// Not implemented. The server side of this doesn't exist yet.
pub async fn load_from_server(page: Page) {
    let _ = page;
}
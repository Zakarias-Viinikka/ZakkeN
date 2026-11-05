use leptos::prelude::*;

use crate::shared_owners::APP_LOCKED;
use crate::drag_reorder_container::{DragReorderContainer, PageInput};
use crate::page_state::OpenPages;

use super::background_activity_indicator::BackgroundActivityIndicator;
use super::callbacks;
use super::connection_indicator::ConnectionIndicator;
use super::page_loading_logic;

#[component]
pub fn MainContent() -> impl IntoView {
    let open = expect_context::<OpenPages>();

    page_loading_logic::install_load_effects(open.page1);

    let pages_for_ui = Signal::derive(move || {
        let pid = open.page1.page_id.get();
        if pid.is_empty() {
            Vec::new()
        } else {
            vec![PageInput { page_id: pid }]
        }
    });

    let blocks_for_ui = Signal::derive(move || open.page1.blocks.get());

    let on_move = callbacks::on_move::install(open);
    let on_block_input = callbacks::on_block_input::install(open);
    let on_delete_request = callbacks::on_delete_request::install(open);
    let on_merge_with_above = callbacks::on_merge_with_above::install(open);
    let on_split = callbacks::on_split::install(open);
    let on_navigate = callbacks::on_navigate::install(open);

    view! {
        <div style=move || {
            if APP_LOCKED.get() {
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
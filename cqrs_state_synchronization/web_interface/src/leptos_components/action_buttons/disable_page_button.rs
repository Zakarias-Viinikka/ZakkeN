use leptos::prelude::*;
use leptos::reactive::spawn_local;

use crate::{
    complicated_signal_flow::{BatcherHandle, LocalPages}, db::ui_actions::disable_page,
    leptos_components::small_components::popup::create_popup,
};

const EYE_IMG: &str = r#"<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path><circle cx="12" cy="12" r="3"></circle></svg>"#;

#[component]
pub fn DisablePageButton(
    page_id: String,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) -> impl IntoView {
    let batcher_handle = expect_context::<BatcherHandle>();
    view! {
        <button
            class="peek_button"
            inner_html=EYE_IMG
            on:click=move |_| {
                let page_id = page_id.clone();
                let batcher_handle = batcher_handle.clone();
                spawn_local(async move {
                    match disable_page(batcher_handle, page_id.clone(), crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR)).await {
                        Err(e) => create_popup(format!("Error: {:?}", e)),
                        Ok(()) => {
                            local_pages_set.update(|pages| {
                                for p in pages.iter_mut() {
                                    if p.page_id == page_id {
                                        p.is_disabled = true;
                                    }
                                }
                            });
                        }
                    }
                });
            }
        ></button>
    }
}

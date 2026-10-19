// Demo-only wiring. Not part of the component. Delete this file, or ignore
// it, when adding PageStack to a real project. The only thing this file
// demonstrates that matters is the shape of the three props and the
// on_move callback — everything else here (db, holders, single-page toggle)
// is scaffolding for the demo.

use leptos::prelude::*;
use crate::page_stack::{BlockInput, MoveEvent, PageInput, PageStack};
use crate::schema::{get_all_blocks, get_all_pages, init_db};
use crate::shared_structs::{BlockHolder, PageHolder};

#[component]
pub fn App() -> impl IntoView {
    let all_pages = RwSignal::new(Vec::<PageHolder>::new());
    let all_blocks = RwSignal::new(Vec::<BlockHolder>::new());

    leptos::task::spawn_local(async move {
        if let Err(e) = init_db().await {
            leptos::logging::error!("init_db failed: {:?}", e);
            return;
        }
        match get_all_pages().await {
            Ok(p) => all_pages.set(p),
            Err(e) => leptos::logging::error!("get_all_pages failed: {:?}", e),
        }
        match get_all_blocks().await {
            Ok(b) => all_blocks.set(b),
            Err(e) => leptos::logging::error!("get_all_blocks failed: {:?}", e),
        }
    });

    let single_page = RwSignal::new(false);

    let pages_signal = Signal::derive(move || {
        let ps: Vec<PageInput> = all_pages
            .get()
            .into_iter()
            .map(|p| PageInput { page_id: p.page_id })
            .collect();
        if single_page.get() {
            ps.into_iter().take(1).collect()
        } else {
            ps
        }
    });

    let blocks_signal = Signal::derive(move || {
        let single = single_page.get();
        let first_page: Option<String> = all_pages.get().first().map(|p| p.page_id.clone());
        all_blocks
            .get()
            .into_iter()
            .filter(|b| !single || Some(&b.page_id) == first_page.as_ref())
            .map(|b| BlockInput {
                block_id: b.block_id,
                page_id: b.page_id,
                content: b.content,
            })
            .collect()
    });

    let on_move = Callback::new(|event: MoveEvent| {
        leptos::logging::log!(
            "moved {} from {} to {}; above={:?} below={:?}",
            event.moved_block_id,
            event.from_page_id,
            event.to_page_id,
            event.above,
            event.below
        );
    });

    view! {
        <label class="ps-viz-toggle" style="position: fixed; top: 1rem; left: 1rem; z-index: 10000;">
            <input
                type="checkbox"
                prop:checked=move || single_page.get()
                on:change=move |e| single_page.set(event_target_checked(&e))
            />
            "single page"
        </label>
        <PageStack pages=pages_signal blocks=blocks_signal on_move=on_move />
    }
}

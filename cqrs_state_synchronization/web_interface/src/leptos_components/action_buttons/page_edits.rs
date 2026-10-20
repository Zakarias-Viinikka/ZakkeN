use crate::leptos_components::small_components::popup::create_popup;
use crate::{complicated_signal_flow::LocalPages, db::ui_actions};
use leptos::prelude::*;
use leptos::reactive::spawn_local;
//use leptos_meta::Stylesheet;

fn disable_page(
    selected_title_to_manipulate: i32,
    local_pages: ReadSignal<Vec<LocalPages>>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) {
    let target_id = selected_title_to_manipulate;
    let page_id = match local_pages
        .get()
        .into_iter()
        .find(|p| p.db_id == target_id as usize)
    {
        Some(p) => p.page_id,
        None => {
            create_popup("No page selected".into());
            return;
        }
    };

    spawn_local(async move {
        match ui_actions::disable_page(page_id, crate::db::cached_ids::SESSION_ID.get_untracked().expect(crate::db::cached_ids::INIT_ERR)).await {
            Err(e) => create_popup(format!("Error: {:?}", e)),
            Ok(()) => {
                local_pages_set.update(|pages| {
                    for p in pages.iter_mut() {
                        if p.db_id == target_id as usize {
                            p.is_disabled = true;
                        }
                    }
                });
            }
        }
    });
}

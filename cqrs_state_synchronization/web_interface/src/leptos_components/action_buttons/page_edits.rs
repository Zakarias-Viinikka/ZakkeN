use crate::leptos_components::small_components::popup::create_popup;
use crate::{db::ui_actions, shared_structs::LocalPages};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos_meta::Stylesheet;

fn disable_page(
    selected_title_to_manipulate: i32,
    local_pages: ReadSignal<Vec<LocalPages>>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) {
    let target_id = selected_title_to_manipulate;
    let yrs_id = match local_pages
        .get()
        .into_iter()
        .find(|p| p.id == target_id as usize)
    {
        Some(p) => p.yrs_id,
        None => {
            create_popup("No page selected".into());
            return;
        }
    };

    spawn_local(async move {
        match ui_actions::disable_page(yrs_id, crate::FAKE_SESSION_ID.to_string()).await {
            Err(e) => create_popup(format!("Error: {:?}", e)),
            Ok(()) => {
                local_pages_set.update(|pages| {
                    for p in pages.iter_mut() {
                        if p.id == target_id as usize {
                            p.is_disabled = true;
                        }
                    }
                });
            }
        }
    });
}

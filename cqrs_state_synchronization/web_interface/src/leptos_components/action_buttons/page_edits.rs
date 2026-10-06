use std::collections::HashMap;
use std::sync::Arc;

use crate::leptos_components::small_components::happy_little_checkbox::HappyLittleCheckbox;
use crate::{checkbox_logic::*, db::ui_actions, shared_structs::LocalPages};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos_meta::Stylesheet;
use crate::leptos_components::small_components::popup::create_popup;

#[component]
pub fn PageEdits(
    current_title_ctr: RwSignal<usize>,
    local_pages: ReadSignal<Vec<LocalPages>>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) -> impl IntoView {
    let (all_checkboxes, map_checkbox_to_callback) =
        AllCheckboxes::new(current_title_ctr, local_pages_set);

    let run_all_seeds =
        create_run_all_seeds_signal(all_checkboxes.checkboxes, map_checkbox_to_callback);
    let selected_title_to_manipulate = RwSignal::new(0);

    view! {
        <Stylesheet href="/css/action_buttons_styling.css" />
        <div class="check_box_container">
            <button on:click=move |_| {
                ui_actions::delete_everything(current_title_ctr, local_pages_set);
                run_all_seeds.set(true);
            }>
            "this will reset db and reload page"
            </button>
            <span class="this_should_take_up_rest_of_width"></span>
            <HappyLittleCheckbox
                checkbox=checkbox_from_map(all_checkboxes.checkboxes.get_untracked(), "insert_three_pages")
            />
            "this will create 3 pages"
        </div>
        <div class="action_info">
            <span>"Current title 'Title: " {move || current_title_ctr.get()} "'" </span> <br/>
            <span>"Current title selected for edits: " {move || selected_title_to_manipulate.get()} </span>
        </div>
        <div class="action_buttons_container">
            <span class="action_buttons_title"> "title 1"</span>
            <button on:click=move |_| {
                spawn_local(async move {
                    ui_actions::create_new_page(current_title_ctr, local_pages_set).await;
                });
            }>
            "Append New Page with 'Title X'"
            </button>
            <button on:click=move |_| disable_page(selected_title_to_manipulate.get(), local_pages, local_pages_set)>"Disable Page"</button>
            <button>"Moving (Todo)"</button>

            <span class="action_buttons_title"> "change selected title"</span>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr += 1)}>"number up"</button>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr -= 1)}>"number down"</button>
        </div>
    }
}

fn create_run_all_seeds_signal(
    all_checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>,
    map_checkbox_to_callback: Arc<CheckboxToCallbackMap>,
) -> RwSignal<bool> {
    let run_all_seeds = RwSignal::new(false);

    Effect::new(move |_| {
        if run_all_seeds.get() {
            let map = map_checkbox_to_callback.clone();
            spawn_local(async move {
                ui_actions::run_all_seeds(all_checkboxes, &map).await;
            });
        }
    });

    run_all_seeds
}

const PANIC_MSG: &str = "panic from checkbox_from_map in page_edits.rs | this should mean that the key is incorrect or not properly initialized in checkbox_logic.rs";
fn checkbox_from_map(
    all_checkboxes: HashMap<String, RwSignal<Checkbox>>,
    key: &str,
) -> RwSignal<Checkbox> {
    all_checkboxes
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("{} | key = {}", PANIC_MSG, key))
}

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

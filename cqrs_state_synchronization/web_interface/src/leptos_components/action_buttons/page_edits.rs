use std::collections::HashMap;

use crate::leptos_components::small_components::happy_little_checkbox::HappyLittleCheckbox;
use crate::{checkbox_logic::*, db::ui_actions, shared_structs::LocalPages};
use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::Stylesheet;

#[component]
pub fn PageEdits(
    current_title_ctr: RwSignal<usize>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) -> impl IntoView {
    let (all_checkboxes, map_checkbox_to_callback) =
        AllCheckboxes::new(current_title_ctr, local_pages_set);

    let selected_title_to_manipulate = RwSignal::new(0);

    view! {
        <Stylesheet href="/css/action_buttons_styling.css" />
        <div class="check_box_container">
            <button on:click=move |_| {
                ui_actions::delete_everything();
                /*
                 * pub fn run_all_seeds(checkboxes: RwSignal<HashMap<String, RwSignal<Checkbox>>>, map_of_callbacks: HashMap<String, Box<dyn Fn() + Send + Sync>>)
                 */
                ui_actions::run_all_seeds(all_checkboxes.checkboxes, map_checkbox_to_callback);
                //reload page
            }>
            "this will reset db and reload page"
            </button>
            <span class="this_should_take_up_rest_of_width"></span>
            <HappyLittleCheckbox
                checkbox=checkbox_from_map(all_checkboxes.checkboxes.get(), "insert_three_pages") //the key can be found in checkbox_logic.rs
            />
        </div>
        <div class="action_info">
            <span>"Current title 'Title: " {move || current_title_ctr.get()} "'" </span> <br/>
            <span>"Current title selected for edits: " {move || selected_title_to_manipulate.get()} </span>
        </div>
        <div class="action_buttons_container">
            <span class="action_buttons_title"> "title 1"</span>
            <button on:click=move |_| ui_actions::create_new_page(current_title_ctr, local_pages_set)>"Append New Page with 'Title X'"</button>
            <button on:click=move |_| delete_page(selected_title_to_manipulate)>"Delete Page"</button>
            <button>"Moving (Todo)"</button>

            //title 2
            <span class="action_buttons_title"> "change selected title"</span>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr += 1)}>"number up"</button>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr -= 1)}>"number down"</button>
        </div>
    }
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

fn thing_for_check_box_to_do() {
    log!("truth");
}

fn delete_page(page_ctr: RwSignal<i32>) {
    log!("deleted page 'Title {}'", page_ctr.get());
}

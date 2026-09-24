use crate::leptos_components::small_components::happy_little_checkbox::HappyLittleCheckbox;
use codee::string::JsonSerdeCodec;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::Stylesheet;
use leptos_use::storage::use_local_storage;

struct AllCheckBoxStates {
    placeholder: Signal<bool>,
    placeholder_set: WriteSignal<bool>,
}

impl AllCheckBoxStates {
    fn new() -> Self {
        let (placeholder, placeholder_set, _) =
            use_local_storage::<bool, JsonSerdeCodec>("placeholder_checkbox");
        Self {
            placeholder,
            placeholder_set,
        }
    }
}

#[component]
pub fn PageEdits() -> impl IntoView {
    let all_checkbox_states = AllCheckBoxStates::new();

    let current_title_ctr = RwSignal::new(0);
    let selected_title_to_manipulate = RwSignal::new(0);
    view! {
        <Stylesheet href="/css/action_buttons_styling.css" />
        <div class="check_box_container">
        /*
         * pub fn HappyLittleCheckbox(
         *     box_is_checked: ReadSignal(bool),
         *     box_is_checked_set: WriteSignal(bool),
         *     speak_your_truth: impl Fn() + 'static,
         * ) -> impl IntoView
         */
            /*<HappyLittleCheckbox
                box_is_checked=all_checkbox_states.placeholder
                box_is_checked_set=all_checkbox_states.placeholder_set
                speak_your_truth=thing_for_check_box_to_do
            />*/
        </div>
        <div class="action_info">
            <span>"Current title 'Title: " {move || current_title_ctr.get()} "'" </span> <br/>
            <span>"Current title selected for edits: " {move || selected_title_to_manipulate.get()} </span>
        </div>
        <div class="action_buttons_container">
            <span class="action_buttons_title"> "title 1"</span>
            <button on:click=move |_| create_new_page(current_title_ctr)>"Append New Page with 'Title X'"</button>
            <button on:click=move |_| delete_page(selected_title_to_manipulate)>"Delete Page"</button>
            <button>"Moving (Todo)"</button>

            //title 2
            <span class="action_buttons_title"> "change selected title"</span>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr += 1)}>"number up"</button>
            <button on:click=move |_| {selected_title_to_manipulate.update(|ctr| *ctr -= 1)}>"number down"</button>
        </div>
    }
}

fn thing_for_check_box_to_do() {
    log!("truth");
}

fn create_new_page(page_ctr: RwSignal<i32>) {
    page_ctr.update(|ctr| *ctr += 1);
    log!("created new page 'Title {}'", page_ctr.get());
}

fn delete_page(page_ctr: RwSignal<i32>) {
    log!("deleted page 'Title {}'", page_ctr.get());
}

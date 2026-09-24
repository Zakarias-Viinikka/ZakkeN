use codee::string::JsonSerdeCodec;
use data_builder_for_operations_that_need_to_be_correct::create_page;
use executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db::page_executor::insert_page_requires_three_db_inserts;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
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

#[derive(Clone)]
pub struct LocalPages {
    pub title: String,
    pub id: usize,
    pub yrs_id: String,
}

#[component]
pub fn PageEdits(
    current_title_ctr: RwSignal<usize>,
    local_pages_set: WriteSignal<Vec<LocalPages>>,
) -> impl IntoView {
    let all_checkbox_states = AllCheckBoxStates::new();

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
            <button on:click=move |_| create_new_page(current_title_ctr, local_pages_set)>"Append New Page with 'Title X'"</button>
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

fn create_new_page(page_ctr: RwSignal<usize>, local_pages: WriteSignal<Vec<LocalPages>>) {
    let session_id = "placeholder-session".to_string();
    let page_id = RwSignal::new("".to_string());
    spawn_local(async move {
        let everything = match create_page(true, session_id) {
            Ok(v) => v,
            Err(e) => {
                log!("create_page failed: {:?}", e);
                return;
            }
        };
        page_id.set(everything.page_to_insert.page_id.clone());
        if let Err(e) = insert_page_requires_three_db_inserts(everything).await {
            log!("insert failed: {:?}", e);
        }
    });

    local_pages.update(|pages| {
        pages.push(LocalPages {
            title: "".to_string(),
            id: page_ctr.get() as usize,
            yrs_id: page_id.get(),
        });
    });

    page_ctr.update(|ctr| *ctr += 1);

    crate::leptos_components::small_components::popup::create_popup("Created Page".into());
}

fn delete_page(page_ctr: RwSignal<i32>) {
    log!("deleted page 'Title {}'", page_ctr.get());
}

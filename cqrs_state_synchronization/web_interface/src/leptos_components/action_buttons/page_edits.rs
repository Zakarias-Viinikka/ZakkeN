use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::Stylesheet;

#[component]
pub fn PageEdits() -> impl IntoView {
    let current_title_ctr = RwSignal::new(0);
    let selected_title_to_manipulate = RwSignal::new(0);
    view! {
        <Stylesheet href="/css/action_buttons_styling.css" />
        <div class="action_info">
            <span>"Current title nr: " {move || current_title_ctr.get()} </span> <br/>
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

fn create_new_page(page_ctr: RwSignal<i32>) {
    page_ctr.update(|ctr| *ctr += 1);
    log!("created new page 'Title {}'", page_ctr.get());
}

fn delete_page(page_ctr: RwSignal<i32>) {
    log!("deleted page 'Title {}'", page_ctr.get());
}

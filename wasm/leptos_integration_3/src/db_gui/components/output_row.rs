use leptos::prelude::*;

#[component]
pub fn OutputRow() -> impl IntoView {
    view! {
        <div class="output-row">
            <div id="table_dump" class="table-dumb"></div>
            <div id="log"></div>
        </div>
    }
}

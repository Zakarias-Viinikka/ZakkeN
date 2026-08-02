use leptos::prelude::*;

#[component]
pub fn DeleteTableSection() -> impl IntoView {
    view! {
        <section>
            <h2>"delete table"</h2>
            <div id="delete-table-list" class="table-chip-list"></div>
        </section>
    }
}

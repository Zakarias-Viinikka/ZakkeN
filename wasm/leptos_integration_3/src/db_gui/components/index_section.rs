use leptos::prelude::*;

#[component]
pub fn IndexSection() -> impl IntoView {
    view! {
        <section>
            <h2>"index column"</h2>
            <label for="index-table">"table name"</label>
            <input id="index-table" type="text" />
            <label for="index-col">"column name"</label>
            <input id="index-col" type="text" />
            <button>"Add index"</button>
        </section>
    }
}

use leptos::prelude::*;

#[component]
pub fn EditSection() -> impl IntoView {
    view! {
        <section>
            <h2>"edit"</h2>
            <label for="edit-row-id">"row id"</label>
            <input id="edit-row-id" type="text" />
            <label for="edit-col">"column name"</label>
            <input id="edit-col" type="text" />
            <label for="edit-val">"new value"</label>
            <input id="edit-val" type="text" />
            <button>"Edit row"</button>
        </section>
    }
}

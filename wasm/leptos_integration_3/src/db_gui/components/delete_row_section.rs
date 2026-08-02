use leptos::prelude::*;

#[component]
pub fn DeleteRowSection() -> impl IntoView {
    view! {
        <section>
            <h2>"delete"</h2>
            <label for="delete-row-id">"row id"</label>
            <input id="delete-row-id" type="text" />
            <button class="danger">"Delete row"</button>
        </section>
    }
}

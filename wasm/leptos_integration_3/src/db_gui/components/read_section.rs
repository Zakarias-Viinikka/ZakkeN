use leptos::prelude::*;

#[component]
pub fn ReadSection() -> impl IntoView {
    view! {
        <section>
            <h2>"read"</h2>
            <label for="read-table">"table name"</label>
            <select id="read-table"></select>
            <button>"Get data"</button>
            <button>"Check table"</button>
        </section>
    }
}

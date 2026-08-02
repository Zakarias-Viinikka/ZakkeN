use leptos::prelude::*;

#[component]
pub fn InsertSection() -> impl IntoView {
    view! {
        <section id="insert-section">
            <h2>"insert"</h2>
            <div id="insert-fields"></div>
            <button>"Insert"</button>
        </section>
    }
}

use leptos::prelude::*;

#[component]
pub fn CreateTableSection() -> impl IntoView {
    view! {
        <section>
            <h2>"create table"</h2>
            <label for="new-table-name">"table name"</label>
            <input id="new-table-name" type="text" />
            <button>"Add column"</button>
            <div id="new-table-columns"></div>
            <button>"Create table"</button>
        </section>
    }
}

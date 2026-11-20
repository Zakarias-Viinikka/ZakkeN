use leptos::prelude::*;

#[component]
pub fn ReadSection(
    set_table_selection: WriteSignal<String>,
    table_names: ReadSignal<Vec<String>>,
) -> impl IntoView {
    view! {
        <section>
            <h2>"read"</h2>
            <label for="read-table">"table name"</label>
            <select
                on:change:target=move |ev| {
                    set_table_selection.set(ev.target().value());
                }
                id="read-table"
            >
                <For
                    each=move || table_names.get()
                    key=|name| name.clone()
                    let:name
                >
                    <option value={name.clone()}>{name.clone()}</option>
                </For>
            </select>
        </section>
    }
}

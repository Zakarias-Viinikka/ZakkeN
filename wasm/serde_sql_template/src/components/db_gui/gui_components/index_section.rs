use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_rusqlite::public_data_shapes::TableColumnInfo;

#[component]
pub fn IndexSection(table_names: ReadSignal<Vec<String>>) -> impl IntoView {
    let selected_table = RwSignal::new(String::new());
    let selected_column = RwSignal::new(String::new());

    let table_columns = LocalResource::new(move || {
        let table = selected_table.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }
            // TODO: call check_table API
            leptos::logging::log!("TODO: check_table for index section");
            Vec::<TableColumnInfo>::new()
        }
    });

    let add_index = move |_| {
        let table = selected_table.get();
        let col = selected_column.get();

        spawn_local(async move {
            // TODO: call create_index API
            let _ = (table, col);
            leptos::logging::log!("TODO: create_index");
        });
    };

    view! {
        <section>
            <h2>"index column"</h2>
            <label for="index-table">"table name"</label>
            <select
                id="index-table"
                on:change:target=move |ev| {
                    selected_table.set(ev.target().value());
                    selected_column.set(String::new());
                }
                prop:value=move || selected_table.get()
            >
                <For
                    each=move || table_names.get()
                    key=|name| name.clone()
                    let:name
                >
                    <option value={name.clone()}>{name.clone()}</option>
                </For>
            </select>
            <label for="index-col">"column name"</label>
            <select
                id="index-col"
                on:change:target=move |ev| selected_column.set(ev.target().value())
                prop:value=move || selected_column.get()
            >
                <For
                    each=move || table_columns.get().unwrap_or_default()
                    key=|col: &TableColumnInfo| col.name.clone()
                    let:col
                >
                    <option value={col.name.clone()}>{col.name.clone()}</option>
                </For>
            </select>
            <button type="button" on:click=add_index>"Add index"</button>
        </section>
    }
}

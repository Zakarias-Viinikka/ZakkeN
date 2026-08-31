use leptos::prelude::*;
use leptos::task::spawn_local;
use protocol::new_table::ColumnDef;

use crate::ask_wrapper;

#[derive(Clone)]
struct ColumnRow {
    id: usize,
    collapsed: RwSignal<bool>,
    name: RwSignal<String>,
    col_type: RwSignal<String>,
    primary_key: RwSignal<bool>,
    not_null: RwSignal<bool>,
    unique: RwSignal<bool>,
    default_value: RwSignal<String>,
    autoincrement: RwSignal<bool>,
}

impl ColumnRow {
    fn new(id: usize) -> Self {
        Self {
            id,
            collapsed: RwSignal::new(true),
            name: RwSignal::new(String::new()),
            col_type: RwSignal::new(String::new()),
            primary_key: RwSignal::new(false),
            not_null: RwSignal::new(true),
            unique: RwSignal::new(false),
            default_value: RwSignal::new(String::new()),
            autoincrement: RwSignal::new(false),
        }
    }

    fn to_column_def(&self) -> ColumnDef {
        ColumnDef {
            name: self.name.get(),
            column_type: self.col_type.get(),
            primary_key: self.primary_key.get(),
            not_null: self.not_null.get(),
            unique: self.unique.get(),
            default_value: self.default_value.get(),
            autoincrement: self.autoincrement.get(),
        }
    }
}

#[component]
pub fn CreateTableSection(set_table_names: WriteSignal<Vec<String>>) -> impl IntoView {
    let table_name = RwSignal::new(String::new());
    let default_id_column = {
        let row = ColumnRow::new(0);
        row.name.set("id".to_string());
        row.col_type.set("INTEGER".to_string());
        row.primary_key.set(true);
        row.not_null.set(true);
        row.autoincrement.set(true);
        row
    };
    let (columns, set_columns) = signal(vec![default_id_column]);
    let next_id = RwSignal::new(1usize);

    let add_column = move |_| {
        let id = next_id.get();
        set_columns.update(|cols| cols.push(ColumnRow::new(id)));
        next_id.update(|n| *n += 1);
    };

    let create_table = move |_| {
        let name = table_name.get();
        let cols: Vec<ColumnDef> = columns.get().iter().map(ColumnRow::to_column_def).collect();

        spawn_local(async move {
            match ask_wrapper::create_table(&name, &cols).await {
                Ok(()) => match ask_wrapper::list_tables().await {
                    Ok(out) => set_table_names.set(out.table_names),
                    Err(e) => leptos::logging::log!("list_tables refresh failed: {:?}", e),
                },
                Err(e) => leptos::logging::log!("create_table failed: {:?}", e),
            }
        });
    };

    view! {
        <section>
            <h2>"create table"</h2>
            <label for="new-table-name">"table name"</label>
            <input
                id="new-table-name"
                type="text"
                on:input:target=move |ev| table_name.set(ev.target().value())
                prop:value=move || table_name.get()
            />
            <button type="button" on:click=add_column>"Add column"</button>
            <div id="new-table-columns">
                <For
                    each=move || columns.get()
                    key=|row| row.id
                    let:row
                >
                    <ColumnRowView row=row />
                </For>
            </div>
            <button type="button" on:click=create_table>"Create table"</button>
        </section>
    }
}

#[component]
fn ColumnRowView(row: ColumnRow) -> impl IntoView {
    view! {
        <div class="column-row">
            <div
                class="column-row-header"
                style="display:flex; align-items:center; gap:8px; cursor:pointer;"
                on:click=move |_| row.collapsed.update(|c| *c = !*c)
            >
                <span>{move || if row.collapsed.get() { "▶" } else { "▼" }}</span>
                <span style="flex:1;">
                    {move || {
                        let n = row.name.get();
                        if n.is_empty() { "(unnamed)".to_string() } else { n }
                    }}
                </span>
            </div>
            <Show when=move || !row.collapsed.get() fallback=|| ()>
                <input
                    type="text"
                    placeholder="column name"
                    on:input:target=move |ev| row.name.set(ev.target().value())
                    prop:value=move || row.name.get()
                />
                <select
                    on:change:target=move |ev| row.col_type.set(ev.target().value())
                    prop:value=move || row.col_type.get()
                >
                    <option value="INTEGER">"INTEGER"</option>
                    <option value="TEXT">"TEXT"</option>
                    <option value="REAL">"REAL"</option>
                    <option value="BLOB">"BLOB"</option>
                    <option value="NUMERIC">"NUMERIC"</option>
                </select>
                <label>
                    <input
                        type="checkbox"
                        on:change:target=move |ev| row.primary_key.set(ev.target().checked())
                        prop:checked=move || row.primary_key.get()
                    />
                    "primary key"
                </label>
                <label>
                    <input
                        type="checkbox"
                        on:change:target=move |ev| row.not_null.set(ev.target().checked())
                        prop:checked=move || row.not_null.get()
                    />
                    "not null"
                </label>
                <label>
                    <input
                        type="checkbox"
                        on:change:target=move |ev| row.unique.set(ev.target().checked())
                        prop:checked=move || row.unique.get()
                    />
                    "unique"
                </label>
                <input
                    type="text"
                    placeholder="default value"
                    on:input:target=move |ev| row.default_value.set(ev.target().value())
                    prop:value=move || row.default_value.get()
                />
                <label>
                    <input
                        type="checkbox"
                        on:change:target=move |ev| row.autoincrement.set(ev.target().checked())
                        prop:checked=move || row.autoincrement.get()
                    />
                    "autoincrement"
                </label>
            </Show>
        </div>
    }
}

use super::LogEntry;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_rusqlite::public_data_shapes::TableColumnInfo;
use wasm_rusqlite::table_row::Col;

#[component]
pub fn InsertSection(
    table_selection: ReadSignal<String>,
    table_columns: LocalResource<Vec<TableColumnInfo>>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    table_dump_refresh: RwSignal<u32>,
) -> impl IntoView {
    let values = RwSignal::new(Vec::<(String, RwSignal<String>)>::new());

    Effect::new(move || {
        if let Some(cols) = table_columns.get() {
            leptos::logging::log!("cols: {}", cols.len());
            let new_values: Vec<(String, RwSignal<String>)> = cols
                .iter()
                .filter(|c| !c.primary_key)
                .map(|c| (c.name.clone(), RwSignal::new(String::new())))
                .collect();
            values.set(new_values);
        }
    });

    let do_insert = move |_| {
        let table = table_selection.get();
        let vals = values.get();

        spawn_local(async move {
            // TODO: build Vec<ColumnValue> from vals and call insert API
            let _ = (table, vals);
            leptos::logging::log!("TODO: insert data");
            table_dump_refresh.update(|n| *n += 1);
            set_log_entries.update(|log| {
                log.push(LogEntry {
                    tag: "insert".to_string(),
                    message: "TODO".to_string(),
                })
            });
        });
    };

    view! {
        <section id="insert-section">
            <h2>"insert"</h2>
            <div id="insert-fields">
                <For
                    each=move || values.get()
                    key=|(name, _)| name.clone()
                    let((name, val_signal))
                >
                    <div class="insert-field">
                        <label>{name}</label>
                        <input
                            type="text"
                            on:input:target=move |ev| val_signal.set(ev.target().value())
                            prop:value=move || val_signal.get()
                        />
                    </div>
                </For>
            </div>
            <button type="button" on:click=do_insert>"Insert"</button>
        </section>
    }
}

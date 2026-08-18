use crate::db_gui::components::LogEntry;
use crate::local_sqlite::local_sqlite_wrapper;
use crate::local_sqlite::local_sqlite_wrapper::TableColumnInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

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
        let col_names: Vec<String> = vals.iter().map(|(name, _)| name.clone()).collect();
        let raw_vals: Vec<String> = vals.iter().map(|(_, sig)| sig.get()).collect();

        spawn_local(async move {
            match local_sqlite_wrapper::insert_data(&table, &col_names, &raw_vals).await {
                Ok(()) => {
                    set_log_entries.update(|log| {
                        log.push(LogEntry {
                            tag: "insert".to_string(),
                            message: "ok".to_string(),
                        })
                    });
                    table_dump_refresh.update(|n| *n += 1);
                }
                Err(e) => set_log_entries.update(|log| {
                    log.push(LogEntry {
                        tag: "insert".to_string(),
                        message: format!("{:?}", e),
                    })
                }),
            }
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

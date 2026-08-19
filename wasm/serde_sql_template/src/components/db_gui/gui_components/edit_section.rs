use super::LogEntry;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn EditSection(
    table_selection: ReadSignal<String>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    table_dump_refresh: RwSignal<u32>,
) -> impl IntoView {
    let row_id = RwSignal::new(String::new());
    let column = RwSignal::new(String::new());
    let new_value = RwSignal::new(String::new());

    let do_edit = move |_| {
        let table = table_selection.get();
        let id = row_id.get();
        let col = column.get();
        let val = new_value.get();

        spawn_local(async move {
            match crate::ask_wrapper::edit_col_in_row(
                &table,
                &id,
                &col,
                wasm_rusqlite::table_row::Col::Text(val),
            )
            .await
            {
                Ok(()) => {
                    table_dump_refresh.update(|n| *n += 1);
                    set_log_entries.update(|log| {
                        log.push(LogEntry {
                            tag: "edit_row".to_string(),
                            message: "ok".to_string(),
                        })
                    });
                }
                Err(e) => {
                    set_log_entries.update(|log| {
                        log.push(LogEntry {
                            tag: "edit_row".to_string(),
                            message: format!("{:?}", e),
                        })
                    });
                }
            }
        });
    };

    view! {
        <section>
            <h2>"edit"</h2>
            <label for="edit-row-id">"row id"</label>
            <input
                id="edit-row-id"
                type="text"
                on:input:target=move |ev| row_id.set(ev.target().value())
                prop:value=move || row_id.get()
            />
            <label for="edit-col">"column name"</label>
            <input
                id="edit-col"
                type="text"
                on:input:target=move |ev| column.set(ev.target().value())
                prop:value=move || column.get()
            />
            <label for="edit-val">"new value"</label>
            <input
                id="edit-val"
                type="text"
                on:input:target=move |ev| new_value.set(ev.target().value())
                prop:value=move || new_value.get()
            />
            <button type="button" on:click=do_edit>"Edit row"</button>
        </section>
    }
}

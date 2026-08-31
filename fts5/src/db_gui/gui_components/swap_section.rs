use crate::ask_wrapper;

//use crate::db_gui::gui_components::LogEntry;
use super::LogEntry;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn SwapSection(
    table_selection: ReadSignal<String>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    table_dump_refresh: RwSignal<u32>,
) -> impl IntoView {
    let row_id_1 = RwSignal::new(String::new());
    let row_id_2 = RwSignal::new(String::new());
    let column = RwSignal::new(String::new());

    let do_swap = move |_| {
        let table = table_selection.get();
        let id1 = row_id_1.get();
        let id2 = row_id_2.get();
        let col = column.get();

        spawn_local(async move {
            match ask_wrapper::swap_columns(&table, &id1, &id2, &col).await {
                Ok(()) => {
                    table_dump_refresh.update(|n| *n += 1);
                    set_log_entries.update(|log| {
                        log.push(LogEntry {
                            tag: "swap_columns".to_string(),
                            message: "ok".to_string(),
                        })
                    });
                }
                Err(e) => {
                    set_log_entries.update(|log| {
                        log.push(LogEntry {
                            tag: "swap_columns".to_string(),
                            message: format!("{:?}", e),
                        })
                    });
                }
            }
        });
    };

    view! {
        <section>
            <h2>"swap"</h2>
            <label for="swap-row-id-1">"row id 1"</label>
            <input
                id="swap-row-id-1"
                type="text"
                on:input:target=move |ev| row_id_1.set(ev.target().value())
                prop:value=move || row_id_1.get()
            />
            <label for="swap-row-id-2">"row id 2"</label>
            <input
                id="swap-row-id-2"
                type="text"
                on:input:target=move |ev| row_id_2.set(ev.target().value())
                prop:value=move || row_id_2.get()
            />
            <label for="swap-col">"column name"</label>
            <input
                id="swap-col"
                type="text"
                on:input:target=move |ev| column.set(ev.target().value())
                prop:value=move || column.get()
            />
            <button type="button" on:click=do_swap>"Swap columns"</button>
        </section>
    }
}

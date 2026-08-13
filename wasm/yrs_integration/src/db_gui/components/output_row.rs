use crate::local_sqlite::local_sqlite_wrapper::TableColumnInfo;
use leptos::prelude::*;

#[derive(Clone)]
pub struct LogEntry {
    pub tag: String,
    pub message: String,
}

#[component]
pub fn OutputRow(
    log_entries: ReadSignal<Vec<LogEntry>>,
    table_dump: LocalResource<Vec<Vec<String>>>,
    table_columns: LocalResource<Vec<TableColumnInfo>>,
) -> impl IntoView {
    view! {
        <div class="output-row">
            <div id="table_dump" class="table-dumb">
                <table class="data-table">
                    <tr>
                        <For
                            each=move || table_columns.get().unwrap_or_default()
                            key=|col| col.name.clone()
                            let:col
                        >
                            <th>{col.name.clone()}</th>
                        </For>
                    </tr>
                    <For
                        each=move || table_dump.get().unwrap_or_default()
                        key=|row| row.join(", ")
                        let:row
                    >
                        <tr>
                            <For
                                each=move || row.clone()
                                key=|cell| cell.clone()
                                let:cell
                            >
                                <td>{cell}</td>
                            </For>
                        </tr>
                    </For>
                </table>
            </div>
            <div id="log">
                <For
                    each=move || log_entries.get()
                    key=|entry| format!("{}-{}", entry.tag, entry.message)
                    let:entry
                >
                    <div><span class="tag">{format!("[{}] ", entry.tag)}</span>{entry.message}</div>
                </For>
            </div>
        </div>
    }
}

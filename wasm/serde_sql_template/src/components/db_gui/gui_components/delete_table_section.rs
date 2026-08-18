use super::LogEntry;
use super::confirm_modal::PendingConfirm;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn DeleteTableSection(
    table_names: ReadSignal<Vec<String>>,
    set_table_names: WriteSignal<Vec<String>>,
    pending_confirm: RwSignal<Option<PendingConfirm>>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
) -> impl IntoView {
    view! {
        <section>
            <h2>"delete table"</h2>
            <div id="delete-table-list" class="table-chip-list">
                <For
                    each=move || table_names.get()
                    key=|name| name.clone()
                    let:name
                >
                    <button
                        type="button"
                        class="table-chip"
                        on:click=move |_| {
                            let table = name.clone();
                            let table_for_confirm = table.clone();
                            pending_confirm.set(Some(PendingConfirm {
                                message: format!("Delete table \"{}\"? This can't be undone.", table),
                                on_confirm: Callback::new(move |_| {
                                    let table = table_for_confirm.clone();
                                    spawn_local(async move {
                                        // TODO: call drop_table API
                                        let _ = table;
                                        leptos::logging::log!("TODO: drop_table");
                                        set_log_entries.update(|log| log.push(LogEntry {
                                            tag: "drop_table".to_string(),
                                            message: "TODO".to_string(),
                                        }));
                                        // TODO: refresh table_names
                                        set_table_names.set(Vec::new());
                                    });
                                }),
                            }));
                        }
                    >
                        {name.clone()}
                    </button>
                </For>
            </div>
        </section>
    }
}

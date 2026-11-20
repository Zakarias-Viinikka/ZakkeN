use super::LogEntry;
use super::confirm_modal::PendingConfirm;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn DeleteRowSection(
    table_selection: ReadSignal<String>,
    pending_confirm: RwSignal<Option<PendingConfirm>>,
    set_log_entries: WriteSignal<Vec<LogEntry>>,
    table_dump_refresh: RwSignal<u32>,
) -> impl IntoView {
    let row_id = RwSignal::new(String::new());

    let confirm_delete = move |_| {
        let table = table_selection.get();
        let id = row_id.get();
        let id_for_confirm = id.clone();

        pending_confirm.set(Some(PendingConfirm {
            message: format!(
                "Delete row {} from \"{}\"? This can't be undone.",
                id, table
            ),
            on_confirm: Callback::new(move |_| {
                let table = table.clone();
                let id = id_for_confirm.clone();
                spawn_local(async move {
                    match crate::ask_wrapper::delete_row(&table, &id).await {
                        Ok(()) => {
                            table_dump_refresh.update(|n| *n += 1);
                            set_log_entries.update(|log| {
                                log.push(LogEntry {
                                    tag: "delete_row".to_string(),
                                    message: "ok".to_string(),
                                })
                            });
                        }
                        Err(e) => {
                            set_log_entries.update(|log| {
                                log.push(LogEntry {
                                    tag: "delete_row".to_string(),
                                    message: format!("{:?}", e),
                                })
                            });
                        }
                    }
                });
            }),
        }));
    };

    view! {
        <section>
            <h2>"delete"</h2>
            <label for="delete-row-id">"row id"</label>
            <input
                id="delete-row-id"
                type="text"
                on:input:target=move |ev| row_id.set(ev.target().value())
                prop:value=move || row_id.get()
            />
            <button type="button" class="danger" on:click=confirm_delete>"Delete row"</button>
        </section>
    }
}

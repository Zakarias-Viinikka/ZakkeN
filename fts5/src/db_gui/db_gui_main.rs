use super::db_gui_assets::DbGuiAssets;
use super::gui_components::*;
use super::tmp_back_ground::TmpBackGround;
use crate::ask_wrapper;
use leptos::prelude::*;
use leptos::task::spawn_local;
use protocol::payload::SelectArgument;

#[component]
pub fn DbGui() -> impl IntoView {
    let (loaded, set_loaded) = signal(false);
    let (table_selection, set_table_selection) = signal("".to_string());
    let (table_names, set_table_names) = signal(Vec::<String>::new());
    let (log_entries, set_log_entries) = signal(Vec::<LogEntry>::new());
    let table_dump_refresh = RwSignal::new(0u32);
    let pending_confirm = RwSignal::new(None::<PendingConfirm>);

    let table_columns = LocalResource::new(move || {
        let table = table_selection.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }

            match ask_wrapper::check_table(&table).await {
                Ok(out) => out.columns,
                Err(e) => {
                    leptos::logging::log!("check_table failed: {:?}", e);
                    Vec::new()
                }
            }
        }
    });

    let table_dump = LocalResource::new(move || {
        let table = table_selection.get();
        table_dump_refresh.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }

            match ask_wrapper::get_data(&table, vec![SelectArgument::All], vec![]).await {
                Ok(out) => out.rows.iter().map(|row| row.to_string_vec()).collect(),
                Err(e) => {
                    leptos::logging::log!("get_data failed: {:?}", e);
                    Vec::new()
                }
            }
        }
    });

    Effect::new(move || {
        spawn_local(async move {
            match ask_wrapper::list_tables().await {
                Ok(out) => {
                    let names = out.table_names;

                    if table_selection.get_untracked().is_empty() {
                        if let Some(first) = names.first() {
                            set_table_selection.set(first.clone());
                        }
                    }

                    set_table_names.set(names);
                }
                Err(e) => leptos::logging::log!("list_tables failed: {:?}", e),
            }
        });
    });

    view! {
        <div class="db-gui">
            <Show when=move || !loaded.get() fallback=|| ()>
                <TmpBackGround />
            </Show>
            <DbGuiAssets loaded=set_loaded />
            <Show when=move || loaded.get() fallback=|| ()>
                <div style="display:flex; align-items:center; gap:12px; margin-bottom:20px;">
                    <h1 style="margin:0;">
                        <span id="status"></span>
                        "db test harness"
                    </h1>
                </div>
                <div class="grid">
                    <InsertSection
                        table_selection=table_selection
                        table_columns=table_columns
                        set_log_entries=set_log_entries
                        table_dump_refresh=table_dump_refresh
                    />
                    <ReadSection set_table_selection=set_table_selection table_names=table_names />
                    <EditSection
                        table_selection=table_selection
                        set_log_entries=set_log_entries
                        table_dump_refresh=table_dump_refresh
                    />
                    <SwapSection
                        table_selection=table_selection
                        set_log_entries=set_log_entries
                        table_dump_refresh=table_dump_refresh
                    />
                    <DeleteRowSection
                        table_selection=table_selection
                        pending_confirm=pending_confirm
                        set_log_entries=set_log_entries
                        table_dump_refresh=table_dump_refresh
                    />
                    <CreateTableSection set_table_names=set_table_names />
                    <DeleteTableSection
                        table_names=table_names
                        set_table_names=set_table_names
                        pending_confirm=pending_confirm
                        set_log_entries=set_log_entries
                    />
                    <ConfirmModal pending_confirm=pending_confirm />
                    <IndexSection table_names=table_names />
                </div>
                <OutputRow
                    log_entries=log_entries
                    table_dump=table_dump
                    table_columns=table_columns
                />
            </Show>
        </div>
    }
}

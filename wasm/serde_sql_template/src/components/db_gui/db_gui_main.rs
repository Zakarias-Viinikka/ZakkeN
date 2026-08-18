use super::db_gui_assets::DbGuiAssets;
use super::gui_components::*;
use super::tmp_back_ground::TmpBackGround;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_rusqlite::public_data_shapes::TableColumnInfo;

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

            // TODO: call check_table API here and return real columns
            leptos::logging::log!("TODO: check_table for {table}");
            Vec::<TableColumnInfo>::new()
        }
    });

    let table_dump = LocalResource::new(move || {
        let table = table_selection.get();
        table_dump_refresh.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }

            // TODO: call get_data API here and return real rows
            leptos::logging::log!("TODO: get_data for {table}");
            Vec::<Vec<String>>::new()
        }
    });

    Effect::new(move || {
        spawn_local(async move {
            // TODO: call list_tables API here and set real table names
            leptos::logging::log!("TODO: list_tables");
            set_table_names.set(Vec::new());

            if table_selection.get_untracked().is_empty() {
                if let Some(first) = table_names.get_untracked().first() {
                    set_table_selection.set(first.clone());
                }
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

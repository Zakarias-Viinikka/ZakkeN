use super::components::*;
use super::db_gui_assets::DbGuiAssets;
use super::tmp_back_ground::TmpBackGround;
use crate::local_sqlite::local_sqlite_wrapper;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn DbGui() -> impl IntoView {
    // true once the JS/CSS assets for this page have finished loading
    let (loaded, set_loaded) = signal(false);

    // which table is currently picked in the "read" dropdown — most sections read from this
    let (table_selection, set_table_selection) = signal("".to_string());

    // list of every table name in the db — used to fill dropdowns and chip lists
    let (table_names, set_table_names) = signal(Vec::<String>::new());

    // running list of action results shown in the log panel (insert ok, errors, etc.)
    let (log_entries, set_log_entries) = signal(Vec::<LogEntry>::new());

    // bump this number to force the table dump panel to refetch its data
    let table_dump_refresh = RwSignal::new(0u32);

    // holds the message + action for the confirm modal; None means the modal is hidden
    let pending_confirm = RwSignal::new(None::<PendingConfirm>);

    // fetches the column list for whichever table is selected (used by insert fields)
    let table_columns = LocalResource::new(move || {
        let table = table_selection.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }
            match local_sqlite_wrapper::check_table(&table).await {
                Ok(cols) => cols,
                Err(e) => {
                    leptos::logging::log!("check_table failed: {:?}", e);
                    Vec::new()
                }
            }
        }
    });

    // fetches all rows for whichever table is selected — reruns when table_dump_refresh changes
    let table_dump = LocalResource::new(move || {
        let table = table_selection.get();
        table_dump_refresh.get();
        async move {
            if table.is_empty() {
                return Vec::new();
            }
            match local_sqlite_wrapper::get_data(&table, "", &["".to_string()]).await {
                Ok(rows) => rows,
                Err(e) => {
                    leptos::logging::log!("get_data failed: {:?}", e);
                    Vec::new()
                }
            }
        }
    });

    // on page load: fetch table names and auto-select the first one
    Effect::new(move || {
        spawn_local(async move {
            match local_sqlite_wrapper::list_tables().await {
                Ok(names) => {
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
            <Show
                when=move || !loaded.get()
                fallback=|| ()
            >
                <TmpBackGround />
            </Show>
            <DbGuiAssets loaded=set_loaded />
            <Show
                when=move || loaded.get()
                fallback=|| ()
            >
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
                    <ReadSection
                        set_table_selection=set_table_selection
                        table_names=table_names
                    />
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
                    <CreateTableSection
                        set_table_names=set_table_names
                    />
                    <DeleteTableSection
                        table_names=table_names
                        set_table_names=set_table_names
                        pending_confirm=pending_confirm
                        set_log_entries=set_log_entries
                    />
                    <ConfirmModal
                        pending_confirm=pending_confirm
                    />
                    <IndexSection
                        table_names=table_names
                    />
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

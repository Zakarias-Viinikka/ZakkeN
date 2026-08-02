use super::components::*;
use super::db_gui_assets::DbGuiAssets;
use super::tmp_back_ground::TmpBackGround;
use leptos::prelude::*;

#[component]
pub fn DbGui() -> impl IntoView {
    let (loaded, set_loaded) = signal(false);
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
                    <InsertSection />
                    <ReadSection />
                    <EditSection />
                    <SwapSection />
                    <DeleteRowSection />
                    <CreateTableSection />
                    <DeleteTableSection />
                    <ConfirmModal />
                    <IndexSection />
                </div>
                <OutputRow />
            </Show>
        </div>
    }
}

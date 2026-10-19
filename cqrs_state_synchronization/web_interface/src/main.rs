use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use web_interface::db::init;
use web_interface::leptos_components::main_content::MainContent;
use web_interface::leptos_components::menu::Menu;
use web_internal_db::db_gui::db_gui_main;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let tables_ready = RwSignal::new(false);

    Effect::new(move |_| {
        spawn_local(async move {
            init::create_all_tables().await;
            tables_ready.set(true);
        });
    });

    view! {
        {move || {
            if !tables_ready.get() {
                return view! { <div>"Loading..."</div> }.into_any();
            }
            view! {
                <Router>
                    <main>
                        <Routes fallback=|| "Not found.">
                            <Route path=path!("/") view=Menu/>
                            <Route path=path!("/Menu") view=Menu/>
                            <Route path=path!("/DbGui") view=db_gui_main::DbGui/>
                            <Route path=path!("/MainContent/:page_id") view=MainContentRoute/>
                        </Routes>
                    </main>
                </Router>
            }.into_any()
        }}
    }
}

#[component]
fn MainContentRoute() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    view! {
        {move || {
            let page_id = params.get().get("page_id").unwrap_or_default();
            if page_id.is_empty() {
                return view! { <div>"No page id in URL"</div> }.into_any();
            }
            view! {
                <MainContent page_id_1=page_id page_id_2=None />
            }.into_any()
        }}
    }
}

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use web_interface::db::{cached_ids, init};
use web_interface::leptos_components::main_content::MainContent;
use web_interface::leptos_components::menu::Menu;
use web_interface::leptos_components::setup_page::SetupPage;
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
            init::init_all().await;
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
                            <Route path=path!("/setup") view=SetupPage/>
                        </Routes>
                        <UserGate/>
                    </main>
                </Router>
            }.into_any()
        }}
    }
}

#[component]
fn UserGate() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let location = leptos_router::hooks::use_location();

    Effect::new(move |_| {
        let path = location.pathname.get();
        if cached_ids::USER_ID.get_untracked().is_none() && path != "/setup" {
            navigate("/setup", Default::default());
        }
    });

    ()
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

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use web_interface::complicated_signal_flow::{Batcher, OpenPages};
use web_interface::page_state::{Page, PageLoadState};
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

    // Batcher lives for the app's lifetime. Handle goes into leptos
    // context so MainContent can pull it out.
    let (batcher, batcher_handle) = Batcher::new();
    provide_context(batcher_handle);
    spawn_local(async move {
        batcher.run().await;
    });

    let open_pages = OpenPages {
        page1: Page::new(String::new()),
        page2: RwSignal::new(None),
    };
    provide_context(open_pages);

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
    let open = expect_context::<OpenPages>();

    Effect::new(move |_| {
        let page_id = params.get().get("page_id").unwrap_or_default();
        if page_id.is_empty() {
            return;
        }
        if open.page1.page_id.get_untracked() == page_id {
            return;
        }
        //this is a leptos method, unrelated to text diff batcher
        batch(move || {
            open.page1.page_id.set(page_id);
            open.page1.boss_of_yrs.set(None);
            open.page1.blocks.set(Vec::new());
            open.page1.preload_request.set(None);
            open.page1.preload_response.set(None);
            open.page1.page_state.set(PageLoadState::Loading);
            open.page2.set(None);
        });
    });

    view! { <MainContent/> }
}

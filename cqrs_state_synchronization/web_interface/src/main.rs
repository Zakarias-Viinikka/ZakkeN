use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use web_interface::db::batcher::BatcherHandle;
use web_interface::db::{cached_ids, init};
use web_interface::expose_methods_main;
use web_interface::leptos_components::main_content::MainContent;
use web_interface::leptos_components::menu::Menu;
use web_interface::leptos_components::custom_port_setting_page::CustomPortSettingPage;
use web_interface::leptos_components::setup_guard::SetupGuard;
use web_interface::leptos_components::setup_page::SetupPage;
use web_interface::leptos_components::small_components::popup::CoolPopup;
use web_interface::page_state::{OpenPages, Page, PageLoadState};

use web_internal_db::db_gui::db_gui_main;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let tables_ready = RwSignal::new(false);

    let batcher_handle = BatcherHandle::spawn_task_and_get_handle();
    let batcher_for_link = batcher_handle.clone();
    provide_context(batcher_handle);

    let open_pages = OpenPages {
        page1: Page::new(String::new()),
        page2: RwSignal::new(None),
    };
    provide_context(open_pages);

    Effect::new(move |_| {
        spawn_local(async move {
            init::init_all().await;
            cached_ids::INIT_DONE.set(true);
            tables_ready.set(true);
        });
    });

    let batcher_for_connect = batcher_for_link.clone();
    Effect::new(move |_| {
        let _ = cached_ids::INIT_DONE.get();
        let _ = cached_ids::SESSION_ID.get();
        let _ = cached_ids::USER_ID.get();
        let _ = web_interface::owner_of_interacting_with_server::CONNECTING_GATE.get();
        web_interface::owner_of_interacting_with_server::connect_to_socket(
            batcher_for_connect.clone(),
            open_pages,
        );
    });

    expose_methods_main!();

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
                            <Route path=path!("/custom_port_setting_page") view=CustomPortSettingPage/>
                        </Routes>
                        <SetupGuard/>
                        <CoolPopup/>
                        <MainForNavigatingFromTheTester/>
                    </main>
                </Router>
            }.into_any()
        }}
    }
}



#[component]
fn MainForNavigatingFromTheTester() -> impl IntoView {
    web_interface::method_navigate!();
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
        //this is a leptos batch. not related to text_diff batcher
        // expresses: "let me do all these changes and only run related effects once im done"
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
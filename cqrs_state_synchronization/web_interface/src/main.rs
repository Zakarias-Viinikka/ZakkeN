use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use web_interface::leptos_components::menu::Menu;
use web_internal_db::db_gui::db_gui_main;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=Menu/>
                    <Route path=path!("/Menu") view=Menu/>
                    <Route path=path!("/DbGui") view=db_gui_main::DbGui/>
                    // <Route path=path!("/ViewPage") view=pages::view_page::ViewPage/>
                </Routes>
            </main>
        </Router>
    }
}

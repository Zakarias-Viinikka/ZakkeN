use connecting_to_server_template::app_main::AppMain;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

fn main() {
    console_error_panic_hook::set_once();
    // trunk serve --open
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=AppMain/>
                    //<Route path=path!("/page1") view=page1::Page1/>
                </Routes>
            </main>
        </Router>
    }
}

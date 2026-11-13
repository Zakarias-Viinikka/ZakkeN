use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

mod app_main;
mod chooser;
mod pick_screen;
mod placeholder_data;
mod resolve_choice;
mod viewer_host;

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
                    <Route path=path!("/") view=app_main::AppMain/>
                    // <Route path=path!("/page1") view=page1::Page1/>
                </Routes>
            </main>
        </Router>
    }
}

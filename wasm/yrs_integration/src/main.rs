use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use leptos_integration_3::app::text_blocks_page::TextBlocksPage;
use leptos_integration_3::db_gui::db_gui_main::DbGui;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>
                // (You can style this later)
            </nav>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=DefaultPage/>
                    <Route path=path!("/textblocks") view=TextBlocksPage/>
                    <Route path=path!("/dbgui") view=DbGui/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn DefaultPage() -> impl IntoView {
    view! {
        <br/> <br/> <br/>
        <h2>"Navigation"</h2>
        <A href="/textblocks">"Text Blocks Page"</A>
        <br/>
        <A href="/dbgui">"DB Admin GUI"</A>
    }
}

use leptos::prelude::*;

use leptos_router::components::A; // for making <A> work
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path; // for the path!() macro

use csr_testing::simple_notion_page::page::SimpleNotionPage;

fn main() {
    console_error_panic_hook::set_once();
    //  trunk serve --open

    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>
            /* ... */
            </nav>
            <main>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=DefaultPage/>
                    <Route path=path!("/yrs_combined_with_ui_text_blocks") view=SimpleNotionPage/>
                </Routes>
            </main>
      </Router>
    }
}

#[component]
fn DefaultPage() -> impl IntoView {
    view! {
        <div class="container">
            <h2>"Pick where u wanna go boss"</h2>
            <A href="/yrs_combined_with_ui_text_blocks">"Simple Notion Page"</A>
        </div>
    }
}

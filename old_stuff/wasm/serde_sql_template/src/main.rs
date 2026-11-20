use leptos::prelude::*;
use leptos_router::components::A; // for making <A> work
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path; // for the path!() macro
use serde_sql_template::components::db_gui::db_gui_main::DbGui;
use serde_sql_template::components::*;

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
                <Route path=path!("/") view=index::IndexPage/>
                <Route path=path!("/dbgui") view=DbGui/>
                //<Route path=path!("/temp") view=Page1/>
            </Routes>
        </main>
      </Router>
    }
}

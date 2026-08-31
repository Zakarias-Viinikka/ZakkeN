use fts5::db_gui::db_gui_main;
use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path; // for the path!() macro
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(Main);
}

#[component]
fn Main() -> impl IntoView {
    view! {
      <Router>
        <nav>
          /* ... */
        </nav>
        <main>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=fts5::app::App/>
                <Route path=path!("/dbgui") view=db_gui_main::DbGui/>        //<- both work
                //<Route path=path!("/page2") view=page2::Page2/> //<- both work
            </Routes>
        </main>
      </Router>
    }
}

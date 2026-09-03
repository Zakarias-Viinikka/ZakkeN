use leptos::prelude::*;
use leptos_router::components::A; // for making <A> work
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path; // for the path!() macro

//use leptos_starter::routing::r2_folder_routing::page1_and_page2::{page1::Page1, page2};
use nav_page::fun_drag::UltimateParent;

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
                <Route path=path!("/") view=DefaultPage/>
                <Route path=path!("/fun_drag") view=UltimateParent/>
            </Routes>
        </main>
      </Router>
    }
}

#[component]
fn DefaultPage() -> impl IntoView {
    view! {
        <br/>   <br/>   <br/>
        <h2> "Default Page" </h2>
        <A href="/fun_drag">"Test for dragging boxes around"</A>
        //<br/>
        //<A href="/page2">"Page 2"</A>
    }
}

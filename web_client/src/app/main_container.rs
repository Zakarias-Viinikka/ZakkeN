use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use popup::popup::PopupContainer;

use crate::app::home::cmp::Home;

#[component]
pub fn AppContainer() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Not found.">
                <Route path=path!("/") view=Home/>
            </Routes>
        </Router>
        <div style="position: absolute; top: 8px; right: 8px; z-index: 1000;">
            <PopupContainer/>
        </div>
    }
}

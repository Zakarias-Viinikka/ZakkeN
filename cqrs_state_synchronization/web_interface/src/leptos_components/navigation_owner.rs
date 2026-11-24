//! `<NavigationOwner />`: no visible output. The one place that navigates for plain code.
//! Must be mounted inside `<Router>`. Watches `complex_state/navigation_request.rs`.
use leptos::prelude::*;

use crate::complex_state::navigation_request::{
    plexget_navigation_request, plexset_clear_navigation_request,
};

#[component]
pub fn NavigationOwner() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    Effect::new(move |_| {
        let Some(path) = plexget_navigation_request() else {
            return;
        };
        plexset_clear_navigation_request();
        navigate(&path, Default::default());
    });
    ()
}

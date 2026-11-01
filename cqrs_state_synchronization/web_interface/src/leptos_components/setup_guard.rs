use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};

use crate::db::cached_ids;

/// Paths that are allowed to render without a `USER_ID` set.
const PATHS_THAT_SKIP_SETUP: &[&str] = &["/setup", "/custom_port_setting_page"];

/// Redirects to `/setup` when `USER_ID` is missing and the current path
/// isn't one of `PATHS_THAT_SKIP_SETUP`.
#[component]
pub fn SetupGuard() -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();

    Effect::new(move |_| {
        let path = location.pathname.get();
        if cached_ids::USER_ID.get_untracked().is_none()
            && !PATHS_THAT_SKIP_SETUP.contains(&path.as_str())
        {
            navigate("/setup", Default::default());
        }
    });

    ()
}

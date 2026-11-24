//! Complex state: the ONE pending request "navigate the app to this path".
//!
//! Who touches it:
//! - `markdown_conversion/main_for_on_link_click/page_nav.rs`: writes a request with
//!   `plexset_navigation_request` when a `[[PageNav|page id]]` link is clicked.
//! - `leptos_components/navigation_owner.rs` (`NavigationOwner`): reads it (tracked, in an
//!   Effect), clears it, and tells the router to go there. Only a component inside the router
//!   can navigate, which is why plain code goes through this file.
//!
//! A new request overwrites an unhandled one.
use std::sync::LazyLock;

use leptos::prelude::*;

static NAVIGATION_REQUEST: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

pub fn plexget_navigation_request() -> Option<String> {
    NAVIGATION_REQUEST.get()
}

/// `path` is an app path like "/MainContent/<page id>".
pub fn plexset_navigation_request(path: String) {
    NAVIGATION_REQUEST.set(Some(path));
}

pub fn plexset_clear_navigation_request() {
    NAVIGATION_REQUEST.set(None);
}

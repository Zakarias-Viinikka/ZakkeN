//! Caller pov: the page's render when the client isn't in lonewolf.
//! There's nothing to reconcile if the drift versions already match, or
//! if the socket isn't up.

use leptos::prelude::*;

use crate::owner_of_interacting_with_server::ConnState;


#[component]
pub fn NotLonewolfView(state: ConnState) -> impl IntoView {
    view! {
        <div class="rp-notlone">
            <strong>"Not in lonewolf state"</strong>
            "Reconcile is only available when this client is out of sync with "
            "the server (lonewolf). Connect and let the server wipe the dataset, "
            "or trigger a drift some other way, then come back. "
            "Current connection: " <code>{format!("{state:?}")}</code>
        </div>
    }
}
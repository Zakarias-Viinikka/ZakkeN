//! Caller pov: the page's render when the client isn't in lonewolf.
//! There's nothing to reconcile if the drift versions already match, or
//! if the socket isn't up.

use leptos::prelude::*;

use crate::owner_of_interacting_with_server::ConnState;

const CSS: &str = r#"
.rp-notlone {
    max-width: 560px;
    background: #1c2433;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-left: 3px solid #8ab4ff;
    border-radius: 10px;
    padding: 18px 22px;
    color: rgba(228, 233, 242, 0.8);
    font-size: 14px;
    line-height: 1.55;
}
.rp-notlone strong {
    display: block;
    color: #e4e9f2;
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 6px;
}
.rp-notlone code {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    background: rgba(228, 233, 242, 0.08);
    padding: 2px 6px;
    border-radius: 4px;
    color: #8ab4ff;
}
"#;

#[component]
pub fn NotLonewolfView(state: ConnState) -> impl IntoView {
    view! {
        <style>{CSS}</style>
        <div class="rp-notlone">
            <strong>"Not in lonewolf state"</strong>
            "Reconcile is only available when this client is out of sync with "
            "the server (lonewolf). Connect and let the server wipe the dataset, "
            "or trigger a drift some other way, then come back. "
            "Current connection: " <code>{format!("{state:?}")}</code>
        </div>
    }
}
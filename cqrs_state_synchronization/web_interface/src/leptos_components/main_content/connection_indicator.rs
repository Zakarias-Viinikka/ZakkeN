use leptos::prelude::*;

use crate::complicated_signal_flow::PageLoadState;

#[component]
pub fn ConnectionIndicator(
    state_1: Signal<PageLoadState>,
    state_2: Signal<Option<PageLoadState>>,
) -> impl IntoView {
    view! {
        <div class="connection-indicator">
            {move || format!("Page 1: {:?}", state_1.get())}
            {move || state_2.get().map(|s| format!("  |  Page 2: {:?}", s)).unwrap_or_default()}
        </div>
    }
}

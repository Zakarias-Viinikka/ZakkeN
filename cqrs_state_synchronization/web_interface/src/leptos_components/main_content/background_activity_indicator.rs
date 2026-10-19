use leptos::prelude::*;

/// Shows what's happening in the background — love letters arriving,
/// applying, saving. Useful for debugging. Currently a placeholder;
/// when the batcher and love-letter flow are wired up, this will read
/// from a shared log signal and list the most recent events.
#[component]
pub fn BackgroundActivityIndicator() -> impl IntoView {
    view! {
        <div class="background-activity-indicator">
            // TODO: render activity events. Source signal TBD.
        </div>
    }
}

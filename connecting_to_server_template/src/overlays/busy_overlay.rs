use leptos::prelude::*;

use crate::overlays::full_screen_blocker::FullScreenBlocker;

const BO_CSS: &str = include_str!("../../css/busy_overlay.css");

#[component]
pub fn BusyOverlay() -> impl IntoView {
    let busy = crate::ui_ops::busy::dataset_op_busy();
    view! {
        <style>{BO_CSS}</style>
        {move || busy.get().map(|msg| view! {
            <FullScreenBlocker>
                <div class="bo-card">{msg}</div>
            </FullScreenBlocker>
        })}
    }
}

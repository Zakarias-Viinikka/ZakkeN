use leptos::prelude::*;

#[component]
pub fn NavContent() -> impl IntoView {
    view! {
        <div
            style:flex = "1"                 // take all remaining width
            style:min-width = "0"            // prevent overflow issues
            style:min-height = "100%"
            style:background = "rgba(255,255,255,0.06)"
            style:backdrop-filter = "blur(20px)"
            style:border-radius = "16px"
            style:border-left = "4px solid #5a9aa8"
            style:box-shadow = "0 8px 32px rgba(0,0,0,0.2), inset 0 1px 0 rgba(255,255,255,0.15), inset 0 0 24px rgba(74,121,132,0.15)"
            style:box-sizing = "border-box"
            style:padding = "var(--space6)"
            style:color = "#e0e0e0"
        ></div>
    }
}

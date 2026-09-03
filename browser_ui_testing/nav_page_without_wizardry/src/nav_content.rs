use leptos::prelude::*;
use leptos_meta::Stylesheet;
use leptos_use::{UseWindowSizeReturn, use_window_size};

#[component]
pub fn NavContent() -> impl IntoView {
    let UseWindowSizeReturn { width, .. } = use_window_size();

    const BREAKPOINT: f64 = 800.0;

    view! {
        <Stylesheet href="/css/content_width.css" />
        <div
            style:min-width = "0"
            style:min-height = "100%"
            style:background = "
                linear-gradient(rgba(74,121,132,0.08) 1px, transparent 1px),
                linear-gradient(90deg, rgba(74,121,132,0.08) 1px, transparent 1px),
                rgba(20,25,28,0.75)
            "
            style:background-size = "22px 22px, 22px 22px, auto"
            style:backdrop-filter = "blur(10px)"
            style:border-radius = "16px"
            style:border-left = "4px solid #4a7984"
            style:box-shadow = "0 8px 32px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.08)"
            style:box-sizing = "border-box"
            style:padding = "var(--space6)"
            style:color = "#e0e0e0"

            class=move || {
                if width.get() < BREAKPOINT {
                    "content-zoomed-in-width"
                } else {
                    "content-normal-width"
                }
            }
        ></div>
    }
}

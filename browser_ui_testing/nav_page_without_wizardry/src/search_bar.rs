use leptos::prelude::*;
use leptos_use::use_media_query;

#[component]
pub fn SearchBar() -> impl IntoView {
    let is_mobile = use_media_query("(max-width: 768px)");

    Effect::new(move |_| {
        leptos::logging::log!("is_mobile: {}", is_mobile.get());
    });

    view! {
        <div
            style:min-height = move || if is_mobile.get() { "6vh" } else { "30px" }
            style:flex = "0 0 auto"
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

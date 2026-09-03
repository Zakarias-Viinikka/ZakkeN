use leptos::prelude::*;
use leptos_meta::Stylesheet;
use nav_page_without_wizardry::background_dots::BackgroundDots;
use nav_page_without_wizardry::menu_position::MenuPosition;
use nav_page_without_wizardry::nav_content::NavContent;

fn main() {
    console_error_panic_hook::set_once();
    //  trunk serve --open

    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <GradientBackground />
        <div
            style:display = "flex"
            style:gap = "12px"
            style:width = "1200px"          // decide the width here
            style:margin = "0 auto"         // center the whole container
        >
            <MenuPosition />
            <NavContent />
        </div>
    }
}

#[component]
fn GradientBackground() -> impl IntoView {
    view! {
        <Stylesheet href="/css/background.css" />
        <div
            style:position = "absolute"
            style:top = "0"
            style:left = "0"
            style:width = "100%"
            style:height = "100%"
            style:object-fit = "cover"
            style:z-index = "-1"
            class="gradient-bg"
        >
            <div class="base"></div>
            <div class="treatment"></div>
            <div class="glow"></div>
            <div class="particles"></div>
            <div class="vignette"></div>
            <div class="noise"></div>
            <div class="halftone"></div>
        </div>
        <BackgroundDots />
    }
}

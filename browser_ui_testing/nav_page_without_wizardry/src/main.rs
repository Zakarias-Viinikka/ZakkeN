use leptos::prelude::*;
use leptos_meta::Stylesheet;
use nav_page_without_wizardry::background_dots::BackgroundDots;
use nav_page_without_wizardry::menu_position::MenuPosition;
use nav_page_without_wizardry::nav_content::NavContent;
use nav_page_without_wizardry::search_bar::SearchBar;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <GradientBackground />

        // Outer container: column layout, centered, full height
        <div
            style:display = "flex"
            style:flex-direction = "column"
            style:gap = "12px"
            style:max-width = "1200px"
            style:width = "100%"
            style:margin = "0 auto"
            style:height = "100%"          // fills the available body height
            style:box-sizing = "border-box"
        >
            // Search bar at the top (natural height)
            <SearchBar />

            // Main content row: MenuPosition + NavContent
            <div
                style:display = "flex"
                style:gap = "12px"
                style:align-items = "stretch"
                style:flex = "1"           // take up remaining vertical space
                style:min-height = "0"     // allow inner scroll if needed
            >
                <MenuPosition />
                <NavContent />
            </div>
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

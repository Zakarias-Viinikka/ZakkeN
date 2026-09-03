use leptos::prelude::*;
use leptos_meta::Stylesheet;
use nav_page_without_wizardry::main_nav::MainNavPage;
use nav_page_without_wizardry::menu_position::MenuPosition;

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
            style:position = "relative"
            style:z-index = "1"
            style:display = "flex"
            style:height = "100%"
            style:width = "100%"
        >
            <MenuPosition />
            <MainNavPage />
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
    }
}

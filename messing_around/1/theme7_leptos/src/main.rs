use leptos::prelude::*;

mod boxes;

use boxes::Theme7;

fn main() {
    console_error_panic_hook::set_once();
    // trunk serve --open

    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! { <Theme7 /> }
}
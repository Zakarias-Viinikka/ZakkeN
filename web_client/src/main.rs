use leptos::prelude::*;
use web_client::app::main_container::AppContainer;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(AppContainer);
}

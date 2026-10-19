mod demo_app;
mod page_stack;
mod schema;
mod shared_structs;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(demo_app::App);
}
//! Listens for `connect_to_socket` on `window`. When fired, calls the
//! same connect path the auto-connect effect uses.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('connect_to_socket'))

#[macro_export]
macro_rules! method_connect_to_socket {
    () => {
        let open_for_event =
            leptos::prelude::expect_context::<$crate::page_state::OpenPages>();
        let handle = leptos::prelude::window_event_listener_untyped(
            "connect_to_socket",
            move |_| {
                $crate::owner_of_interacting_with_server::connect_to_socket(open_for_event);
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}
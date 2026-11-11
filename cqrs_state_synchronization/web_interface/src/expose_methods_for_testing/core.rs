//! Entry point for exposing Rust methods to the tester.
//!
//! `expose_methods_main!()` is invoked once from the App component. It
//! calls one `method_*!()` per method we want exposed. Each `method_*!()`
//! sets up a `window` event listener and tears it down via `on_cleanup`.
//!
//! From the browser console or Playwright, fire the event the method
//! listens for. Example for `method_show_popup!()`:
//!
//!     window.dispatchEvent(new CustomEvent('show_popup', { detail: 'hi' }))

#[macro_export]
macro_rules! expose_methods_main {
    () => {
        $crate::method_show_popup!();
        $crate::method_set_port!();
        $crate::method_connect_to_socket!();
        $crate::method_wipe_local_and_pull_from_server!();
        $crate::method_bump_local_drift_version_and_reload!();
        $crate::method_reset_everything!();
        $crate::method_insert_three_pages!();
        $crate::method_wipe_server!();
        $crate::method_read_titles!();
        $crate::method_read_client_state!();
        $crate::method_read_conn_state!();
    };
}
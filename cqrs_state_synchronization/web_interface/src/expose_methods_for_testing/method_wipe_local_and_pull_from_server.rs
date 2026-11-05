//! Listens for `wipe_local_and_pull_from_server` on `window`. Wipes
//! local pages, blocks, pending letters, seen letters, and the stored
//! drift_version, then reloads. On reload, login writes the server's
//! drift_version and bootstrap pulls the server's dataset. Exits
//! lonewolf.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('wipe_local_and_pull_from_server'))

#[macro_export]
macro_rules! method_wipe_local_and_pull_from_server {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "wipe_local_and_pull_from_server",
            move |_| {
                leptos::task::spawn_local(async move {
                    match $crate::db::dev_actions::wipe_local_and_pull_from_server().await {
                        Ok(()) => {
                            $crate::leptos_components::dev_panel::sidebar_buttons::reload_the_page();
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("wipe_local_and_pull_from_server: {e:?}"),
                            );
                        }
                    }
                });
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}

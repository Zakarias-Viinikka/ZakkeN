//! Listens for `reset_everything` on `window`. Wipes every local table
//! (including key_value_storage), then reloads. Client comes back as a
//! fresh install.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('reset_everything'))

#[macro_export]
macro_rules! method_reset_everything {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "reset_everything",
            move |_| {
                leptos::task::spawn_local(async move {
                    match $crate::db::dev_actions::reset_everything().await {
                        Ok(()) => {
                            $crate::leptos_components::dev_panel::sidebar_buttons::reload_the_page();
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("reset_everything: {e:?}"),
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

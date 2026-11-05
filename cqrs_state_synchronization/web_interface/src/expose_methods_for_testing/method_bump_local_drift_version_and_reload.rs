//! Listens for `bump_local_drift_version_and_reload` on `window`.
//! Writes a fake value into the stored drift_version, then reloads. On
//! reload, login compares it against the server's and enters Lonewolf.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('bump_local_drift_version_and_reload'))

#[macro_export]
macro_rules! method_bump_local_drift_version_and_reload {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "bump_local_drift_version_and_reload",
            move |_| {
                leptos::task::spawn_local(async move {
                    match $crate::db::dev_actions::bump_local_drift_version_for_testing().await {
                        Ok(()) => {
                            $crate::leptos_components::dev_panel::sidebar_buttons::reload_the_page();
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("bump_local_drift_version_and_reload: {e:?}"),
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

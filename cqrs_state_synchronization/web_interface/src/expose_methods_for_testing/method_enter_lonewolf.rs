//! Listens for `enter_lonewolf` on `window`. Calls the same action
//! the dev-sidebar deliberate-lonewolf button does.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('enter_lonewolf'))

#[macro_export]
macro_rules! method_enter_lonewolf {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "enter_lonewolf",
            move |_| {
                leptos::task::spawn_local(async move {
                    if let Err(e) = $crate::db::dev_actions::enter_lone_wolf().await {
                        $crate::leptos_components::small_components::popup::show_popup(
                            format!("enter_lonewolf: {e:?}"),
                        );
                    }
                });
            },
        );
        leptos::prelude::on_cleanup(move || {
            handle.remove();
        });
    };
}

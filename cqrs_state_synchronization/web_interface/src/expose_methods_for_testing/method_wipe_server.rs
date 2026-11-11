//! Listens for `wipe_server` on `window`. Sends WipeDataset to the
//! server. Same call as the "wipe server" dev panel button.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('wipe_server'))

#[macro_export]
macro_rules! method_wipe_server {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "wipe_server",
            move |_| {
                leptos::task::spawn_local(async move {
                    use shared_internals::ClientCommand;
                    match $crate::shared_owners::router_handle::send(ClientCommand::WipeDataset).await {
                        Ok(_) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                "server wiped".to_string(),
                            );
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("wipe server failed: {e:?}"),
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

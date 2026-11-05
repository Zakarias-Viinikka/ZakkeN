//! Listens for `insert_three_pages` on `window`. Inserts the three
//! seed pages via `db::dev_actions::insert_three_pages`.
//!
//! From the tester:
//!
//!     window.dispatchEvent(new CustomEvent('insert_three_pages'))

#[macro_export]
macro_rules! method_insert_three_pages {
    () => {
        let handle = leptos::prelude::window_event_listener_untyped(
            "insert_three_pages",
            move |_| {
                leptos::task::spawn_local(async move {
                    match $crate::db::dev_actions::insert_three_pages().await {
                        Ok(()) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                "inserted three pages".to_string(),
                            );
                        }
                        Err(e) => {
                            $crate::leptos_components::small_components::popup::show_popup(
                                format!("insert_three_pages: {e:?}"),
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

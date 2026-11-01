//! Caller pov: the reusable diff viewer. Pass in `left_page` and
//! `right_page` as signals of Vec<String>, an `output` WriteSignal that
//! the viewer will set when the caller asks it to finish, and a
//! `FinishHandle` the caller uses to ask. `on_finished` is optional; if
//! provided, it runs after each finish.
//!
//! The viewer snapshots its inputs on mount. Changing either input
//! signal remounts the inner viewer, so all per-block and per-row state
//! (edits, drags, resets) is fresh — edits and drags never survive an
//! input change. That is deliberate.
//!
//! The only caller setup needed is one stylesheet link in index.html,
//! documented in this crate's lib.rs.
//!
//! Internals pov: an outer component that watches both inputs via a
//! Memo and re-keys a `<For>` on every change. The single-child `<For>`
//! gives us the "remount the inner on input change" behaviour without
//! any custom keying. The inner component (in
//! the_frozen_viewer_for_one_mount.rs) owns the diff computation, the
//! visual order signal, and the finish Effect.

use leptos::prelude::*;

use crate::finish_handle::FinishHandle;
use crate::the_frozen_viewer_for_one_mount::TheFrozenViewerForOneMount;

#[component]
pub fn DiffViewer(
    #[prop(into)] left_page: Signal<Vec<String>>,
    #[prop(into)] right_page: Signal<Vec<String>>,
    #[prop(into)] output: WriteSignal<Vec<String>>,
    handle: FinishHandle,
    #[prop(optional)] on_finished: Option<Callback<()>>,
) -> impl IntoView {
    // Combined key: any change to either page remounts the inner viewer.
    // Using the pair as the key (rather than a monotonic counter) means
    // a set to the same content does NOT remount — leptos's <For> reuses
    // the existing view when the key matches.
    let snapshot_key = Memo::new(move |_| (left_page.get(), right_page.get()));

    view! {
        <For
            each=move || vec![snapshot_key.get()]
            key=|pair: &(Vec<String>, Vec<String>)| pair.clone()
            children=move |pair: (Vec<String>, Vec<String>)| {
                let (left_snapshot, right_snapshot) = pair;
                view! {
                    <TheFrozenViewerForOneMount
                        left=left_snapshot
                        right=right_snapshot
                        output=output
                        handle=handle
                        on_finished=on_finished
                    />
                }
            }
        />
    }
}

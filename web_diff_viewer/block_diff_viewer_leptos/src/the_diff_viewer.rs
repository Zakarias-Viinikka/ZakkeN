//! Caller pov: the reusable diff viewer. `server_page` may be `None`
//! for a client-only page. `client_page` is always required.
//! `kept_side` decides which side is the editable right-hand side: the
//! other side goes on the left (discarded). `output` is a
//! `WriteSignal<ViewerOutput>` the viewer will set when the caller asks
//! it to finish. `handle` is the caller's `FinishHandle`;
//! `on_finished` is optional and runs after each finish.
//!
//! The viewer snapshots its inputs on mount. Changing any of them
//! remounts the inner viewer, so all per-block and per-row state
//! (edits, drags, resets) is fresh — edits and drags never survive an
//! input change. That is deliberate.

use leptos::prelude::*;

use crate::finish_handle::FinishHandle;
use crate::the_frozen_viewer_for_one_mount::TheFrozenViewerForOneMount;
use crate::the_public_types::{
    PageBlocksForTheViewer, PickedSide, ViewerOutput,
};

#[component]
pub fn DiffViewer(
    #[prop(into)] server_page: Signal<Option<PageBlocksForTheViewer>>,
    #[prop(into)] client_page: Signal<PageBlocksForTheViewer>,
    #[prop(into)] kept_side: Signal<PickedSide>,
    #[prop(into)] output: WriteSignal<ViewerOutput>,
    handle: FinishHandle,
    #[prop(optional)] on_finished: Option<Callback<()>>,
) -> impl IntoView {
    let snapshot_key = Memo::new(move |_| {
        (server_page.get(), client_page.get(), kept_side.get())
    });

    view! {
        <For
            each=move || vec![snapshot_key.get()]
            key=|pair: &(
                Option<PageBlocksForTheViewer>,
                PageBlocksForTheViewer,
                PickedSide,
            )| pair.clone()
            children=move |pair: (
                Option<PageBlocksForTheViewer>,
                PageBlocksForTheViewer,
                PickedSide,
            )| {
                let (sp, cp, ks) = pair;
                view! {
                    <TheFrozenViewerForOneMount
                        server_page=sp
                        client_page=cp
                        kept_side=ks
                        output=output
                        handle=handle
                        on_finished=on_finished
                    />
                }
            }
        />
    }
}
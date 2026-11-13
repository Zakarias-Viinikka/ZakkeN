//! Caller pov: the reusable diff viewer. `server_page` may be `None`
//! for a client-only page. `client_page` is always required.
//! `kept_side` decides which side is editable. `server_page_boss` is
//! optional: when `Some`, the viewer builds love letters on finish;
//! when `None`, love letters come back empty (diff-only mode). `output`
//! is a `WriteSignal<ViewerOutput>`; `handle` is `FinishHandle`;
//! `on_finished` is optional.
//!
//! The viewer snapshots its page inputs (not the boss) on mount.
//! Changing any page input remounts the inner viewer; edits and drags
//! never survive a remount. The boss is only read inside the finish
//! effect.

use std::sync::Arc;

use leptos::prelude::*;
use my_yrs_lib::BossOfYrs;

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
    #[prop(into)] server_page_boss: Signal<Option<Arc<BossOfYrs>>>,
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
                let boss_at_mount = server_page_boss.get_untracked();
                view! {
                    <TheFrozenViewerForOneMount
                        server_page=sp
                        client_page=cp
                        kept_side=ks
                        server_page_boss=boss_at_mount
                        output=output
                        handle=handle
                        on_finished=on_finished
                    />
                }
            }
        />
    }
}
use leptos::prelude::*;

use crate::overlays::full_screen_blocker::FullScreenBlocker;
use crate::protocol::AdoptPage;

const RM_CSS: &str = include_str!("../../css/reconcile_modal.css");

#[component]
pub fn ReconcileModal(
    open: Signal<bool>,
    error: Signal<Option<String>>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    view! {
        <style>{RM_CSS}</style>
        <Show when=move || open.get() fallback=|| ()>
            <ReconcileModalBody error=error on_confirm=on_confirm on_cancel=on_cancel />
        </Show>
    }
}

#[component]
fn ReconcileModalBody(
    error: Signal<Option<String>>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let local_pages = crate::local_pages::read_local_pages_placeholder();
    view! {
        <FullScreenBlocker>
            <div class="rm-card">
                <div class="rm-head">
                    <span class="rm-title">"Reconcile"</span>
                </div>
                <div class="rm-body">
                    <div class="rm-note">
                        "Mock: the real flow passes local pages to the server, "
                        "the server checks each one and sends its own version "
                        "when it doesn't match. Nothing is stored in between."
                    </div>
                    {move || error.get().map(|e| view! { <div class="rm-error">{e}</div> })}
                    {local_pages
                        .into_iter()
                        .enumerate()
                        .map(|(i, page)| view! { <PageRow page=page differs=i % 2 == 0 /> })
                        .collect_view()}
                </div>
                <div class="rm-foot">
                    <button class="btn" on:click=move |_| on_cancel.run(())>"Cancel"</button>
                    <button class="btn" on:click=move |_| on_confirm.run(())>"Confirm"</button>
                </div>
            </div>
        </FullScreenBlocker>
    }
}

#[component]
fn PageRow(page: AdoptPage, differs: bool) -> impl IntoView {
    let label = if differs {
        "server would send its version"
    } else {
        "matches, nothing to send"
    };
    view! {
        <div class="rm-page-row">
            <strong>{page.page_id}</strong>
            <span class="rm-page-state">{label}</span>
        </div>
    }
}
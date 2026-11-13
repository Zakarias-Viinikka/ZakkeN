//! Caller pov: hosts the `DiffViewer` for a picked page. Done and the
//! numbered close-box both trigger a finish; Done also writes the
//! preview strip below and stays open. X (numbered box) saves and
//! closes. Reset returns to the picker. The love letters for this page
//! come out of the viewer on finish.

use std::sync::Arc;

use block_diff_viewer_leptos::{
    DiffViewer, FinishHandle, LoveLetter, PageBlocksForTheViewer,
    PickedSide, ViewerOutput,
};
use leptos::prelude::*;
use my_yrs_lib::BossOfYrs;

use super::ready_view::summary_of_one_love_letter;


#[component]
pub fn InlineDiffViewerHost(
    number: usize,
    page_id: String,
    server_page: PageBlocksForTheViewer,
    client_page: PageBlocksForTheViewer,
    kept_side: PickedSide,
    server_page_boss: Arc<BossOfYrs>,
    show_reset: bool,
    on_reset: Callback<()>,
    on_done: Callback<ViewerOutput>,
    on_close: Callback<ViewerOutput>,
) -> impl IntoView {
    let client_count = client_page.blocks.len();
    let server_count = server_page.blocks.len();
    let server_signal = RwSignal::new(Some(server_page));
    let client_signal = RwSignal::new(client_page);
    let kept_side_signal = RwSignal::new(kept_side);
    let boss_signal = RwSignal::new(Some(server_page_boss));
    let (output, set_output) = signal(ViewerOutput::default());
    let handle = FinishHandle::new();
    let handle_for_close = handle.clone();
    let handle_for_done = handle.clone();
    let done_clicked = RwSignal::new(false);

    let preview: RwSignal<Option<Vec<LoveLetter>>> = RwSignal::new(None);

    let on_finished = Callback::new(move |_: ()| {
        let v = output.get_untracked();
        if done_clicked.get_untracked() {
            preview.set(Some(v.love_letters.clone()));
            on_done.run(v);
        } else {
            on_close.run(v);
        }
    });

    view! {
        <div class="rp-host">
            <div class="rp-host-toolbar">
                <button
                    class="rp-host-number-box"
                    type="button"
                    title="close and save"
                    on:click=move |_| {
                        done_clicked.set(false);
                        handle_for_close.finish_and_write_the_result();
                    }
                >
                    {number}
                </button>
                <span class="rp-host-title">{page_id}</span>
                <span class="rp-host-counts">
                    {format!("{} left · {} right", server_count, client_count)}
                </span>
                <span class="rp-host-hint">
                    "left: discarded · right: kept (editable, draggable)"
                </span>
                <span class="rp-host-legend">
                    <span class="rp-host-legend-item">
                        <span class="rp-host-legend-dot" style="background: rgb(210, 80, 80);"></span>
                        "deleted"
                    </span>
                    <span class="rp-host-legend-item">
                        <span class="rp-host-legend-dot" style="background: rgb(80, 180, 80);"></span>
                        "inserted"
                    </span>
                    <span class="rp-host-legend-item">
                        <span class="rp-host-legend-dot" style="background: rgb(100, 160, 220);"></span>
                        "edited"
                    </span>
                    <span class="rp-host-legend-item">
                        <span class="rp-host-legend-dot" style="background: rgb(170, 120, 220);"></span>
                        "moved"
                    </span>
                </span>
                <Show when=move || show_reset>
                    <button on:click=move |_| on_reset.run(())>"Reset"</button>
                </Show>
                <button
                    class="rp-host-btn-primary"
                    on:click=move |_| {
                        done_clicked.set(true);
                        handle_for_done.finish_and_write_the_result();
                    }
                >
                    "Done"
                </button>
            </div>
            <div class="rp-host-body">
                <DiffViewer
                    server_page=server_signal
                    client_page=client_signal
                    kept_side=kept_side_signal
                    server_page_boss=boss_signal
                    output=set_output
                    handle=handle
                    on_finished=on_finished
                />
            </div>
            <Show when=move || preview.get().is_some()>
                <div class="rp-host-preview">
                    <div class="rp-host-preview-title">"Would generate:"</div>
                    {move || {
                        let v = preview.get().unwrap_or_default();
                        if v.is_empty() {
                            view! {
                                <div class="rp-host-preview-empty">
                                    "(no love letters — kept side matches the server)"
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <ul>
                                    {v.iter()
                                        .map(|l| view! { <li>{summary_of_one_love_letter(l)}</li> })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </Show>
        </div>
    }
}

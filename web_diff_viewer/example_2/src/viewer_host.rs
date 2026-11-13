//! Caller pov: hosts one `DiffViewer`. Done and the numbered close-box
//! both call `handle.finish_and_write_the_result()`, which fires
//! `on_finished`, which reads `output` and calls `on_done` or
//! `on_close` with the resulting `ViewerOutput`. Reset returns to the
//! pick screen without committing.

use block_diff_viewer_leptos::{
    DiffViewer, FinishHandle, PageBlocksForTheViewer, PickedSide, ViewerOutput,
};
use leptos::prelude::*;

const CSS: &str = r#"
.viewer-host {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: #1c2433;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-radius: 10px;
    overflow: hidden;
}
.viewer-toolbar {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 18px;
    border-bottom: 1px solid rgba(228, 233, 242, 0.09);
    flex-shrink: 0;
    flex-wrap: wrap;
}
.viewer-number-box {
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(138, 180, 255, 0.1);
    border: 1px solid #8ab4ff;
    border-radius: 8px;
    color: #8ab4ff;
    font-family: ui-monospace, monospace;
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
    transition: background 120ms ease;
}
.viewer-number-box:hover {
    background: rgba(138, 180, 255, 0.18);
}
.viewer-title {
    font-family: ui-monospace, monospace;
    font-size: 16px;
    color: rgba(228, 233, 242, 0.9);
    flex-shrink: 0;
    font-weight: 500;
}
.viewer-counts {
    font-size: 13px;
    color: rgba(228, 233, 242, 0.5);
    flex-shrink: 0;
}
.viewer-hint {
    font-size: 13px;
    color: rgba(228, 233, 242, 0.5);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.viewer-legend {
    display: none;
    align-items: center;
    gap: 14px;
    font-size: 13px;
    color: rgba(228, 233, 242, 0.5);
    flex-shrink: 0;
}
.viewer-legend-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
}
.viewer-legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 2px;
    display: inline-block;
}
@media (min-width: 900px) {
    .viewer-body .dv-legend { display: none; }
    .viewer-legend { display: inline-flex; }
}
.viewer-toolbar button {
    font-family: inherit;
    font-size: 13px;
    padding: 8px 16px;
    border: 1px solid rgba(228, 233, 242, 0.14);
    border-radius: 6px;
    background: transparent;
    color: rgba(228, 233, 242, 0.8);
    cursor: pointer;
    flex-shrink: 0;
    transition: border-color 120ms ease, color 120ms ease, background 120ms ease;
}
.viewer-toolbar button:hover {
    border-color: #8ab4ff;
    color: #8ab4ff;
}
.viewer-toolbar button.viewer-btn-primary {
    background: #8ab4ff;
    border-color: #8ab4ff;
    color: #0a0e18;
    font-weight: 600;
}
.viewer-toolbar button.viewer-btn-primary:hover {
    background: #a4c5ff;
    border-color: #a4c5ff;
    color: #0a0e18;
}
.viewer-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    background: white;
    display: flex;
    flex-direction: column;
}
.viewer-body > .dv-root {
    flex: 1 1 auto;
    height: auto;
    min-height: 0;
}
.viewer-preview {
    flex-shrink: 0;
    padding: 12px 18px;
    background: #161d2a;
    border-top: 1px solid rgba(228, 233, 242, 0.09);
    font-family: ui-monospace, monospace;
    font-size: 12px;
    color: rgba(228, 233, 242, 0.85);
    max-height: 25vh;
    overflow: auto;
}
.viewer-preview-title {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    font-weight: 600;
    color: rgba(228, 233, 242, 0.55);
    margin-bottom: 8px;
}
.viewer-preview-empty {
    color: rgba(228, 233, 242, 0.4);
    font-style: italic;
}
.viewer-preview ul {
    margin: 0;
    padding-left: 20px;
}
.viewer-preview li {
    line-height: 1.5;
}
"#;

#[component]
pub fn ViewerHost(
    number: usize,
    page_id: String,
    server_page: Option<PageBlocksForTheViewer>,
    client_page: PageBlocksForTheViewer,
    kept_side: PickedSide,
    show_reset: bool,
    preview: Signal<Option<Vec<String>>>,
    on_reset: Callback<()>,
    on_done: Callback<ViewerOutput>,
    on_close: Callback<ViewerOutput>,
) -> impl IntoView {
    let client_count = client_page.blocks.len();
    let server_count = server_page
        .as_ref()
        .map(|p| p.blocks.len())
        .unwrap_or(0);
    let server_signal = RwSignal::new(server_page);
    let client_signal = RwSignal::new(client_page);
    let kept_side_signal = RwSignal::new(kept_side);
    let (output, set_output) = signal(ViewerOutput::default());
    let handle = FinishHandle::new();
    let handle_for_close = handle.clone();
    let handle_for_done = handle.clone();
    let done_clicked = RwSignal::new(false);

    let on_finished = Callback::new(move |_: ()| {
        let v = output.get_untracked();
        if done_clicked.get_untracked() {
            on_done.run(v);
        } else {
            on_close.run(v);
        }
    });

    view! {
        <style>{CSS}</style>
        <div class="viewer-host">
            <div class="viewer-toolbar">
                <button
                    class="viewer-number-box"
                    type="button"
                    title="close and save edits"
                    on:click=move |_| {
                        done_clicked.set(false);
                        handle_for_close.finish_and_write_the_result();
                    }
                >
                    {number}
                </button>
                <span class="viewer-title">{page_id}</span>
                <span class="viewer-counts">
                    {format!("{} left · {} right", server_count, client_count)}
                </span>
                <span class="viewer-hint">
                    "left: server · right: kept (editable, draggable)"
                </span>
                <span class="viewer-legend">
                    <span class="viewer-legend-item">
                        <span class="viewer-legend-dot" style="background: rgb(210, 80, 80);"></span>
                        "deleted"
                    </span>
                    <span class="viewer-legend-item">
                        <span class="viewer-legend-dot" style="background: rgb(80, 180, 80);"></span>
                        "inserted"
                    </span>
                    <span class="viewer-legend-item">
                        <span class="viewer-legend-dot" style="background: rgb(100, 160, 220);"></span>
                        "edited"
                    </span>
                    <span class="viewer-legend-item">
                        <span class="viewer-legend-dot" style="background: rgb(170, 120, 220);"></span>
                        "moved"
                    </span>
                </span>
                <Show when=move || show_reset>
                    <button on:click=move |_| on_reset.run(())>"Reset"</button>
                </Show>
                <button
                    class="viewer-btn-primary"
                    on:click=move |_| {
                        done_clicked.set(true);
                        handle_for_done.finish_and_write_the_result();
                    }
                >
                    "Done"
                </button>
            </div>
            <div class="viewer-body">
                <DiffViewer
                    server_page=server_signal
                    client_page=client_signal
                    kept_side=kept_side_signal
                    server_page_boss=Signal::derive(move || None)
                    output=set_output
                    handle=handle
                    on_finished=on_finished
                />
            </div>
            <Show when=move || preview.get().is_some()>
                <div class="viewer-preview">
                    <div class="viewer-preview-title">"Would generate:"</div>
                    {move || {
                        let v = preview.get().unwrap_or_default();
                        if v.is_empty() {
                            view! {
                                <div class="viewer-preview-empty">
                                    "(no love letters — kept side matches the server)"
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <ul>
                                    {v.into_iter()
                                        .map(|s| view! { <li>{s}</li> })
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
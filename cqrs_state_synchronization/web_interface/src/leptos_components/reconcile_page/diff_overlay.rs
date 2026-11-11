//! Caller pov: a fullscreen overlay hosting the reusable `DiffViewer`.
//! Left side is the client's version of the page, right side is the
//! server's. Back cancels without changing anything. Done captures the
//! edited right side, fires `on_done`, and closes.

use block_diff_viewer_leptos::{DiffViewer, FinishHandle};
use leptos::prelude::*;

const CSS: &str = r#"
.rp-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: rgba(10, 14, 24, 0.85);
    display: flex;
    flex-direction: column;
    padding: 16px;
    box-sizing: border-box;
    font-family: system-ui, -apple-system, sans-serif;
}
.rp-overlay-bar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    background: #1c2433;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-radius: 10px 10px 0 0;
    border-bottom: none;
}
.rp-overlay-title {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    color: rgba(228, 233, 242, 0.85);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.rp-overlay-hint {
    font-size: 11px;
    color: rgba(228, 233, 242, 0.45);
    margin-right: 8px;
}
.rp-overlay-btn {
    font-family: inherit;
    font-size: 12px;
    padding: 6px 14px;
    border: 1px solid rgba(228, 233, 242, 0.15);
    border-radius: 5px;
    background: transparent;
    color: #e4e9f2;
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease;
}
.rp-overlay-btn:hover {
    border-color: rgba(228, 233, 242, 0.35);
}
.rp-overlay-btn-primary {
    background: #8ab4ff;
    border-color: #8ab4ff;
    color: #0a0e18;
    font-weight: 600;
}
.rp-overlay-btn-primary:hover {
    background: #a4c5ff;
    border-color: #a4c5ff;
}
.rp-overlay-body {
    flex: 1;
    overflow: auto;
    background: white;
    border-radius: 0 0 10px 10px;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-top: none;
}
"#;

#[component]
pub fn DiffOverlay(
    page_id: String,
    left_blocks: Vec<String>,
    right_blocks: Vec<String>,
    on_close: Callback<()>,
    on_done: Callback<Vec<String>>,
) -> impl IntoView {
    let left = RwSignal::new(left_blocks);
    let right = RwSignal::new(right_blocks);
    let (output, set_output) = signal(Vec::<String>::new());
    let handle = FinishHandle::new();

    let on_finished = Callback::new(move |_: ()| {
        let result = output.get_untracked();
        on_done.run(result);
    });

    view! {
        <style>{CSS}</style>
        <div class="rp-overlay">
            <div class="rp-overlay-bar">
                <span class="rp-overlay-title">{page_id}</span>
                <span class="rp-overlay-hint">
                    "left: yours · right: server's (editable, draggable)"
                </span>
                <button class="rp-overlay-btn" on:click=move |_| on_close.run(())>
                    "Back"
                </button>
                <button
                    class="rp-overlay-btn rp-overlay-btn-primary"
                    on:click=move |_| handle.finish_and_write_the_result()
                >
                    "Done"
                </button>
            </div>
            <div class="rp-overlay-body">
                <DiffViewer
                    left_page=Signal::derive(move || left.get())
                    right_page=Signal::derive(move || right.get())
                    output=set_output
                    handle=handle
                    on_finished=on_finished
                />
            </div>
        </div>
    }
}
//! Caller pov: shown when a page is open but the user hasn't yet
//! decided whose version is the base. Top: a plain side-by-side preview
//! of both pages so the mismatch is visible. Bottom: two framed
//! buttons ("use mine", "use server's") — one of them picks the kept
//! side. The numbered box in the header is a close button.

use leptos::prelude::*;

use generate_love_letters::Block;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickedSide {
    Mine,
    Server,
}

const CSS: &str = r#"
.pick-screen {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 20px;
    background: #1c2433;
    border: 1px solid rgba(228, 233, 242, 0.09);
    border-radius: 10px;
}
.pick-header {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
    flex-shrink: 0;
    width: 100%;
}
.pick-header-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    justify-self: start;
}
.pick-number-box {
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
.pick-number-box:hover {
    background: rgba(138, 180, 255, 0.18);
}
.pick-title {
    font-family: ui-monospace, monospace;
    font-size: 16px;
    color: rgba(228, 233, 242, 0.85);
    font-weight: 500;
}
.pick-preview {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
}
.pick-pane {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(228, 233, 242, 0.08);
    border-radius: 8px;
    padding: 16px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.pick-pane-title {
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: rgba(228, 233, 242, 0.55);
    font-weight: 600;
    margin-bottom: 10px;
    position: sticky;
    top: 0;
    background: #1c2433;
    padding-bottom: 6px;
    margin-left: -16px;
    margin-right: -16px;
    padding-left: 16px;
    padding-right: 16px;
}
.pick-pane-block {
    white-space: pre-wrap;
    word-break: break-word;
    color: rgba(228, 233, 242, 0.85);
    font-family: ui-monospace, monospace;
    font-size: 14px;
    line-height: 1.55;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 4px;
    min-height: 22px;
}
.pick-actions {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-self: center;
}
.pick-btn {
    cursor: pointer;
    font-family: inherit;
    font-size: 13px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    color: rgba(228, 233, 242, 0.85);
    background: transparent;
    border: 5px double rgba(228, 233, 242, 0.15);
    padding: 12px 28px;
    position: relative;
    overflow: hidden;
    transition: all 550ms cubic-bezier(0.19, 1, 0.22, 1);
    outline: 2px solid transparent;
    outline-offset: 2px;
}
.pick-btn span {
    position: relative;
    z-index: 20;
}
.pick-btn::after {
    background: #fff;
    content: "";
    height: 200px;
    left: -90px;
    top: -50px;
    opacity: 0;
    position: absolute;
    transform: rotate(35deg);
    transition: all 550ms cubic-bezier(0.19, 1, 0.22, 1);
    width: 80px;
    pointer-events: none;
}
.pick-btn:hover {
    border-color: rgba(228, 233, 242, 0.45);
    color: #e4e9f2;
    outline-color: rgba(228, 233, 242, 0.2);
    box-shadow: inset 0 0 25px rgba(255, 255, 255, 0.1), 0 0 20px rgba(138, 180, 255, 0.15);
}
.pick-btn:hover::after {
    left: 120%;
    opacity: 0.4;
}
"#;

#[component]
pub fn PickScreen(
    number: usize,
    page_id: String,
    client_blocks: Vec<Block>,
    server_blocks: Vec<Block>,
    on_pick: Callback<PickedSide>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <style>{CSS}</style>
        <div class="pick-screen">
            <div class="pick-header">
                <div class="pick-header-left">
                    <button
                        class="pick-number-box"
                        type="button"
                        title="close"
                        on:click=move |_| on_close.run(())
                    >
                        {number}
                    </button>
                    <span class="pick-title">{page_id}</span>
                </div>
                <div class="pick-actions">
                    <button class="pick-btn" on:click=move |_| on_pick.run(PickedSide::Mine)>
                        <span>"Use mine"</span>
                    </button>
                    <button class="pick-btn" on:click=move |_| on_pick.run(PickedSide::Server)>
                        <span>"Use server's"</span>
                    </button>
                </div>
                <span></span>
            </div>
            <div class="pick-preview">
                <div class="pick-pane">
                    <div class="pick-pane-title">"Yours"</div>
                    {client_blocks
                        .into_iter()
                        .map(|b| view! { <div class="pick-pane-block">{b.text}</div> })
                        .collect_view()}
                </div>
                <div class="pick-pane">
                    <div class="pick-pane-title">"Server's"</div>
                    {server_blocks
                        .into_iter()
                        .map(|b| view! { <div class="pick-pane-block">{b.text}</div> })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

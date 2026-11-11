//! Caller pov: one card in the reconcile grid, showing one mismatched
//! page. Clicking the card opens the diff overlay. Use-mine, use-server,
//! and open-diff are the three actions. A resolved card turns green and
//! shows which choice was made.

use leptos::prelude::*;

use super::resolve_choice::ResolveChoice;

const CSS: &str = r#"
.rp-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(228, 233, 242, 0.12);
    border-radius: 10px;
    padding: 14px 16px;
    cursor: pointer;
    transition: border-color 140ms ease, background 140ms ease;
    color: #e4e9f2;
    display: flex;
    flex-direction: column;
    min-height: 130px;
}
.rp-card:hover {
    border-color: rgba(138, 180, 255, 0.4);
    background: rgba(255, 255, 255, 0.05);
}
.rp-card-resolved {
    border-color: rgba(76, 175, 80, 0.55);
    background: rgba(76, 175, 80, 0.07);
}
.rp-card-resolved:hover {
    border-color: rgba(76, 175, 80, 0.75);
    background: rgba(76, 175, 80, 0.1);
}
.rp-card-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 10px;
}
.rp-card-title {
    font-family: ui-monospace, monospace;
    font-size: 12px;
    color: rgba(228, 233, 242, 0.85);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    min-width: 0;
}
.rp-card-badge {
    flex-shrink: 0;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: #6ec06e;
    background: rgba(76, 175, 80, 0.15);
    padding: 3px 7px;
    border-radius: 4px;
}
.rp-card-preview {
    font-size: 12px;
    color: rgba(228, 233, 242, 0.55);
    line-height: 1.5;
    margin-bottom: 12px;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    flex: 1;
}
.rp-card-counts {
    display: flex;
    gap: 16px;
    font-size: 11px;
    color: rgba(228, 233, 242, 0.42);
    margin-bottom: 10px;
}
.rp-card-counts span strong {
    color: rgba(228, 233, 242, 0.7);
    font-weight: 500;
    margin-right: 3px;
}
.rp-card-actions {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
}
.rp-card-btn {
    font-family: inherit;
    font-size: 11px;
    padding: 5px 10px;
    border: 1px solid rgba(228, 233, 242, 0.14);
    border-radius: 5px;
    background: transparent;
    color: rgba(228, 233, 242, 0.8);
    cursor: pointer;
    transition: border-color 120ms ease, color 120ms ease;
}
.rp-card-btn:hover {
    border-color: #8ab4ff;
    color: #8ab4ff;
}
"#;

#[component]
pub fn MismatchedCard(
    page_id: String,
    client_block_count: usize,
    server_block_count: usize,
    first_client_block: String,
    choice: Signal<Option<ResolveChoice>>,
    on_pick_mine: Callback<()>,
    on_pick_server: Callback<()>,
    on_open_diff: Callback<()>,
) -> impl IntoView {
    let resolved = Signal::derive(move || choice.get().is_some());
    let choice_label = Signal::derive(move || choice.get().map(|c| c.label().to_string()));

    let card_class = move || {
        if resolved.get() {
            "rp-card rp-card-resolved"
        } else {
            "rp-card"
        }
    };

    view! {
        <style>{CSS}</style>
        <div class=card_class on:click=move |_| on_open_diff.run(())>
            <div class="rp-card-head">
                <span class="rp-card-title">{page_id}</span>
                {move || {
                    choice_label.get().map(|l| view! {
                        <span class="rp-card-badge">{l}</span>
                    })
                }}
            </div>
            <div class="rp-card-preview">
                {if first_client_block.is_empty() {
                    "(no blocks on this client)".to_string()
                } else {
                    first_client_block.chars().take(140).collect::<String>()
                }}
            </div>
            <div class="rp-card-counts">
                <span><strong>{client_block_count}</strong>"client"</span>
                <span><strong>{server_block_count}</strong>"server"</span>
            </div>
            <div class="rp-card-actions" on:click=move |e| e.stop_propagation()>
                <button class="rp-card-btn" on:click=move |_| on_pick_mine.run(())>
                    "use mine"
                </button>
                <button class="rp-card-btn" on:click=move |_| on_pick_server.run(())>
                    "use server's"
                </button>
                <button class="rp-card-btn" on:click=move |_| on_open_diff.run(())>
                    "open diff"
                </button>
            </div>
        </div>
    }
}
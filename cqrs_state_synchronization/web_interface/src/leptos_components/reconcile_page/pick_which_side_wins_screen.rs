//! Caller pov: shown when a page is open but the user hasn't yet picked
//! whose version is the base. Numbered box + page_id on the left, the
//! two framed "use mine" / "use server's" buttons dead-center, and a
//! plain side-by-side preview below so the mismatch is visible.

use leptos::prelude::*;

use block_diff_viewer_leptos::PickedSide;
use block_diff_viewer_leptos::Block;


#[component]
pub fn PickWhichSideWinsScreen(
    number: usize,
    page_id: String,
    client_blocks: Vec<Block>,
    server_blocks: Vec<Block>,
    on_pick: Callback<PickedSide>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="rp-pick-screen">
            <div class="rp-pick-header">
                <div class="rp-pick-header-left">
                    <button
                        class="rp-pick-number-box"
                        type="button"
                        title="close"
                        on:click=move |_| on_close.run(())
                    >
                        {number}
                    </button>
                    <span class="rp-pick-title">{page_id}</span>
                </div>
                <div class="rp-pick-actions">
                    <button
                        class="rp-pick-btn"
                        on:click=move |_| on_pick.run(PickedSide::Mine)
                    >
                        <span>"Use mine"</span>
                    </button>
                    <button
                        class="rp-pick-btn"
                        on:click=move |_| on_pick.run(PickedSide::Server)
                    >
                        <span>"Use server's"</span>
                    </button>
                </div>
                <span></span>
            </div>
            <div class="rp-pick-preview">
                <div class="rp-pick-pane">
                    <div class="rp-pick-pane-title">"Yours"</div>
                    {client_blocks
                        .into_iter()
                        .map(|b| view! { <div class="rp-pick-pane-block">{b.text}</div> })
                        .collect_view()}
                </div>
                <div class="rp-pick-pane">
                    <div class="rp-pick-pane-title">"Server's"</div>
                    {server_blocks
                        .into_iter()
                        .map(|b| view! { <div class="rp-pick-pane-block">{b.text}</div> })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

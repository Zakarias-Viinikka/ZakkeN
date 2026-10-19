use leptos::prelude::*;
use crate::complicated_signal_flow::BlockRow;
use super::drag_reorder_black_box::{BlockRenderInput, DragReorderBlackBox, MoveEvent, PageInput};

use super::my_block_content::MyBlockContent;

/// The wrapper I actually call from MainContent. It is the only thing
/// that knows about both the black-box drag component and the block
/// content component I own. Everything rapier/drag/reorder/vacuum/
/// cascade stays inside PagesUiThatOnlyOwnsDragReorder — I never touch
/// it.
#[component]
pub fn DragReorderContainer(
    #[prop(into)] pages: Signal<Vec<PageInput>>,
    #[prop(into)] blocks: Signal<Vec<BlockRow>>,
    on_move: Callback<MoveEvent>,
    on_block_input: Callback<(String, String)>,
) -> impl IntoView {
    let render_block = Callback::new(move |block: BlockRenderInput| {
        let block_id = block.block_id.clone();
        let on_input = Callback::new(move |v: String| {
            on_block_input.run((block_id.clone(), v));
        });
        view! {
            <MyBlockContent block=block on_input=on_input />
        }
        .into_any()
    });

    view! {
        <DragReorderBlackBox
            pages=pages
            blocks=blocks
            on_move=on_move
            render_block=render_block
        />
    }
}

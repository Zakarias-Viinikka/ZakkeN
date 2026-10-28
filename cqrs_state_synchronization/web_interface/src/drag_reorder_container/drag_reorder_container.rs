use leptos::prelude::*;
use crate::complicated_signal_flow::{BlockInputEvent, BlockRow};
use super::drag_reorder_black_box::{BlockRenderInput, DragReorderBlackBox, MoveEvent, PageInput};

use super::my_block_content::MyBlockContent;

#[component]
pub fn DragReorderContainer(
    #[prop(into)] pages: Signal<Vec<PageInput>>,
    #[prop(into)] blocks: Signal<Vec<BlockRow>>,
    on_move: Callback<MoveEvent>,
    on_block_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
) -> impl IntoView {
    let render_block = Callback::new(move |block: BlockRenderInput| {
        let on_input = Callback::new(move |evt: BlockInputEvent| {
            on_block_input.run(evt);
        });
        let on_del = Callback::new(move |bid: String| {
            on_delete_request.run(bid);
        });
        let on_merge = Callback::new(move |(bid, text): (String, String)| {
            on_merge_with_above.run((bid, text));
        });
        let on_split_cb = Callback::new(move |args: (String, String, String)| {
            on_split.run(args);
        });
        view! {
            <MyBlockContent
                block=block
                on_input=on_input
                on_delete_request=on_del
                on_merge_with_above=on_merge
                on_split=on_split_cb
                on_navigate=on_navigate
            />
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
//! Thin adapter: takes the drag-reorder black box's `BlockRenderInput` and renders the
//! markdown converter. All editing, key and caret logic lives in `markdown_conversion/` and
//! the callbacks in `leptos_components/main_content/callbacks/`.
use crate::bbox_drag_reorder_container::BlockRenderInput;
use crate::complex_state::BlockInputEvent;
use crate::markdown_conversion::MarkdownConversion;
use crate::markdown_conversion::main_for_on_link_click::main_on_link_click;
use leptos::prelude::*;

#[component]
pub fn MyBlockContent(
    block: BlockRenderInput,
    on_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
) -> impl IntoView {
    let ghost = block.ghost;
    let ghost_signal = Signal::derive(move || ghost.get());
    let placeholder = if block.is_title { "Untitled" } else { "" }.to_string();
    let on_link_click = Callback::new(move |(command, rest): (String, String)| {
        main_on_link_click(&command, &rest);
    });

    view! {
        <MarkdownConversion
            block_id=block.block_id.to_string()
            content=block.content.clone()
            on_input=on_input
            on_delete_request=on_delete_request
            on_merge_with_above=on_merge_with_above
            on_split=on_split
            on_navigate=on_navigate
            is_title=block.is_title
            ghost=ghost_signal
            placeholder=placeholder
            on_link_click=on_link_click
            class="ps-content"
        />
    }
}
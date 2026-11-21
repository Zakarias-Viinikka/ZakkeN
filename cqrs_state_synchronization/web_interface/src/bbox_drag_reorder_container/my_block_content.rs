use crate::bbox_drag_reorder_container::BlockRenderInput;
use crate::bbox_markdown_conversion::{MarkdownConversion, MarkdownInputEvent};
use crate::complex_state::{
    BlockInputEvent, FOCUS_BLOCK_AT_POS, FOCUS_BLOCK_AT_START, FOCUS_BLOCK_ID,
};
use leptos::prelude::*;

/// One block's content. The editing itself lives in `markdown_conversion`;
/// this file only adapts it to the app's types and focus globals.
#[component]
pub fn MyBlockContent(
    block: BlockRenderInput,
    on_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
) -> impl IntoView {
    let block_id = block.block_id.clone();
    let content = block.content.clone();
    let ghost = block.ghost;
    let is_title = block.is_title;
    let placeholder = if is_title { "Untitled" } else { "" };

    // Local focus signals in the shape the markdown module wants.
    // The app-wide globals are forwarded into them when they target this block.
    let focus_end = ArcRwSignal::new(None::<String>);
    let focus_start = ArcRwSignal::new(None::<String>);
    let focus_pos = ArcRwSignal::new(None::<(String, usize)>);

    {
        let my_id = block_id.clone();
        let local = focus_end.clone();
        Effect::new(move |_| {
            let Some(target) = FOCUS_BLOCK_ID.get() else {
                return;
            };
            if target != my_id {
                return;
            }
            local.set(Some(target));
            FOCUS_BLOCK_ID.set(None);
        });
    }
    {
        let my_id = block_id.clone();
        let local = focus_start.clone();
        Effect::new(move |_| {
            let Some(target) = FOCUS_BLOCK_AT_START.get() else {
                return;
            };
            if target != my_id {
                return;
            }
            local.set(Some(target));
            FOCUS_BLOCK_AT_START.set(None);
        });
    }

    {
        let my_id = block_id.clone();
        let local = focus_pos.clone();
        Effect::new(move |_| {
            let Some((target, pos)) = FOCUS_BLOCK_AT_POS.get() else {
                return;
            };
            if target != my_id {
                return;
            }
            local.set(Some((target, pos)));
            FOCUS_BLOCK_AT_POS.set(None);
        });
    }

    let on_markdown_input = Callback::new(move |e: MarkdownInputEvent| {
        on_input.run(BlockInputEvent {
            block_id: e.block_id,
            old_text: e.old_text,
            new_text: e.new_text,
            was_enter: e.was_enter,
            cursor_pos: e.cursor_pos,
        });
    });

    view! {
        <MarkdownConversion
            block_id=block_id
            content=content
            is_title=is_title
            ghost=ghost
            placeholder=placeholder
            class="ps-content"
            focus_at_end=focus_end
            focus_at_start=focus_start
            focus_at_pos=focus_pos
            on_input=on_markdown_input
            on_delete_request=on_delete_request
            on_merge_with_above=on_merge_with_above
            on_split=on_split
            on_navigate=on_navigate
        />
    }
}

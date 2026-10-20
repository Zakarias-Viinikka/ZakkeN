use crate::complicated_signal_flow::{BlockInputEvent, FOCUS_BLOCK_ID};
use crate::drag_reorder_container::BlockRenderInput;
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

/// The actual block content. This is the file I own and edit for block
/// behaviour. Right now: a textarea that fires on_input for every
/// keystroke. Later: on:click, on:focus, effects, anything I want.
#[component]
pub fn MyBlockContent(
    block: BlockRenderInput,
    on_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
) -> impl IntoView {
    let initial = block.content.get_untracked();
    let ghost = block.ghost;
    let was_enter = StoredValue::new(false);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let my_block_id = block.block_id.to_string();
    let my_block_id_for_focus = my_block_id.clone();
    let content_signal = block.content.clone();

    // Auto-size on mount. Retried across a few frames because the DOM
    // node may not be laid out on the first effect tick.
    Effect::new(move |_| {
        spawn_local(async move {
            for _ in 0..3 {
                gloo_timers::future::TimeoutFuture::new(16).await;
                if let Some(el) = textarea_ref.get() {
                    let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");
                    let sh = el.scroll_height();
                    let _ = web_sys::HtmlElement::style(&el)
                        .set_property("height", &format!("{}px", sh));
                }
            }
        });
    });

    Effect::new(move |_| {
        let Some(target) = FOCUS_BLOCK_ID.get() else {
            return;
        };
        if target != my_block_id_for_focus {
            return;
        }
        if let Some(el) = textarea_ref.get() {
            let _ = el.focus();
        }
        FOCUS_BLOCK_ID.set(None);
    });

    let is_title = block.is_title;
    let on_keydown = move |e: ev::KeyboardEvent| {
        if e.key() == "Enter" {
            was_enter.set_value(true);
            return;
        }
        if (e.key() == "Delete" || e.key() == "Backspace")
            && !is_title
            && content_signal.get_untracked().is_empty()
        {
            on_delete_request.run(my_block_id.clone());
        }
    };

    let on_ev = move |e: ev::Event| {
        let Some(t) = e.target() else {
            return;
        };
        let ta: web_sys::HtmlTextAreaElement = t.unchecked_into();
        let new_val = ta.value();
        let old_val = block.content.get_untracked();
        let cursor_pos = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
        let was_enter_val = was_enter.get_value();
        was_enter.set_value(false);

        block.content.set(new_val.clone());
        on_input.run(BlockInputEvent {
            block_id: block.block_id.to_string(),
            old_text: old_val,
            new_text: new_val,
            was_enter: was_enter_val,
            cursor_pos,
        });

        let _ = web_sys::HtmlElement::style(&ta).set_property("height", "auto");
        let sh = ta.scroll_height();
        let _ = web_sys::HtmlElement::style(&ta).set_property("height", &format!("{}px", sh));
    };

    let placeholder = if block.is_title { "Untitled" } else { "" };

    view! {
        <textarea
            node_ref=textarea_ref
            class="ps-content"
            prop:value=initial
            placeholder=placeholder
            prop:disabled=move || ghost.get()
            rows="1"
            on:keydown=on_keydown
            on:input=on_ev
            style="border: 1px solid rgba(0, 0, 0, 0.05); outline: none; resize: none;"
        ></textarea>
    }
}
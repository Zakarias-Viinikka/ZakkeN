use crate::complicated_signal_flow::{BlockInputEvent, FOCUS_BLOCK_AT_START, FOCUS_BLOCK_ID};
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
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
) -> impl IntoView {
    let initial = block.content.get_untracked();
    let ghost = block.ghost;
    let was_enter = StoredValue::new(false);
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let my_block_id = block.block_id.to_string();
    let my_block_id_for_focus = my_block_id.clone();
    let my_block_id_for_start = my_block_id.clone();
    let content_signal = block.content.clone();
    let content_signal_for_render = content_signal.clone();
    let content_signal_for_focus = content_signal.clone();
    let content_signal_for_start = content_signal.clone();

    // Auto-size on mount. Retried across a few frames because the DOM
    // node may not be laid out on the first effect tick.
    Effect::new(move |_| {
        spawn_local(async move {
            for _ in 0..3 {
                gloo_timers::future::TimeoutFuture::new(16).await;
                if let Some(el) = textarea_ref.get_untracked() {
                    let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");
                    let sh = el.scroll_height();
                    let _ = web_sys::HtmlElement::style(&el)
                        .set_property("height", &format!("{}px", sh));
                }
            }
        });
    });

    // Keep the textarea's value in sync with the block's content signal.
    // No-op during typing (signal and textarea already match), but
    // exposes external content changes (split, merge) to the DOM.
    Effect::new(move |_| {
        let current = content_signal_for_render.get();
        if let Some(el) = textarea_ref.get_untracked() {
            if el.value() != current {
                el.set_value(&current);
                let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");
                let sh = el.scroll_height();
                let _ = web_sys::HtmlElement::style(&el)
                    .set_property("height", &format!("{}px", sh));
            }
        }
    });

    Effect::new(move |_| {
        let Some(target) = FOCUS_BLOCK_ID.get() else {
            return;
        };
        if target != my_block_id_for_focus {
            return;
        }
        if let Some(el) = textarea_ref.get() {
            let current = content_signal_for_focus.get_untracked();
            if el.value() != current {
                el.set_value(&current);
            }
            let _ = el.focus();
            let pos = current.encode_utf16().count() as u32;
            let _ = el.set_selection_range(pos, pos);
        }
        FOCUS_BLOCK_ID.set(None);
    });

    Effect::new(move |_| {
        let sig = FOCUS_BLOCK_AT_START.get();
        let is_target = sig.as_deref() == Some(my_block_id_for_start.as_str());
        if is_target {
            leptos::logging::log!("FAS effect fires for target {}", my_block_id_for_start);
        }
        let Some(_target) = sig else { return; };
        if !is_target {
            return;
        }
        let content_sig = content_signal_for_start.clone();
        let my_id = my_block_id_for_start.clone();
        spawn_local(async move {
            for i in 0..20 {
                if let Some(el) = textarea_ref.get_untracked() {
                    let current = content_sig.get_untracked();
                    if el.value() != current {
                        el.set_value(&current);
                    }
                    let _ = el.focus();
                    let _ = el.set_selection_range(0, 0);
                    let active_tag = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                        .map(|e| e.tag_name())
                        .unwrap_or_default();
                    leptos::logging::log!(
                        "FAS focused on {} at try {} active={}",
                        my_id, i, active_tag,
                    );
                    FOCUS_BLOCK_AT_START.set(None);
                    return;
                }
                gloo_timers::future::TimeoutFuture::new(16).await;
            }
            leptos::logging::log!("FAS gave up on {}", my_id);
            FOCUS_BLOCK_AT_START.set(None);
        });
    });

    let is_title = block.is_title;
    let on_keydown = move |e: ev::KeyboardEvent| {
        let key = e.key();
        // Up/Down: the browser moves the caret within the text first.
        // Only when the caret is already at the very end (Down) or the
        // very start (Up) does the key jump to the neighbouring block.
        if (key == "ArrowUp" || key == "ArrowDown")
            && !e.shift_key()
            && !e.ctrl_key()
            && !e.alt_key()
            && !e.meta_key()
        {
            if let Some(el) = textarea_ref.get_untracked() {
                let len = el.value().encode_utf16().count() as u32;
                let start = el.selection_start().ok().flatten().unwrap_or(0);
                let end = el.selection_end().ok().flatten().unwrap_or(0);
                let go_down = key == "ArrowDown";
                let at_edge = if go_down {
                    start == len && end == len
                } else {
                    start == 0 && end == 0
                };
                if at_edge {
                    e.prevent_default();
                    on_navigate.run((my_block_id.clone(), go_down));
                    return;
                }
            }
        }
        if key == "Enter" {
            if !is_title {
                if let Some(el) = textarea_ref.get_untracked() {
                    let value = el.value();
                    // Whitespace-only counts as empty, so Enter in an
                    // "empty" block just types a newline, never splits.
                    if !value.trim().is_empty() {
                        // Split at the cursor instead of typing a newline.
                        e.prevent_default();
                        let utf16: Vec<u16> = value.encode_utf16().collect();
                        let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
                        let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;
                        let start = start.min(utf16.len());
                        let end = end.clamp(start, utf16.len());
                        let before = String::from_utf16_lossy(&utf16[..start]);
                        let after = String::from_utf16_lossy(&utf16[end..]);
                        // Drop the caret now so it doesn't sit in the old
                        // block while the new one is being created.
                        let _ = el.blur();
                        on_split.run((my_block_id.clone(), before, after));
                        return;
                    }
                }
            }
            was_enter.set_value(true);
            return;
        }
        if is_title {
            return;
        }
        let is_empty = content_signal.get_untracked().is_empty();
        if (key == "Backspace" || key == "Delete") && is_empty {
            on_delete_request.run(my_block_id.clone());
            return;
        }
        if key == "Backspace" && !is_empty {
            if let Some(el) = textarea_ref.get_untracked() {
                let start = el.selection_start().ok().flatten().unwrap_or(1);
                let end = el.selection_end().ok().flatten().unwrap_or(1);
                if start == 0 && end == 0 {
                    on_merge_with_above.run((my_block_id.clone(), el.value()));
                }
            }
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
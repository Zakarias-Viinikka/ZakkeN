use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use super::types::BlockInputEvent;
use super::types::BlockRenderInput;

#[component]
pub fn MyBlockContent(
    block: BlockRenderInput,
    on_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
    focus_block_id: RwSignal<Option<String>>,
    focus_block_at_start: RwSignal<Option<String>>,
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
    // exposes external content changes (remote edits, split, merge) to
    // the DOM. When the textarea is focused, the caret is preserved
    // across the change.
    Effect::new(move |_| {
        let new_value = content_signal_for_render.get();
        let Some(el) = textarea_ref.get_untracked() else {
            return;
        };
        let old_value = el.value();
        if old_value == new_value {
            return;
        }
        let is_focused = {
            let active = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element());
            match active {
                Some(active) => {
                    let a: &JsValue = active.as_ref();
                    let e: &JsValue = el.as_ref();
                    a == e
                }
                None => false,
            }
        };
        let caret_before = el.selection_start().ok().flatten().unwrap_or(0);
        el.set_value(&new_value);
        if is_focused {
            let new_caret = map_caret_across_edit(&old_value, &new_value, caret_before);
            let _ = el.set_selection_range(new_caret, new_caret);
        }
        let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");
        let sh = el.scroll_height();
        let _ = web_sys::HtmlElement::style(&el)
            .set_property("height", &format!("{}px", sh));
    });

    Effect::new(move |_| {
        let Some(target) = focus_block_id.get() else {
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
        focus_block_id.set(None);
    });

    Effect::new(move |_| {
        let sig = focus_block_at_start.get();
        let is_target = sig.as_deref() == Some(my_block_id_for_start.as_str());
        let Some(_target) = sig else { return; };
        if !is_target {
            return;
        }
        let content_sig = content_signal_for_start.clone();
        let my_id = my_block_id_for_start.clone();
        spawn_local(async move {
            for _ in 0..20 {
                if let Some(el) = textarea_ref.get_untracked() {
                    let current = content_sig.get_untracked();
                    if el.value() != current {
                        el.set_value(&current);
                    }
                    let _ = el.focus();
                    let _ = el.set_selection_range(0, 0);
                    focus_block_at_start.set(None);
                    return;
                }
                gloo_timers::future::TimeoutFuture::new(16).await;
            }
            leptos::logging::log!("FAS gave up on {}", my_id);
            focus_block_at_start.set(None);
        });
    });

    let is_title = block.is_title;
    let on_keydown = move |e: ev::KeyboardEvent| {
        let key = e.key();
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
                    if !value.trim().is_empty() {
                        e.prevent_default();
                        let utf16: Vec<u16> = value.encode_utf16().collect();
                        let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
                        let end = el.selection_end().ok().flatten().unwrap_or(0) as usize;
                        let start = start.min(utf16.len());
                        let end = end.clamp(start, utf16.len());
                        let before = String::from_utf16_lossy(&utf16[..start]);
                        let after = String::from_utf16_lossy(&utf16[end..]);
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

/// Map a caret position from `old` to `new` when `new` is `old` with
/// one contiguous edit applied. All lengths are UTF-16 units, matching
/// what `selection_start` / `set_selection_range` use.
fn map_caret_across_edit(old: &str, new: &str, caret: u32) -> u32 {
    let old_u: Vec<u16> = old.encode_utf16().collect();
    let new_u: Vec<u16> = new.encode_utf16().collect();
    let caret = caret as usize;

    let min_len = old_u.len().min(new_u.len());
    let mut prefix = 0;
    while prefix < min_len && old_u[prefix] == new_u[prefix] {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix < (old_u.len() - prefix)
        && suffix < (new_u.len() - prefix)
        && old_u[old_u.len() - 1 - suffix] == new_u[new_u.len() - 1 - suffix]
    {
        suffix += 1;
    }

    if caret <= prefix {
        return caret as u32;
    }
    if caret >= old_u.len() - suffix {
        let delta = new_u.len() as isize - old_u.len() as isize;
        let shifted = (caret as isize + delta).max(prefix as isize);
        return shifted as u32;
    }
    prefix as u32
}

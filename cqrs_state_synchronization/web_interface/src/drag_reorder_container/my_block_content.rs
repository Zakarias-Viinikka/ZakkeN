use crate::drag_reorder_container::BlockRenderInput;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// The actual block content. This is the file I own and edit for block
/// behaviour. Right now: a textarea that fires on_input for every
/// keystroke. Later: on:click, on:focus, effects, anything I want.
#[component]
pub fn MyBlockContent(block: BlockRenderInput, on_input: Callback<String>) -> impl IntoView {
    let initial = block.content.get_untracked();
    let ghost = block.ghost;

    let on_ev = move |e: ev::Event| {
        let Some(t) = e.target() else {
            return;
        };
        let ta: web_sys::HtmlTextAreaElement = t.unchecked_into();
        let new_val = ta.value();
        block.content.set(new_val.clone());
        on_input.run(new_val);
        let _ = web_sys::HtmlElement::style(&ta).set_property("height", "auto");
        let sh = ta.scroll_height();
        let _ = web_sys::HtmlElement::style(&ta).set_property("height", &format!("{}px", sh));
    };

    let placeholder = if block.is_title {
        "Untitled"
    } else {
        "Write something…"
    };

    view! {
        <textarea
            class="ps-content"
            prop:value=initial
            placeholder=placeholder
            prop:disabled=move || ghost.get()
            rows="1"
            on:input=on_ev
        ></textarea>
    }
}

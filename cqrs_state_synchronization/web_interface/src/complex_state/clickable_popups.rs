use std::sync::LazyLock;

use leptos::prelude::*;

/// The tester sets `window.__popups_unclickable__ = true` via a Playwright
/// init script, so popups start unclickable even after a page reload.
fn tester_wants_unclickable() -> bool {
    web_sys::window()
        .and_then(|w| {
            js_sys::Reflect::get(&w, &wasm_bindgen::JsValue::from_str("__popups_unclickable__"))
                .ok()
        })
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

static POPUPS_ARE_CLICKABLE: LazyLock<ArcRwSignal<bool>> =
    LazyLock::new(|| ArcRwSignal::new(!tester_wants_unclickable()));

pub fn plexget_popup_clickability() -> bool {
    POPUPS_ARE_CLICKABLE.get()
}

pub fn plexset_popup_clickability(b: bool) {
    POPUPS_ARE_CLICKABLE.set(b);
}

pub fn plextoggle_popup_clickability() {
    POPUPS_ARE_CLICKABLE.update(|b| *b = !*b);
}

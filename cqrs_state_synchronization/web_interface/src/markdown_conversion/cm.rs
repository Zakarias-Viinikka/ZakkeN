//! The ONLY place that talks to the JS bridge (markdown_js/src/editor.js, built to
//! markdown_web_output/cm_bridge.js, loaded as a classic script by index.html).
//! Calls into JS: plain `extern "C"` fns.
//! Events out of JS: window CustomEvent "cm".
//! Everything outside this file sees typed Rust values, never JSON.
#![allow(dead_code)]
use std::collections::HashMap;

use leptos::html::Div;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::markdown_conversion::edit::Change;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = cmBridge, js_name = createEditor)]
    fn js_create_editor(id: &str, container: &web_sys::Element, initial_text: &str, titles_json: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setText)]
    fn js_set_text(id: &str, new_text: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = getText)]
    fn js_get_text(id: &str) -> String;

    #[wasm_bindgen(js_namespace = cmBridge, js_name = getSelection)]
    fn js_get_selection(id: &str) -> js_sys::Array;

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setSelection)]
    fn js_set_selection(id: &str, anchor: u32, head: u32);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = focus)]
    fn js_focus(id: &str, edge: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = focusKeepCaret)]
    fn js_focus_keep_caret(id: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setEditable)]
    fn js_set_editable(id: &str, editable: bool);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setPlaceholder)]
    fn js_set_placeholder(id: &str, text: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setTitles)]
    fn js_set_titles(id: &str, titles_json: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = destroy)]
    fn js_destroy(id: &str);
}

/// Title info for wikilink widgets. Key in the map = page id.
#[derive(Clone, Debug, Serialize)]
pub struct PageTitle {
    pub title: String,
    pub disabled: bool,
}

fn titles_to_json(titles: &HashMap<String, PageTitle>) -> String {
    serde_json::to_string(titles).unwrap_or_else(|_| "{}".to_string())
}

/// Push text into the editor (minimal diff, caret kept, no echo event, not in undo history).
pub fn set_text(id: &str, text: &str) {
    js_set_text(id, text);
}

/// Current text of the editor.
pub fn get_text(id: &str) -> String {
    js_get_text(id)
}

/// (anchor, head), UTF-16 offsets.
pub fn get_selection(id: &str) -> (usize, usize) {
    let a = js_get_selection(id);
    let n = |i| a.get(i).as_f64().unwrap_or(0.0) as usize;
    (n(0), n(1))
}

pub fn set_selection(id: &str, anchor: usize, head: usize) {
    js_set_selection(id, anchor as u32, head as u32);
}

/// edge: "start" or "end". Moves the caret there and focuses the editor.
pub fn focus(id: &str, edge: &str) {
    js_focus(id, edge);
}

/// Focuses the editor and leaves the selection where it is.
pub fn focus_keep_caret(id: &str) {
    js_focus_keep_caret(id);
}

pub fn set_editable(id: &str, editable: bool) {
    js_set_editable(id, editable);
}

pub fn set_placeholder(id: &str, text: &str) {
    js_set_placeholder(id, text);
}

pub fn set_titles(id: &str, titles: &HashMap<String, PageTitle>) {
    js_set_titles(id, &titles_to_json(titles));
}

/// A user edit: changes in OLD-document coordinates plus the caret after the edit.
#[derive(Clone, Debug)]
pub struct Edit {
    pub changes: Vec<Change>,
    pub head: usize,
}

/// A key press the editor reports before acting on it. UTF-16 offsets.
#[derive(Clone, Debug)]
pub struct KeyPress {
    pub key: String,
    pub anchor: usize,
    pub head: usize,
    pub doc_len: usize,
    /// number of selection ranges (>1 = multi-cursor)
    pub ranges: usize,
}

/// Events JS reports (wire format, private).
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CmEvent {
    Changes {
        id: String,
        changes: Vec<Change>,
        head: usize,
    },
    Key {
        id: String,
        key: String,
        anchor: usize,
        head: usize,
        #[serde(rename = "docLen")]
        doc_len: usize,
        ranges: usize,
    },
    LinkClick {
        id: String,
        command: String,
        rest: String,
    },
}

impl CmEvent {
    fn id(&self) -> &str {
        match self {
            CmEvent::Changes { id, .. } | CmEvent::Key { id, .. } | CmEvent::LinkClick { id, .. } => id,
        }
    }
}

/// One CodeMirror editor.
/// - `on_edit`: user edits (never Rust's own `set_text`).
/// - `on_key`: return true to consume the key (CM then does nothing), false to let CM handle it.
/// - `on_ready`: called once the editor exists (safe to call set_* functions after that).
#[component]
pub fn CmEditor(
    #[prop(into)] id: String,
    #[prop(into)] initial: String,
    on_edit: Callback<Edit>,
    on_key: Callback<KeyPress, bool>,
    #[prop(optional)] initial_titles: HashMap<String, PageTitle>,
    #[prop(optional)] on_link_click: Option<Callback<(String, String)>>,
    #[prop(optional)] on_ready: Option<Callback<()>>,
) -> impl IntoView {
    let host_ref = NodeRef::<Div>::new();
    let titles_json = titles_to_json(&initial_titles);

    // Create the editor once the host div is mounted.
    {
        let id = id.clone();
        Effect::new(move |_| {
            let Some(div) = host_ref.get() else {
                return;
            };
            let el: &web_sys::Element = div.as_ref();
            js_create_editor(&id, el, &initial, &titles_json);
            if let Some(cb) = on_ready {
                cb.run(());
            }
        });
    }

    // JS -> Rust events.
    let my_id = id.clone();
    let handle = window_event_listener_untyped("cm", move |ev| {
        let Ok(custom) = ev.dyn_into::<web_sys::CustomEvent>() else {
            return;
        };
        let Some(json) = custom.detail().as_string() else {
            return;
        };
        let event = match serde_json::from_str::<CmEvent>(&json) {
            Ok(e) => e,
            Err(err) => {
                leptos::logging::error!("bad cm event: {err} in {json}");
                return;
            }
        };
        if event.id() != my_id {
            return;
        }
        match event {
            CmEvent::Changes { changes, head, .. } => on_edit.run(Edit { changes, head }),
            CmEvent::Key {
                key,
                anchor,
                head,
                doc_len,
                ranges,
                ..
            } => {
                let consumed = on_key.run(KeyPress {
                    key,
                    anchor,
                    head,
                    doc_len,
                    ranges,
                });
                if consumed {
                    // JS emit() sees this and tells CodeMirror the key is handled.
                    custom.prevent_default();
                }
            }
            CmEvent::LinkClick { command, rest, .. } => {
                if let Some(cb) = on_link_click {
                    cb.run((command, rest));
                }
            }
        }
    });

    on_cleanup(move || {
        handle.remove();
        js_destroy(&id);
    });

    view! { <div node_ref=host_ref></div> }
}

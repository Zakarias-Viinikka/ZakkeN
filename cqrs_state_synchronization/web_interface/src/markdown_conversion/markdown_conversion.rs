use std::sync::atomic::{AtomicUsize, Ordering};
use leptos::html::Div;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

static NEXT_EDITOR_ID: AtomicUsize = AtomicUsize::new(1);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = cmBridge, js_name = createEditor)]
    fn js_create_editor(
        id: &str,
        container: &web_sys::Element,
        initial_text: &str,
        titles_json: &str,
        callbacks: &JsValue,
    );

    #[wasm_bindgen(js_namespace = cmBridge, js_name = applyExternalText)]
    fn js_apply_external_text(id: &str, new_text: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = getText)]
    fn js_get_text(id: &str) -> String;

    #[wasm_bindgen(js_namespace = cmBridge, js_name = getCursor)]
    fn js_get_cursor(id: &str) -> u32;

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setTitles)]
    fn js_set_titles(id: &str, titles_json: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setEditable)]
    fn js_set_editable(id: &str, editable: bool);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setPlaceholder)]
    fn js_set_placeholder(id: &str, text: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = focus)]
    fn js_focus(id: &str, edge: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = destroy)]
    fn js_destroy(id: &str);
}

struct EditorHandle {
    id: String,
    _on_changes: Closure<dyn FnMut(String)>,
    _on_link_click: Closure<dyn FnMut(String)>,
    _on_intent: Closure<dyn FnMut(String)>,
    _on_height: Closure<dyn FnMut(f64)>,
}

impl Drop for EditorHandle {
    fn drop(&mut self) {
        js_destroy(&self.id);
    }
}

/// Same fields as the real app's BlockInputEvent.
#[derive(Clone, Debug)]
pub struct MarkdownInputEvent {
    pub block_id: String,
    pub old_text: String,
    pub new_text: String,
    pub was_enter: bool,
    pub cursor_pos: usize,
}

fn install_focus_effect(
    focus: Option<ArcRwSignal<Option<String>>>,
    ready: RwSignal<bool>,
    block_id: String,
    editor_id: StoredValue<String>,
    edge: &'static str,
) {
    let Some(sig) = focus else {
        return;
    };
    Effect::new(move |_| {
        let Some(target) = sig.get() else {
            return;
        };
        if target != block_id || !ready.get() {
            return;
        }
        js_focus(&editor_id.get_value(), edge);
        sig.set(None);
    });
}

/// Markdown editor for one block.
///
/// Boundary:
/// - `content` is the single source of truth. Typing writes into it;
///   outside changes to it are pushed into the editor.
/// - Every callback carries the block id, so the app needs no adapter.
/// - Intents (arrows, Enter, Backspace) arrive as typed callbacks.
///   The JSON used between the editor and this component never leaks out.
/// - `focus_at_end` / `focus_at_start`: set to Some(block_id) to focus that
///   block. The component clears the signal after focusing.
#[component]
pub fn MarkdownConversion(
    #[prop(into)] block_id: String,
    content: ArcRwSignal<String>,
    on_input: Callback<MarkdownInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
    #[prop(optional)] is_title: bool,
    #[prop(optional, into)] ghost: Option<Signal<bool>>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional)] focus_at_end: Option<ArcRwSignal<Option<String>>>,
    #[prop(optional)] focus_at_start: Option<ArcRwSignal<Option<String>>>,
    #[prop(optional)] on_link_click: Option<Callback<String>>,
    #[prop(optional, into)] titles_json: Option<Signal<String>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let editor_id = StoredValue::new(format!(
        "markdown-editor-{}",
        NEXT_EDITOR_ID.fetch_add(1, Ordering::Relaxed)
    ));

    let host_ref = NodeRef::<Div>::new();
    let handle_holder: StoredValue<Option<EditorHandle>, LocalStorage> =
        StoredValue::new_local(None);
    // Flips to true once the editor exists. Effects below wait on it.
    let ready = RwSignal::new(false);
    // Set when Enter is pressed without a split; consumed by the next edit.
    let was_enter = StoredValue::new(false);

    // Create the editor once the host div is mounted.
    {
        let content = content.clone();
        let block_id = block_id.clone();
        Effect::new(move |_| {
            let Some(div) = host_ref.get() else {
                return;
            };
            if handle_holder.with_value(|h| h.is_some()) {
                return;
            }

            let id_str = editor_id.get_value();
            let initial_text = content.get_untracked();
            let init_titles = titles_json
                .map(|t| t.get_untracked())
                .unwrap_or_else(|| "{}".to_string());

            // User typed: update content, then tell the app (old -> new).
            let content_for_changes = content.clone();
            let id_for_changes = id_str.clone();
            let block_for_changes = block_id.clone();
            let on_changes = Closure::wrap(Box::new(move |_changes_json: String| {
                let was_enter_val = was_enter.get_value();
                was_enter.set_value(false);
                let current = js_get_text(&id_for_changes);
                let old = content_for_changes.get_untracked();
                if old == current {
                    return;
                }
                let cursor_pos = js_get_cursor(&id_for_changes) as usize;
                content_for_changes.set(current.clone());
                on_input.run(MarkdownInputEvent {
                    block_id: block_for_changes.clone(),
                    old_text: old,
                    new_text: current,
                    was_enter: was_enter_val,
                    cursor_pos,
                });
            }) as Box<dyn FnMut(String)>);

            let on_links = Closure::wrap(Box::new(move |page_id: String| {
                if let Some(cb) = on_link_click {
                    cb.run(page_id);
                }
            }) as Box<dyn FnMut(String)>);

            // Editor intents (JSON) -> typed app callbacks.
            let content_for_intent = content.clone();
            let block_for_intent = block_id.clone();
            let on_intent_closure = Closure::wrap(Box::new(move |json: String| {
                let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) else {
                    return;
                };
                match v["kind"].as_str() {
                    Some("nav") => {
                        let go_down = v["direction"].as_str() == Some("down");
                        on_navigate.run((block_for_intent.clone(), go_down));
                    }
                    Some("split") if !is_title => {
                        let before = v["before"].as_str().unwrap_or("").to_string();
                        let after = v["after"].as_str().unwrap_or("").to_string();
                        on_split.run((block_for_intent.clone(), before, after));
                    }
                    Some("merge_up") if !is_title => {
                        let text = content_for_intent.get_untracked();
                        if text.is_empty() {
                            on_delete_request.run(block_for_intent.clone());
                        } else {
                            on_merge_with_above.run((block_for_intent.clone(), text));
                        }
                    }
                    Some("enter_passthrough") => was_enter.set_value(true),
                    _ => {}
                }
            }) as Box<dyn FnMut(String)>);

            let on_height_closure =
                Closure::wrap(Box::new(move |_height: f64| {}) as Box<dyn FnMut(f64)>);

            let callbacks = js_sys::Object::new();
            let _ = js_sys::Reflect::set(&callbacks, &JsValue::from_str("onChanges"), on_changes.as_ref().unchecked_ref());
            let _ = js_sys::Reflect::set(&callbacks, &JsValue::from_str("onLinkClick"), on_links.as_ref().unchecked_ref());
            let _ = js_sys::Reflect::set(&callbacks, &JsValue::from_str("onIntent"), on_intent_closure.as_ref().unchecked_ref());
            let _ = js_sys::Reflect::set(&callbacks, &JsValue::from_str("onHeight"), on_height_closure.as_ref().unchecked_ref());

            let el: &web_sys::Element = div.unchecked_ref();
            js_create_editor(&id_str, el, &initial_text, &init_titles, callbacks.as_ref());

            handle_holder.set_value(Some(EditorHandle {
                id: id_str,
                _on_changes: on_changes,
                _on_link_click: on_links,
                _on_intent: on_intent_closure,
                _on_height: on_height_closure,
            }));
            ready.set(true);
        });
    }

    // content changed from outside (server sync) -> push into the editor.
    {
        let content = content.clone();
        Effect::new(move |_| {
            let new_text = content.get();
            if !ready.get() {
                return;
            }
            let id_str = editor_id.get_value();
            if js_get_text(&id_str) != new_text {
                js_apply_external_text(&id_str, &new_text);
            }
        });
    }

    if let Some(t_sig) = titles_json {
        Effect::new(move |_| {
            let json = t_sig.get();
            if !ready.get() {
                return;
            }
            js_set_titles(&editor_id.get_value(), &json);
        });
    }

    if let Some(g) = ghost {
        Effect::new(move |_| {
            let is_ghost = g.get();
            if !ready.get() {
                return;
            }
            js_set_editable(&editor_id.get_value(), !is_ghost);
        });
    }

    if let Some(text) = placeholder {
        Effect::new(move |_| {
            if !ready.get() {
                return;
            }
            js_set_placeholder(&editor_id.get_value(), &text);
        });
    }

    install_focus_effect(focus_at_end, ready, block_id.clone(), editor_id, "end");
    install_focus_effect(focus_at_start, ready, block_id.clone(), editor_id, "start");

    on_cleanup(move || {
        handle_holder.set_value(None);
    });

    let container_class = class.unwrap_or_else(|| "markdown-conversion-box".to_string());

    view! {
        <div node_ref=host_ref class=container_class></div>
    }
}

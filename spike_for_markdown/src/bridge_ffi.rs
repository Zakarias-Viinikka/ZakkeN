use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::closure::Closure;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
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

    #[wasm_bindgen(js_namespace = cmBridge, js_name = setTitles)]
    fn js_set_titles(id: &str, titles_json: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = applyExternalText)]
    fn js_apply_external_text(id: &str, new_text: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = dispatchChanges)]
    fn js_dispatch_changes(id: &str, changes_json: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = focus)]
    fn js_focus(id: &str, edge: &str);

    #[wasm_bindgen(js_namespace = cmBridge, js_name = getText)]
    fn js_get_text(id: &str) -> String;

    #[wasm_bindgen(js_namespace = cmBridge, js_name = destroy)]
    fn js_destroy(id: &str);
}

/// Owns the JS closures passed to CodeMirror so they stay valid for the lifetime of the
/// editor instance. Dropping this calls `cmBridge.destroy` first so JS never invokes a
/// freed Rust closure.
pub struct EditorBridgeHandle {
    block_id: String,
    _on_changes: Closure<dyn FnMut(String)>,
    _on_link_click: Closure<dyn FnMut(String)>,
    _on_intent: Closure<dyn FnMut(String)>,
    _on_height: Closure<dyn FnMut(f64)>,
}

impl EditorBridgeHandle {
    pub fn mount(
        block_id: String,
        container: &web_sys::Element,
        initial_text: &str,
        titles_json: &str,
        on_changes: impl FnMut(String) + 'static,
        on_link_click: impl FnMut(String) + 'static,
        on_intent: impl FnMut(String) + 'static,
        on_height: impl FnMut(f64) + 'static,
    ) -> Self {
        let on_changes_cb = Closure::wrap(Box::new(on_changes) as Box<dyn FnMut(String)>);
        let on_link_click_cb = Closure::wrap(Box::new(on_link_click) as Box<dyn FnMut(String)>);
        let on_intent_cb = Closure::wrap(Box::new(on_intent) as Box<dyn FnMut(String)>);
        let on_height_cb = Closure::wrap(Box::new(on_height) as Box<dyn FnMut(f64)>);

        let callbacks_obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &callbacks_obj,
            &JsValue::from_str("onChanges"),
            on_changes_cb.as_ref().unchecked_ref(),
        );
        let _ = js_sys::Reflect::set(
            &callbacks_obj,
            &JsValue::from_str("onLinkClick"),
            on_link_click_cb.as_ref().unchecked_ref(),
        );
        let _ = js_sys::Reflect::set(
            &callbacks_obj,
            &JsValue::from_str("onIntent"),
            on_intent_cb.as_ref().unchecked_ref(),
        );
        let _ = js_sys::Reflect::set(
            &callbacks_obj,
            &JsValue::from_str("onHeight"),
            on_height_cb.as_ref().unchecked_ref(),
        );

        #[cfg(target_arch = "wasm32")]
        js_create_editor(
            &block_id,
            container,
            initial_text,
            titles_json,
            callbacks_obj.as_ref(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        let _ = (container, initial_text, titles_json, callbacks_obj);

        Self {
            block_id,
            _on_changes: on_changes_cb,
            _on_link_click: on_link_click_cb,
            _on_intent: on_intent_cb,
            _on_height: on_height_cb,
        }
    }
}

impl Drop for EditorBridgeHandle {
    fn drop(&mut self) {
        destroy(&self.block_id);
    }
}

pub fn set_titles(id: &str, titles_json: &str) {
    #[cfg(target_arch = "wasm32")]
    js_set_titles(id, titles_json);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (id, titles_json);
}

pub fn apply_external_text(id: &str, new_text: &str) {
    #[cfg(target_arch = "wasm32")]
    js_apply_external_text(id, new_text);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (id, new_text);
}

pub fn dispatch_changes(id: &str, changes_json: &str) {
    #[cfg(target_arch = "wasm32")]
    js_dispatch_changes(id, changes_json);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (id, changes_json);
}

pub fn focus(id: &str, edge: &str) {
    #[cfg(target_arch = "wasm32")]
    js_focus(id, edge);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (id, edge);
}

pub fn get_text(id: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_get_text(id)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = id;
        String::new()
    }
}

pub fn destroy(id: &str) {
    #[cfg(target_arch = "wasm32")]
    js_destroy(id);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = id;
}

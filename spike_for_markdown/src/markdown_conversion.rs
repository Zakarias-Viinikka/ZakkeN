//! `<MarkdownConversion />`: blackbox markdown editor for ONE block. Props mirror the real
//! app's `MyBlockContent` so it can replace the `<textarea>`.
//!
//! Boundary:
//! - `content` is the single source of truth. User typing writes into it; outside changes to it
//!   are pushed into the editor (minimal diff, caret kept, no echo).
//! - Every callback carries the block id, so the app needs no adapter.
//! - Key intents (arrows, Enter, Backspace, Delete) are decided in Rust (`keys.rs`) and arrive
//!   as typed callbacks. JSON never leaves `cm.rs`.
//! - `focus_at_end` / `focus_at_start`: set to Some(block_id) to focus that block. The
//!   component clears the signal after focusing.
//! - Cursor positions and offsets are UTF-16 code units.
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use leptos::prelude::*;

use crate::cm::{self, CmEditor, Edit, KeyPress, PageTitle};
use crate::edit::apply_changes;
use crate::keys::{self, Intent};

static NEXT_EDITOR_ID: AtomicUsize = AtomicUsize::new(1);

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
    editor_id: String,
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
        cm::focus(&editor_id, edge);
        sig.set(None);
    });
}

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
    #[prop(optional, into)] titles: Option<Signal<HashMap<String, PageTitle>>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let editor_id = format!(
        "markdown-editor-{}",
        NEXT_EDITOR_ID.fetch_add(1, Ordering::Relaxed)
    );
    // Flips to true once the editor exists. Effects below wait on it.
    let ready = RwSignal::new(false);
    // Set when Enter was pressed on an empty block (not consumed); read by the next edit.
    let was_enter = StoredValue::new(false);

    // User typed: apply the changes to `content` (pure Rust), tell the app (old -> new).
    let on_edit = {
        let content = content.clone();
        let block_id = block_id.clone();
        let editor_id = editor_id.clone();
        Callback::new(move |edit: Edit| {
            let was_enter_val = was_enter.get_value();
            was_enter.set_value(false);
            let old = content.get_untracked();
            let mut new = apply_changes(&old, &edit.changes);
            // Safety net: the editor is the truth for what the user typed.
            let editor_text = cm::get_text(&editor_id);
            if editor_text != new {
                leptos::logging::error!(
                    "markdown_conversion: rust text != editor text, using editor text. rust={new:?} editor={editor_text:?}"
                );
                new = editor_text;
            }
            if new == old {
                return;
            }
            content.set(new.clone());
            on_input.run(MarkdownInputEvent {
                block_id: block_id.clone(),
                old_text: old,
                new_text: new,
                was_enter: was_enter_val,
                cursor_pos: edit.head,
            });
        })
    };

    // Key pressed: Rust decides what it means and whether the editor must ignore it.
    let on_key = {
        let content = content.clone();
        let block_id = block_id.clone();
        let editor_id = editor_id.clone();
        Callback::new(move |k: KeyPress| -> bool {
            let Some(d) = keys::decide(&k.key, k.anchor, k.head, k.doc_len, k.ranges, || {
                cm::get_text(&editor_id)
            }) else {
                return false;
            };
            match d.intent {
                Intent::NavUp => on_navigate.run((block_id.clone(), false)),
                Intent::NavDown => on_navigate.run((block_id.clone(), true)),
                Intent::Split { before, after } if !is_title => {
                    on_split.run((block_id.clone(), before, after))
                }
                Intent::MergeUp if !is_title => {
                    let text = content.get_untracked();
                    if text.is_empty() {
                        on_delete_request.run(block_id.clone());
                    } else {
                        on_merge_with_above.run((block_id.clone(), text));
                    }
                }
                Intent::EnterOnEmpty => was_enter.set_value(true),
                _ => {}
            }
            d.consume
        })
    };

    let link_cb = Callback::new(move |page_id: String| {
        if let Some(cb) = on_link_click {
            cb.run(page_id);
        }
    });
    let ready_cb = Callback::new(move |_: ()| ready.set(true));

    // content changed from outside (server sync) -> push into the editor.
    {
        let content = content.clone();
        let id = editor_id.clone();
        Effect::new(move |_| {
            let new_text = content.get();
            if !ready.get() {
                return;
            }
            if cm::get_text(&id) != new_text {
                cm::set_text(&id, &new_text);
            }
        });
    }

    if let Some(t_sig) = titles {
        let id = editor_id.clone();
        Effect::new(move |_| {
            let map = t_sig.get();
            if !ready.get() {
                return;
            }
            cm::set_titles(&id, &map);
        });
    }

    if let Some(g) = ghost {
        let id = editor_id.clone();
        Effect::new(move |_| {
            let is_ghost = g.get();
            if !ready.get() {
                return;
            }
            cm::set_editable(&id, !is_ghost);
        });
    }

    if let Some(text) = placeholder {
        let id = editor_id.clone();
        Effect::new(move |_| {
            if !ready.get() {
                return;
            }
            cm::set_placeholder(&id, &text);
        });
    }

    install_focus_effect(focus_at_end, ready, block_id.clone(), editor_id.clone(), "end");
    install_focus_effect(focus_at_start, ready, block_id.clone(), editor_id.clone(), "start");

    let initial = content.get_untracked();
    let init_titles = titles.map(|t| t.get_untracked()).unwrap_or_default();
    let container_class = class.unwrap_or_else(|| "markdown-conversion-box".to_string());

    view! {
        <div class=container_class>
            <CmEditor
                id=editor_id
                initial=initial
                on_edit=on_edit
                on_key=on_key
                initial_titles=init_titles
                on_link_click=link_cb
                on_ready=ready_cb
            />
        </div>
    }
}

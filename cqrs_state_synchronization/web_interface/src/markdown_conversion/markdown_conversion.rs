//! `<MarkdownConversion />`: blackbox markdown editor for ONE block. Props mirror the app's
//! `MyBlockContent` so it can replace the `<textarea>`.
//!
//! Boundary:
//! - `content` is the single source of truth. User typing writes into it; outside changes to it
//!   are pushed into the editor (minimal diff, caret kept, no echo).
//! - Every callback carries the block id, so the app needs no adapter.
//! - Key intents (arrows, Enter, Backspace, Delete) are decided in Rust (`keys.rs`) and arrive
//!   as typed callbacks. JSON never leaves `cm.rs`.
//! - Carets: there is no focus prop. Once the editor is ready, this component registers a
//!   handler under its block_id in `complex_state/caret/caret_handlers_registry.rs`.
//!   `CaretOwner` calls it. The handler syncs the editor text from `content` first, then places
//!   the caret and focuses. Cleanup unregisters only if the entry is still this mount's own.
//! - Cursor positions and offsets are UTF-16 code units.
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use leptos::prelude::*;

use crate::complex_state::BlockInputEvent;
use crate::complex_state::caret::caret_handlers_registry::{
    CaretHandler, CaretHandlerToken, plexset_register_caret_handler,
    plexset_unregister_caret_handler_if_still_mine,
};
use crate::complex_state::caret::caret_request::CaretPlace;
use crate::markdown_conversion::cm::{self, CmEditor, Edit, KeyPress, PageTitle};
use crate::markdown_conversion::edit::apply_changes;
use crate::markdown_conversion::keys::{self, Intent};

static NEXT_EDITOR_ID: AtomicUsize = AtomicUsize::new(1);

#[component]
pub fn MarkdownConversion(
    #[prop(into)] block_id: String,
    content: ArcRwSignal<String>,
    on_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
    #[prop(optional)] is_title: bool,
    #[prop(optional, into)] ghost: Option<Signal<bool>>,
    #[prop(optional, into)] placeholder: Option<String>,
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
            on_input.run(BlockInputEvent {
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
            let Some(d) = keys::decide(&k.key, k.anchor, k.head, k.doc_len, k.ranges, is_title, || {
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
                Intent::EnterWithoutSplit => was_enter.set_value(true),
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

    install_caret_handler_registration(
        block_id.clone(),
        editor_id.clone(),
        content.clone(),
        ready,
    );

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

/// Registers this block's caret handler once the editor is ready; unregisters on cleanup, but
/// only if the registry entry is still this mount's own (same block id may be re-mounted).
fn install_caret_handler_registration(
    block_id: String,
    editor_id: String,
    content: ArcRwSignal<String>,
    ready: RwSignal<bool>,
) {
    let my_token: Arc<Mutex<Option<CaretHandlerToken>>> = Arc::new(Mutex::new(None));

    {
        let block_id = block_id.clone();
        let my_token = my_token.clone();
        Effect::new(move |_| {
            if !ready.get() {
                return;
            }
            let handler = make_caret_handler(editor_id.clone(), content.clone());
            let token = plexset_register_caret_handler(block_id.clone(), handler);
            *my_token.lock().unwrap() = Some(token);
        });
    }

    on_cleanup(move || {
        if let Some(token) = my_token.lock().unwrap().take() {
            plexset_unregister_caret_handler_if_still_mine(&block_id, token);
        }
    });
}

/// (1) sync editor text from `content` if they differ, (2) place caret + focus + scroll.
/// `At(n)` beyond the editor's length returns false (CaretOwner retries next frame) unless
/// `is_last_try`, then it clamps.
fn make_caret_handler(editor_id: String, content: ArcRwSignal<String>) -> CaretHandler {
    Arc::new(move |place: CaretPlace, is_last_try: bool| -> bool {
        let wanted = content.get_untracked();
        let mut editor_text = cm::get_text(&editor_id);
        if editor_text != wanted {
            cm::set_text(&editor_id, &wanted);
            editor_text = wanted;
        }
        let len = editor_text.encode_utf16().count();
        let pos = match place {
            CaretPlace::Start => 0,
            CaretPlace::End => len,
            CaretPlace::At(n) if n > len && !is_last_try => return false,
            CaretPlace::At(n) => n.min(len),
        };
        cm::set_selection(&editor_id, pos, pos);
        cm::focus_keep_caret(&editor_id);
        true
    })
}

use std::collections::BTreeMap;
use std::rc::Rc;

use leptos::html::Div;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use yrs::{GetString, Text};

use crate::authority_doc::AuthorityBlockDoc;
use crate::bridge_ffi::{self, EditorBridgeHandle};
use crate::edit_translation::{CmChange, TextEdit, convert_changes_to_edits_desc, utf16_len};
use crate::fuzz::{XorShift64, generate_random_changes};
use crate::link_parser::parse_links;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TitleEntry {
    title: String,
    disabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind")]
#[allow(dead_code)]
enum EditorIntent {
    #[serde(rename = "nav")]
    Nav { direction: String },
    #[serde(rename = "split")]
    Split { before: String, after: String },
    #[serde(rename = "merge_up")]
    MergeUp,
    #[serde(rename = "enter_passthrough")]
    EnterPassthrough,
}

#[derive(Clone)]
struct BlockDebugState {
    id: &'static str,
    authority: Rc<AuthorityBlockDoc>,
    cm_text: RwSignal<String>,
    authority_text: RwSignal<String>,
    in_sync: RwSignal<bool>,
    mismatch_note: RwSignal<Option<String>>,
    last_edits: RwSignal<Vec<String>>,
    height_px: RwSignal<i32>,
    link_clicks: RwSignal<Vec<String>>,
    intent_log: RwSignal<Vec<String>>,
}

#[component]
pub fn App() -> impl IntoView {
    let initial_titles = BTreeMap::from([
        (
            "page-cats".to_string(),
            TitleEntry {
                title: "Cats".to_string(),
                disabled: false,
            },
        ),
        (
            "page-dogs".to_string(),
            TitleEntry {
                title: "Dogs".to_string(),
                disabled: false,
            },
        ),
    ]);
    let titles_map = RwSignal::new(initial_titles);
    let fuzz_status = RwSignal::new("Not run".to_string());
    let copy_status = RwSignal::new(String::new());

    let blocks = vec![
        make_block_state(
            "block-1",
            "# Heading One\nSee **bold**, *italic*, and `inline code` here.",
        ),
        make_block_state(
            "block-2",
            "Links: [[Cats|page-cats]] and [[Missing Page|page-missing]] — try editing around them.",
        ),
        make_block_state(
            "block-3",
            "Emoji 😀 CJK 日本語 Family 👨‍👩‍👧‍👦 Accents café résumé — plus a long paragraph so CodeMirror wraps lines naturally inside the block container and ResizeObserver reports height updates.",
        ),
    ];
    let block_ids: Vec<&'static str> = blocks.iter().map(|b| b.id).collect();

    let b2_for_start = blocks[1].clone();
    let on_remote_start = move |_| {
        run_remote_sim(&b2_for_start, |txn, text_ref| {
            text_ref.insert(txn, 0, "[remote] ");
        });
    };

    let b2_for_mid = blocks[1].clone();
    let on_remote_mid = move |_| {
        run_remote_sim(&b2_for_mid, |txn, text_ref| {
            let cur = text_ref.get_string(txn);
            let mid = find_safe_utf16_midpoint(&cur);
            text_ref.insert(txn, mid, " <MID> ");
        });
    };

    let b2_for_end = blocks[1].clone();
    let on_remote_end = move |_| {
        run_remote_sim(&b2_for_end, |txn, text_ref| {
            let len = utf16_len(&text_ref.get_string(txn));
            text_ref.insert(txn, len, " [end]");
        });
    };

    let b2_for_inside_link = blocks[1].clone();
    let on_remote_inside_link = move |_| {
        run_remote_sim(&b2_for_inside_link, |txn, text_ref| {
            let cur = text_ref.get_string(txn);
            if let Some(link) = parse_links(&cur).first() {
                // Insert a newline inside the token's title so it breaks into plain text.
                let pos = utf16_len(&cur[..link.start_byte]) + 3;
                text_ref.insert(txn, pos, "\n");
            }
        });
    };

    let b2_for_del_link = blocks[1].clone();
    let on_remote_del_link = move |_| {
        run_remote_sim(&b2_for_del_link, |txn, text_ref| {
            let cur = text_ref.get_string(txn);
            if let Some(link) = parse_links(&cur).first() {
                let start_u16 = utf16_len(&cur[..link.start_byte]);
                let len_u16 = utf16_len(&cur[link.start_byte..link.end_byte]);
                text_ref.remove_range(txn, start_u16, len_u16);
            }
        });
    };

    let ids_for_rename = block_ids.clone();
    let on_rename_page = move |_| {
        titles_map.update(|map| {
            if let Some(entry) = map.get_mut("page-cats") {
                entry.title = if entry.title == "Cats" {
                    "Cats & Kittens".to_string()
                } else {
                    "Cats".to_string()
                };
            }
        });
        let json = serde_json::to_string(&titles_map.get_untracked()).unwrap_or_default();
        for id in &ids_for_rename {
            bridge_ffi::set_titles(id, &json);
        }
    };

    let ids_for_disable = block_ids.clone();
    let on_toggle_disable = move |_| {
        titles_map.update(|map| {
            if let Some(entry) = map.get_mut("page-cats") {
                entry.disabled = !entry.disabled;
            }
        });
        let json = serde_json::to_string(&titles_map.get_untracked()).unwrap_or_default();
        for id in &ids_for_disable {
            bridge_ffi::set_titles(id, &json);
        }
    };

    let blocks_for_resync = blocks.clone();
    let on_resync = move |_| {
        for b in &blocks_for_resync {
            if let Ok(auth) = b.authority.read_text() {
                bridge_ffi::apply_external_text(b.id, &auth);
                refresh_sync_signals(b);
            }
        }
    };

    let blocks_for_fuzz = blocks.clone();
    let on_fuzz = move |_| {
        let seed: u64 = 12345;
        let result = run_browser_fuzz(&blocks_for_fuzz[2], seed, 500);
        fuzz_status.set(result);
    };

    let blocks_for_copy = blocks.clone();
    let on_copy_report = move |_| {
        let report = build_plain_report(&blocks_for_copy, &fuzz_status.get_untracked());
        if let Some(win) = web_sys::window() {
            let clipboard = win.navigator().clipboard();
            let _ = clipboard.write_text(&report);
            copy_status.set("Copied report to clipboard.".to_string());
        } else {
            copy_status.set("No window available.".to_string());
        }
    };

    view! {
        <h1>"CodeMirror 6 + yrs Markdown Spike"</h1>
        <div class="toolbar">
            <button on:click=on_remote_start>"Remote: start (b2)"</button>
            <button on:click=on_remote_mid>"Remote: middle (b2)"</button>
            <button on:click=on_remote_end>"Remote: end (b2)"</button>
            <button on:click=on_remote_inside_link>"Remote: break link (b2)"</button>
            <button on:click=on_remote_del_link>"Remote: delete link (b2)"</button>
            <button on:click=on_rename_page>"Rename page-cats"</button>
            <button on:click=on_toggle_disable>"Toggle disable page-cats"</button>
            <button on:click=on_resync>"Resync"</button>
            <button on:click=on_fuzz>"Fuzz 500x (b3)"</button>
            <button on:click=on_copy_report>"Copy report"</button>
            <span><strong>"Fuzz: "</strong>{move || fuzz_status.get()}</span>
            <span>{move || copy_status.get()}</span>
        </div>
        <div>
            {blocks
                .iter()
                .enumerate()
                .map(|(idx, block)| {
                    let prev_id = if idx > 0 { Some(block_ids[idx - 1]) } else { None };
                    let next_id = if idx + 1 < block_ids.len() {
                        Some(block_ids[idx + 1])
                    } else {
                        None
                    };
                    view! {
                        <BlockCard
                            block=block.clone()
                            titles_map=titles_map
                            prev_id=prev_id
                            next_id=next_id
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn BlockCard(
    block: BlockDebugState,
    titles_map: RwSignal<BTreeMap<String, TitleEntry>>,
    prev_id: Option<&'static str>,
    next_id: Option<&'static str>,
) -> impl IntoView {
    let host_ref = NodeRef::<Div>::new();
    let bridge_holder: StoredValue<Option<EditorBridgeHandle>, LocalStorage> =
        StoredValue::new_local(None);

    let block_for_mount = block.clone();
    Effect::new(move |_| {
        let Some(div) = host_ref.get() else {
            return;
        };
        if bridge_holder.with_value(|v| v.is_some()) {
            return;
        }
        let el: &web_sys::Element = div.unchecked_ref();
        let initial_text = block_for_mount.authority.read_text().unwrap_or_default();
        let titles_json = serde_json::to_string(&titles_map.get_untracked()).unwrap_or_default();

        let b_changes = block_for_mount.clone();
        let b_links = block_for_mount.clone();
        let b_intent = block_for_mount.clone();
        let b_height = block_for_mount.clone();

        let handle = EditorBridgeHandle::mount(
            block_for_mount.id.to_string(),
            el,
            &initial_text,
            &titles_json,
            move |changes_json| handle_cm_changes(&b_changes, &changes_json),
            move |page_id| {
                b_links.link_clicks.update(|log| {
                    log.push(format!("clicked {page_id}"));
                    if log.len() > 10 {
                        log.remove(0);
                    }
                });
            },
            move |intent_json| handle_cm_intent(&b_intent, &intent_json, prev_id, next_id),
            move |height| {
                b_height.height_px.set(height.round() as i32);
            },
        );
        bridge_holder.set_value(Some(handle));
    });

    on_cleanup(move || {
        bridge_holder.set_value(None);
    });

    view! {
        <div class="block-card">
            <div class="block-header">
                <div>
                    <strong>{block.id}</strong>
                    {" | height: "}
                    {move || format!("{}px", block.height_px.get())}
                </div>
                <div>
                    {move || {
                        if block.in_sync.get() {
                            view! { <span class="sync-ok">"IN SYNC"</span> }.into_any()
                        } else {
                            let msg = block
                                .mismatch_note
                                .get()
                                .unwrap_or_else(|| "MISMATCH".to_string());
                            view! { <span class="sync-bad">{msg}</span> }.into_any()
                        }
                    }}
                </div>
            </div>
            <div node_ref=host_ref></div>
            <div class="debug-grid">
                <div class="debug-box">
                    <strong>"CM text"</strong>
                    <pre>{move || block.cm_text.get()}</pre>
                </div>
                <div class="debug-box">
                    <strong>"Authority text (yrs)"</strong>
                    <pre>{move || block.authority_text.get()}</pre>
                </div>
                <div class="debug-box">
                    <strong>"Last 10 edit batches (TextEdit)"</strong>
                    <pre>{move || block.last_edits.get().join("\n")}</pre>
                </div>
                <div class="debug-box">
                    <strong>"Link clicks & intents"</strong>
                    <pre>{move || {
                        let clicks = block.link_clicks.get().join("\n");
                        let intents = block.intent_log.get().join("\n");
                        format!("Clicks:\n{clicks}\n\nIntents:\n{intents}")
                    }}</pre>
                </div>
            </div>
        </div>
    }
}

fn make_block_state(id: &'static str, initial: &str) -> BlockDebugState {
    let authority = Rc::new(AuthorityBlockDoc::new(id, initial));
    BlockDebugState {
        id,
        authority,
        cm_text: RwSignal::new(initial.to_string()),
        authority_text: RwSignal::new(initial.to_string()),
        in_sync: RwSignal::new(true),
        mismatch_note: RwSignal::new(None),
        last_edits: RwSignal::new(Vec::new()),
        height_px: RwSignal::new(0),
        link_clicks: RwSignal::new(Vec::new()),
        intent_log: RwSignal::new(Vec::new()),
    }
}

fn handle_cm_changes(block: &BlockDebugState, changes_json: &str) {
    let changes: Vec<CmChange> = match serde_json::from_str(changes_json) {
        Ok(c) => c,
        Err(e) => {
            leptos::logging::error!("Failed to parse CM changes JSON: {e}");
            block.in_sync.set(false);
            block.mismatch_note.set(Some(format!("JSON parse error: {e}")));
            return;
        }
    };

    let old_auth = match block.authority.read_text() {
        Ok(s) => s,
        Err(e) => {
            leptos::logging::error!("Failed to read authority: {e}");
            block.in_sync.set(false);
            block.mismatch_note.set(Some(e));
            return;
        }
    };

    let edits: Vec<TextEdit> = match convert_changes_to_edits_desc(&old_auth, &changes) {
        Ok(ed) => ed,
        Err(e) => {
            leptos::logging::error!("Edit translation error on {}: {e}", block.id);
            block.in_sync.set(false);
            block.mismatch_note.set(Some(format!("Translate error: {e}")));
            return;
        }
    };

    if let Err(e) = block.authority.apply_edits(&edits) {
        leptos::logging::error!("yrs apply_edits error on {}: {e}", block.id);
        block.in_sync.set(false);
        block.mismatch_note.set(Some(format!("yrs error: {e}")));
        return;
    }

    block.last_edits.update(|list| {
        list.push(format!("{edits:?}"));
        if list.len() > 10 {
            list.remove(0);
        }
    });

    refresh_sync_signals(block);
}

fn handle_cm_intent(
    block: &BlockDebugState,
    intent_json: &str,
    prev_id: Option<&'static str>,
    next_id: Option<&'static str>,
) {
    block.intent_log.update(|log| {
        log.push(intent_json.to_string());
        if log.len() > 10 {
            log.remove(0);
        }
    });

    if let Ok(intent) = serde_json::from_str::<EditorIntent>(intent_json) {
        match intent {
            EditorIntent::Nav { direction } => {
                if direction == "up" {
                    if let Some(target) = prev_id {
                        bridge_ffi::focus(target, "end");
                    }
                } else if direction == "down" {
                    if let Some(target) = next_id {
                        bridge_ffi::focus(target, "start");
                    }
                }
            }
            EditorIntent::MergeUp => {
                if let Some(target) = prev_id {
                    bridge_ffi::focus(target, "end");
                }
            }
            EditorIntent::Split { .. } => {}
        }
    }
}

fn refresh_sync_signals(block: &BlockDebugState) {
    let auth = block.authority.read_text().unwrap_or_default();
    let cm = bridge_ffi::get_text(block.id);
    let ok = auth == cm;
    if !ok {
        leptos::logging::error!(
            "SYNC MISMATCH on {}!\nauthority: {:?}\ncm:        {:?}",
            block.id,
            auth,
            cm
        );
        block.mismatch_note.set(Some("MISMATCH (use Resync)".to_string()));
    } else {
        block.mismatch_note.set(None);
    }
    block.authority_text.set(auth);
    block.cm_text.set(cm);
    block.in_sync.set(ok);
}

fn run_remote_sim<F>(block: &BlockDebugState, edit_fn: F)
where
    F: FnOnce(&mut yrs::TransactionMut<'_>, &yrs::XmlTextRef),
{
    match block.authority.simulate_remote_edit(edit_fn) {
        Ok(new_text) => {
            bridge_ffi::apply_external_text(block.id, &new_text);
            refresh_sync_signals(block);
        }
        Err(e) => {
            leptos::logging::error!("Remote sim failed on {}: {e}", block.id);
        }
    }
}

fn run_browser_fuzz(block: &BlockDebugState, seed: u64, rounds: usize) -> String {
    let mut rng = XorShift64::new(seed);
    for round in 0..rounds {
        let current = match block.authority.read_text() {
            Ok(s) => s,
            Err(e) => return format!("ERR round {round} seed={seed}: {e}"),
        };
        let changes = if current.chars().count() > 300 {
            vec![CmChange {
                from: 0,
                to: utf16_len(&current),
                insert: "Fuzz reset 😀 日本語 [[Cats|page-cats]]".to_string(),
            }]
        } else {
            generate_random_changes(&current, &mut rng)
        };
        let json = match serde_json::to_string(&changes) {
            Ok(j) => j,
            Err(e) => return format!("ERR serialize round {round} seed={seed}: {e}"),
        };

        bridge_ffi::dispatch_changes(block.id, &json);

        let auth = block.authority.read_text().unwrap_or_default();
        let cm = bridge_ffi::get_text(block.id);
        if auth != cm {
            refresh_sync_signals(block);
            return format!(
                "MISMATCH at round {round} (seed={seed}): auth_len={} cm_len={}",
                auth.len(),
                cm.len()
            );
        }
    }
    refresh_sync_signals(block);
    format!("PASS {rounds}/{rounds} (seed={seed})")
}

fn find_safe_utf16_midpoint(s: &str) -> u32 {
    let half = s.len() / 2;
    let mut best = 0;
    for (idx, _) in s.char_indices() {
        if idx <= half {
            best = idx;
        } else {
            break;
        }
    }
    utf16_len(&s[..best])
}

fn build_plain_report(blocks: &[BlockDebugState], fuzz: &str) -> String {
    let mut lines = Vec::new();
    lines.push("=== Markdown Spike Report ===".to_string());
    lines.push(format!("Fuzz: {fuzz}"));
    for b in blocks {
        let sync = if b.in_sync.get_untracked() {
            "IN SYNC"
        } else {
            "MISMATCH"
        };
        lines.push(format!(
            "\n[{}] sync={} height={}px",
            b.id,
            sync,
            b.height_px.get_untracked()
        ));
        lines.push(format!("  auth: {:?}", b.authority_text.get_untracked()));
        lines.push(format!("  cm:   {:?}", b.cm_text.get_untracked()));
        let intents = b.intent_log.get_untracked();
        if !intents.is_empty() {
            lines.push(format!("  intents: {}", intents.join(" | ")));
        }
        let clicks = b.link_clicks.get_untracked();
        if !clicks.is_empty() {
            lines.push(format!("  clicks: {}", clicks.join(" | ")));
        }
    }
    lines.join("\n")
}

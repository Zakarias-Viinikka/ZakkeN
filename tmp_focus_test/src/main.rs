use gloo_timers::future::TimeoutFuture;
use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::LazyLock;
use wasm_bindgen::JsCast;

static FOCUS_BLOCK_AT_START: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Clone, Debug, PartialEq)]
struct BlockData {
    id: String,
    content: String,
}

#[derive(Clone)]
struct BlockBox {
    internal_id: usize,
    id: String,
    content: ArcRwSignal<String>,
}

#[component]
fn App() -> impl IntoView {
    let blocks = RwSignal::new(vec![
        BlockData { id: "a".into(), content: String::new() },
        BlockData { id: "b".into(), content: String::new() },
        BlockData { id: "c".into(), content: String::new() },
    ]);
    let next_id = RwSignal::new(1usize);

    let on_split = Callback::new(move |(block_id, after): (String, String)| {
        let blocks = blocks;
        let next_id = next_id;

        spawn_local(async move {
            TimeoutFuture::new(50).await;

            let mut new_id_for_focus = None;
            blocks.update(|list| {
                if let Some(idx) = list.iter().position(|b| b.id == block_id) {
                    let new_id = format!("b-{}", next_id.get_untracked());
                    next_id.update(|n| *n += 1);
                    list.insert(idx + 1, BlockData {
                        id: new_id.clone(),
                        content: after,
                    });
                    new_id_for_focus = Some(new_id);
                }
            });

            if let Some(id) = new_id_for_focus {
                FOCUS_BLOCK_AT_START.set(Some(id));
            }
        });
    });

    let on_content_change = Callback::new(move |(block_id, new_text): (String, String)| {
        blocks.update(|list| {
            if let Some(b) = list.iter_mut().find(|b| b.id == block_id) {
                b.content = new_text;
            }
        });
    });

    view! {
        <div style="padding: 20px; font-family: monospace;">
            <div style="margin-bottom: 12px;">"Press Enter to split the current block"</div>
            <BlockContainer
                blocks=Signal::derive(move || blocks.get())
                on_split=on_split
                on_content_change=on_content_change
            />
        </div>
    }
}

#[component]
fn BlockContainer(
    #[prop(into)] blocks: Signal<Vec<BlockData>>,
    on_split: Callback<(String, String)>,
    on_content_change: Callback<(String, String)>,
) -> impl IntoView {
    let boxes = RwSignal::new(Vec::<BlockBox>::new());
    let next_internal_id = RwSignal::new(1usize);

    Effect::new(move |_| {
        let incoming = blocks.get();
        let current = boxes.get_untracked();

        // Sync content while preserving each existing block's signal.
        for incoming_block in &incoming {
            if let Some(existing) = current.iter().find(|b| b.id == incoming_block.id) {
                if existing.content.get_untracked() != incoming_block.content {
                    existing.content.set(incoming_block.content.clone());
                }
            }
        }

        // Content-only changes do not require rebuilding the list.
        let same_order = current.len() == incoming.len()
            && current.iter().zip(incoming.iter())
                .all(|(current, incoming)| current.id == incoming.id);

        if same_order {
            return;
        }

        // Reuse existing blocks and their numeric IDs; assign new IDs
        // only to blocks not currently present in the internal list.
        let mut new_boxes = Vec::with_capacity(incoming.len());

        for incoming_block in &incoming {
            if let Some(existing) = current.iter().find(|b| b.id == incoming_block.id) {
                new_boxes.push(existing.clone());
            } else {
                let internal_id = next_internal_id.get_untracked();
                next_internal_id.set(internal_id + 1);

                new_boxes.push(BlockBox {
                    internal_id,
                    id: incoming_block.id.clone(),
                    content: ArcRwSignal::new(incoming_block.content.clone()),
                });
            }
        }

        boxes.set(new_boxes);
    });

    view! {
        <div>
            <For
                each=move || boxes.get()
                key=|b| b.internal_id
                children=move |b| {
                    view! {
                        <BlockItem
                            block=b
                            on_split=on_split
                            on_content_change=on_content_change
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn BlockItem(
    block: BlockBox,
    on_split: Callback<(String, String)>,
    on_content_change: Callback<(String, String)>,
) -> impl IntoView {
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let initial = block.content.get_untracked();
    let my_id_for_focus = block.id.clone();
    let my_id_for_input = block.id.clone();
    let my_id_for_keydown = block.id.clone();
    let content_signal = block.content.clone();
    let content_signal_for_render = content_signal.clone();
    let content_signal_for_focus = content_signal.clone();
    let content_signal_for_keydown = content_signal.clone();
    let on_split_for_keydown = on_split.clone();
    let on_content_change_for_keydown = on_content_change.clone();

    // Keep the textarea synchronized with the internal content signal.
    Effect::new(move |_| {
        let current = content_signal_for_render.get();
        if let Some(el) = textarea_ref.get_untracked() {
            if el.value() != current {
                el.set_value(&current);
            }
        }
    });

    Effect::new(move |_| {
        let Some(target) = FOCUS_BLOCK_AT_START.get() else {
            return;
        };
        if target != my_id_for_focus {
            return;
        }

        if let Some(el) = textarea_ref.get_untracked() {
            let current = content_signal_for_focus.get_untracked();
            if el.value() != current {
                el.set_value(&current);
            }
            let _ = el.focus();
            let pos = current.encode_utf16().count() as u32;
            let _ = el.set_selection_range(pos, pos);
        }

        FOCUS_BLOCK_AT_START.set(None);
    });

    let on_keydown = move |e: ev::KeyboardEvent| {
        if e.key() != "Enter" {
            return;
        }

        e.prevent_default();

        let Some(el) = textarea_ref.get_untracked() else {
            return;
        };

        let current = el.value();
        let start = el.selection_start().ok().flatten().unwrap_or(0) as usize;
        let end = el.selection_end().ok().flatten().unwrap_or(start as u32) as usize;
        let utf16: Vec<u16> = current.encode_utf16().collect();

        if start > end || end > utf16.len() {
            return;
        }

        let before = String::from_utf16_lossy(&utf16[..start]);
        let after = String::from_utf16_lossy(&utf16[end..]);

        content_signal_for_keydown.set(before.clone());
        on_content_change_for_keydown.run((my_id_for_keydown.clone(), before));
        on_split_for_keydown.run((my_id_for_keydown.clone(), after));
    };

    let on_input = move |e: ev::Event| {
        let Some(t) = e.target() else {
            return;
        };
        let ta: web_sys::HtmlTextAreaElement = t.unchecked_into();
        let new_val = ta.value();

        content_signal.set(new_val.clone());
        on_content_change.run((my_id_for_input.clone(), new_val));
    };

    view! {
        <textarea
            node_ref=textarea_ref
            prop:value=initial
            rows="1"
            style="width: 400px; display: block; margin: 4px 0;"
            on:keydown=on_keydown
            on:input=on_input
        ></textarea>
    }
}
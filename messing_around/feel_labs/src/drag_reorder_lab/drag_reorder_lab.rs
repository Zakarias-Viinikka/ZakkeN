use leptos::prelude::*;
use std::cell::Cell;

use super::drag_reorder_black_box::DragReorderBlackBox;
use super::my_block_content::MyBlockContent;
use super::types::{BlockInputEvent, BlockRenderInput, BlockRow, MoveEvent, PageInput};

thread_local! {
    static BLOCK_CTR: Cell<u64> = const { Cell::new(1) };
}

fn new_block_id() -> String {
    BLOCK_CTR.with(|c| {
        let n = c.get();
        c.set(n + 1);
        format!("local-{}", n)
    })
}

fn initial_blocks() -> Vec<BlockRow> {
    vec![
        BlockRow { block_id: "p1-title".into(), page_id: "p1".into(), content: "Page one".into(), is_title: true },
        BlockRow { block_id: "p1-b1".into(), page_id: "p1".into(), content: "Some text on page one.".into(), is_title: false },
        BlockRow { block_id: "p1-b2".into(), page_id: "p1".into(), content: "Another block here.".into(), is_title: false },
        BlockRow { block_id: "p1-b3".into(), page_id: "p1".into(), content: "A third block, draggable.".into(), is_title: false },
        BlockRow { block_id: "p2-title".into(), page_id: "p2".into(), content: "Page two".into(), is_title: true },
        BlockRow { block_id: "p2-b1".into(), page_id: "p2".into(), content: "Blocks can move between pages.".into(), is_title: false },
        BlockRow { block_id: "p2-b2".into(), page_id: "p2".into(), content: "Drag me over to the left.".into(), is_title: false },
    ]
}

#[component]
pub fn DragReorderLab() -> impl IntoView {
    let blocks = RwSignal::new(initial_blocks());
    let focus_block_id = RwSignal::new(None::<String>);
    let focus_block_at_start = RwSignal::new(None::<String>);

    let pages_signal = Signal::derive(move || {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for b in blocks.get() {
            if seen.insert(b.page_id.clone()) {
                out.push(PageInput { page_id: b.page_id.clone() });
            }
        }
        out
    });
    let blocks_signal = blocks.read_only();

    let on_move = Callback::new(move |ev: MoveEvent| {
        blocks.update(|list| {
            let Some(pos) = list.iter().position(|b| b.block_id == ev.moved_block_id) else { return; };
            let mut moved = list.remove(pos);
            moved.page_id = ev.to_page_id.clone();

            let insert_pos = match &ev.above {
                Some(above_id) => list
                    .iter()
                    .position(|b| b.block_id == *above_id)
                    .map(|p| p + 1)
                    .unwrap_or_else(|| {
                        list.iter()
                            .enumerate()
                            .rev()
                            .find(|(_, b)| b.page_id == ev.to_page_id)
                            .map(|(i, _)| i + 1)
                            .unwrap_or(list.len())
                    }),
                None => list
                    .iter()
                    .enumerate()
                    .find(|(_, b)| b.page_id == ev.to_page_id)
                    .map(|(i, _)| i)
                    .unwrap_or(list.len()),
            };
            list.insert(insert_pos, moved);
        });
    });

    let on_input = Callback::new(move |evt: BlockInputEvent| {
        blocks.update(|list| {
            if let Some(b) = list.iter_mut().find(|b| b.block_id == evt.block_id) {
                b.content = evt.new_text;
            }
        });
    });

    let on_split = Callback::new(move |(block_id, before, after): (String, String, String)| {
        let new_id = new_block_id();
        blocks.update(|list| {
            let Some(idx) = list.iter().position(|b| b.block_id == block_id) else { return; };
            let page_id = list[idx].page_id.clone();
            list[idx].content = before;
            list.insert(idx + 1, BlockRow {
                block_id: new_id.clone(),
                page_id,
                content: after,
                is_title: false,
            });
        });
        focus_block_at_start.set(Some(new_id));
    });

    let on_delete = Callback::new(move |block_id: String| {
        let mut above_id: Option<String> = None;
        blocks.update(|list| {
            let Some(idx) = list.iter().position(|b| b.block_id == block_id) else { return; };
            let normal_count = list.iter().filter(|b| !b.is_title).count();
            if normal_count <= 1 { return; }
            above_id = idx.checked_sub(1).map(|i| list[i].block_id.clone());
            list.remove(idx);
        });
        if let Some(id) = above_id {
            focus_block_id.set(Some(id));
        }
    });

    let on_merge = Callback::new(move |(block_id, current_text): (String, String)| {
        let mut focus_next: Option<String> = None;
        blocks.update(|list| {
            let Some(idx) = list.iter().position(|b| b.block_id == block_id) else { return; };
            if idx == 0 { return; }
            if list[idx - 1].is_title { return; }
            let above_idx = idx - 1;
            let above_id = list[above_idx].block_id.clone();
            let above_content = list[above_idx].content.clone();
            let merged = format!("{}{}", above_content, current_text);
            list[above_idx].content = merged;
            list.remove(idx);
            focus_next = Some(above_id);
        });
        if let Some(id) = focus_next {
            focus_block_id.set(Some(id));
        }
    });

    let on_navigate = Callback::new(move |(block_id, go_down): (String, bool)| {
        let list = blocks.get_untracked();
        let Some(idx) = list.iter().position(|b| b.block_id == block_id) else { return; };
        if go_down {
            if let Some(next) = list.get(idx + 1) {
                focus_block_at_start.set(Some(next.block_id.clone()));
            }
        } else if idx > 0 {
            focus_block_id.set(Some(list[idx - 1].block_id.clone()));
        }
    });

    let render_block = Callback::new(move |block: BlockRenderInput| {
        view! {
            <MyBlockContent
                block=block
                on_input=on_input
                on_delete_request=on_delete
                on_merge_with_above=on_merge
                on_split=on_split
                on_navigate=on_navigate
                focus_block_id=focus_block_id
                focus_block_at_start=focus_block_at_start
            />
        }
        .into_any()
    });

    view! {
        <div style="padding: 16px;">
            <DragReorderBlackBox
                pages=pages_signal
                blocks=blocks_signal
                on_move=on_move
                render_block=render_block
            />
        </div>
    }
}

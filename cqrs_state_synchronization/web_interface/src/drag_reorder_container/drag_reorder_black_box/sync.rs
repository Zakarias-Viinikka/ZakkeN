use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::cascade::release_pass;
use super::drag_state::{DragState, RectDragState};
use super::layout::{Block, Phase, commit, UNMEASURED_HEIGHT};
use super::rapier_world::RapierWorld;
use super::rebuild::rebuild_world;
use crate::complicated_signal_flow::BlockRow;
use super::types::PageInput;

pub fn sync_from_props(
    pages: Signal<Vec<PageInput>>,
    blocks: Signal<Vec<BlockRow>>,
    boxes: RwSignal<Vec<Block>>,
    id_map: RwSignal<HashMap<String, usize>>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    next_id: RwSignal<usize>,
    stored: RwSignal<Option<usize>>,
    drag: RwSignal<Option<DragState>>,
    rect_drag: RwSignal<Option<RectDragState>>,
    rapier: &RapierWorld,
) {
    let incoming_pages = pages.get();
    let incoming_blocks = blocks.get();

    let current_boxes = boxes.get_untracked();
    let current_map = id_map.get_untracked();
    let current_pids = current_page_ids.get_untracked();

    // Keep the black box's content signals in sync with the incoming
    // block rows. Needed when a block's text changes without the user
    // typing in it (e.g. another block merged into this one).
    for incoming in &incoming_blocks {
        if let Some(b) = current_boxes
            .iter()
            .find(|b| b.block_id.as_ref() == incoming.block_id.as_str())
        {
            if b.content.get_untracked() != incoming.content {
                b.content.set(incoming.content.clone());
            }
        }
    }

    let incoming_pids = incoming_pages
        .iter()
        .map(|p| p.page_id.clone())
        .collect::<Vec<_>>();
    let mut incoming_arrangement = Vec::new();
    for page in &incoming_pages {
        let page_blocks = incoming_blocks
            .iter()
            .filter(|b| b.page_id == page.page_id)
            .map(|b| b.block_id.clone())
            .collect::<Vec<_>>();
        incoming_arrangement.push(page_blocks);
    }

    let mut current_arrangement = Vec::new();
    for p in 0..page_count.get_untracked() {
        current_arrangement.push(
            rapier
                .page_order(p)
                .into_iter()
                .map(|id| {
                    current_boxes
                        .iter()
                        .find(|b| b.internal_id == id)
                        .map(|b| b.block_id.to_string())
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>(),
        );
    }

    if incoming_pids == current_pids && incoming_arrangement == current_arrangement {
        return;
    }

    // Pure delete: same pages, same block count on every page except one,
    // where exactly one block is gone and the rest keep their order.
    if incoming_pids == current_pids && incoming_arrangement.len() == current_arrangement.len() {
        let mut removed: Option<String> = None;
        let mut only_one_page_changed = true;
        let mut changed_pages = 0;
        for (inc, cur) in incoming_arrangement.iter().zip(current_arrangement.iter()) {
            if inc == cur {
                continue;
            }
            changed_pages += 1;
            if cur.len() != inc.len() + 1 {
                only_one_page_changed = false;
                break;
            }
            let mut gone: Option<&String> = None;
            let mut j = 0;
            for c in cur.iter() {
                if j < inc.len() && &inc[j] == c {
                    j += 1;
                } else if gone.is_none() {
                    gone = Some(c);
                } else {
                    only_one_page_changed = false;
                    break;
                }
            }
            if !only_one_page_changed || j != inc.len() {
                only_one_page_changed = false;
                break;
            }
            if let Some(g) = gone {
                removed = Some(g.clone());
            }
        }
        if only_one_page_changed && changed_pages == 1 {
            if let Some(removed_block_id) = removed {
                fast_delete(
                    removed_block_id,
                    id_map,
                    &incoming_pages,
                    &incoming_arrangement,
                    &current_boxes,
                    boxes,
                    page_count,
                    current_page_ids,
                    stored,
                    drag,
                    rect_drag,
                    rapier,
                    incoming_pids,
                );
                return;
            }
        }
    }

    // Pure add: same pages, same page count, one page gained exactly
    // one block, order otherwise preserved.
    if incoming_pids == current_pids && incoming_arrangement.len() == current_arrangement.len() {
        let mut added: Option<String> = None;
        let mut only_one_page_changed = true;
        let mut changed_pages = 0;
        for (inc, cur) in incoming_arrangement.iter().zip(current_arrangement.iter()) {
            if inc == cur {
                continue;
            }
            changed_pages += 1;
            if inc.len() != cur.len() + 1 {
                only_one_page_changed = false;
                break;
            }
            let mut new_one: Option<&String> = None;
            let mut j = 0;
            for item in inc.iter() {
                if j < cur.len() && &cur[j] == item {
                    j += 1;
                } else if new_one.is_none() {
                    new_one = Some(item);
                } else {
                    only_one_page_changed = false;
                    break;
                }
            }
            if !only_one_page_changed || j != cur.len() {
                only_one_page_changed = false;
                break;
            }
            if let Some(n) = new_one {
                added = Some(n.clone());
            }
        }
        if only_one_page_changed && changed_pages == 1 {
            if let Some(added_block_id) = added {
                fast_add(
                    added_block_id,
                    id_map,
                    &incoming_pages,
                    &incoming_arrangement,
                    &incoming_blocks,
                    &current_boxes,
                    boxes,
                    next_id,
                    page_count,
                    current_page_ids,
                    drag,
                    rect_drag,
                    rapier,
                    incoming_pids,
                );
                return;
            }
        }
    }

    let mut new_map = HashMap::new();
    let mut new_boxes = Vec::new();

    for page in &incoming_pages {
        for block in &incoming_blocks {
            if block.page_id != page.page_id {
                continue;
            }
            let internal_id = if let Some(&id) = current_map.get(&block.block_id) {
                id
            } else {
                let id = next_id.get_untracked();
                next_id.set(id + 1);
                id
            };
            new_map.insert(block.block_id.clone(), internal_id);

            if let Some(existing) = current_boxes.iter().find(|b| b.internal_id == internal_id) {
                let mut e = existing.clone();
                e.is_title = block.is_title;
                new_boxes.push(e);
            } else {
                new_boxes.push(Block {
                    internal_id,
                    block_id: Arc::from(block.block_id.as_str()),
                    content: ArcRwSignal::new(block.content.clone()),
                    x: 0.0,
                    y: 0.0,
                    target_y: 0.0,
                    vel: 0.0,
                    h: UNMEASURED_HEIGHT,
                    phase: Phase::Idle,
                    prev: None,
                    ghost: false,
                    measured: false,
                    is_title: block.is_title,
                });
            }
        }
    }

    if let Some(stored_id) = stored.get_untracked() {
        if !new_map.values().any(|&id| id == stored_id) {
            stored.set(None);
        }
    }

    let orders: Vec<Vec<usize>> = incoming_pages
        .iter()
        .map(|p| {
            incoming_blocks
                .iter()
                .filter(|b| b.page_id == p.page_id)
                .map(|b| new_map[&b.block_id])
                .collect()
        })
        .collect();

    let snap_ids = new_boxes.iter().map(|b| b.internal_id).collect::<Vec<_>>();
    commit(&mut new_boxes, &orders, &snap_ids);
    boxes.set(new_boxes.clone());
    id_map.set(new_map);
    current_page_ids.set(incoming_pids);
    page_count.set(incoming_pages.len());
    rebuild_world(rapier, &new_boxes, incoming_pages.len());

    drag.set(None);
    rect_drag.set(None);
}

#[allow(clippy::too_many_arguments)]
fn fast_delete(
    removed_block_id: String,
    id_map: RwSignal<HashMap<String, usize>>,
    incoming_pages: &[PageInput],
    incoming_arrangement: &[Vec<String>],
    current_boxes: &[Block],
    boxes: RwSignal<Vec<Block>>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    stored: RwSignal<Option<usize>>,
    drag: RwSignal<Option<DragState>>,
    rect_drag: RwSignal<Option<RectDragState>>,
    rapier: &RapierWorld,
    incoming_pids: Vec<String>,
) {
    let mut new_boxes: Vec<Block> = current_boxes
        .iter()
        .filter(|b| b.block_id.as_ref() != removed_block_id.as_str())
        .cloned()
        .collect();

    if let Some(stored_id) = stored.get_untracked() {
        if !new_boxes.iter().any(|b| b.internal_id == stored_id) {
            stored.set(None);
        }
    }

    let id_of: HashMap<&str, usize> = new_boxes
        .iter()
        .map(|b| (b.block_id.as_ref(), b.internal_id))
        .collect();

    let orders: Vec<Vec<usize>> = incoming_arrangement
        .iter()
        .map(|page| {
            page.iter()
                .filter_map(|bid| id_of.get(bid.as_str()).copied())
                .collect()
        })
        .collect();

    commit(&mut new_boxes, &orders, &[]);
    release_pass(&mut new_boxes);
    boxes.set(new_boxes.clone());
    id_map.set(
        new_boxes
            .iter()
            .map(|b| (b.block_id.to_string(), b.internal_id))
            .collect(),
    );
    current_page_ids.set(incoming_pids);
    page_count.set(incoming_pages.len());
    rebuild_world(rapier, &new_boxes, incoming_pages.len());

    drag.set(None);
    rect_drag.set(None);
}

#[allow(clippy::too_many_arguments)]
fn fast_add(
    added_block_id: String,
    id_map: RwSignal<HashMap<String, usize>>,
    incoming_pages: &[PageInput],
    incoming_arrangement: &[Vec<String>],
    incoming_blocks: &[BlockRow],
    current_boxes: &[Block],
    boxes: RwSignal<Vec<Block>>,
    next_id: RwSignal<usize>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    drag: RwSignal<Option<DragState>>,
    rect_drag: RwSignal<Option<RectDragState>>,
    rapier: &RapierWorld,
    incoming_pids: Vec<String>,
) {
    let Some(added_row) = incoming_blocks.iter().find(|b| b.block_id == added_block_id) else {
        return;
    };

    let mut new_boxes: Vec<Block> = current_boxes.to_vec();
    let internal_id = next_id.get_untracked();
    next_id.set(internal_id + 1);
    new_boxes.push(Block {
        internal_id,
        block_id: Arc::from(added_row.block_id.as_str()),
        content: ArcRwSignal::new(added_row.content.clone()),
        x: 0.0,
        y: 0.0,
        target_y: 0.0,
        vel: 0.0,
        h: UNMEASURED_HEIGHT,
        phase: Phase::Idle,
        prev: None,
        ghost: false,
        measured: false,
        is_title: added_row.is_title,
    });

    let id_of: HashMap<&str, usize> = new_boxes
        .iter()
        .map(|b| (b.block_id.as_ref(), b.internal_id))
        .collect();

    let orders: Vec<Vec<usize>> = incoming_arrangement
        .iter()
        .map(|page| {
            page.iter()
                .filter_map(|bid| id_of.get(bid.as_str()).copied())
                .collect()
        })
        .collect();

    commit(&mut new_boxes, &orders, &[internal_id]);
    release_pass(&mut new_boxes);
    boxes.set(new_boxes.clone());
    id_map.set(
        new_boxes
            .iter()
            .map(|b| (b.block_id.to_string(), b.internal_id))
            .collect(),
    );
    current_page_ids.set(incoming_pids);
    page_count.set(incoming_pages.len());
    rebuild_world(rapier, &new_boxes, incoming_pages.len());

    drag.set(None);
    rect_drag.set(None);
}

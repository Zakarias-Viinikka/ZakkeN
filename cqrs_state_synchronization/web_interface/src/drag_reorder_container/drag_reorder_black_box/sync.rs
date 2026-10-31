use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::cascade::release_pass;
use super::drag_state::{DragState, RectDragState};
use super::layout::{Block, Phase, commit, UNMEASURED_HEIGHT};
use super::rapier_world::RapierWorld;
use super::rebuild::rebuild_world;
use crate::shared_owners::BlockRow;
use super::types::PageInput;

#[derive(Clone, Copy)]
pub(crate) struct SyncState<'a> {
    pub(crate) boxes: RwSignal<Vec<Block>>,
    pub(crate) id_map: RwSignal<HashMap<String, usize>>,
    pub(crate) page_count: RwSignal<usize>,
    pub(crate) current_page_ids: RwSignal<Vec<String>>,
    pub(crate) next_id: RwSignal<usize>,
    pub(crate) stored: RwSignal<Option<usize>>,
    pub(crate) drag: RwSignal<Option<DragState>>,
    pub(crate) rect_drag: RwSignal<Option<RectDragState>>,
    pub(crate) rapier: &'a RapierWorld,
}

enum SingleChange {
    Removed(String),
    Added(String),
}

pub(crate) fn sync_from_props(
    pages: Signal<Vec<PageInput>>,
    blocks: Signal<Vec<BlockRow>>,
    state: SyncState,
) {
    let incoming_pages = pages.get();
    let incoming_blocks = blocks.get();

    let current_boxes = state.boxes.get_untracked();
    let current_map = state.id_map.get_untracked();
    let current_pids = state.current_page_ids.get_untracked();

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
    for p in 0..state.page_count.get_untracked() {
        current_arrangement.push(
            state
                .rapier
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

    if incoming_pids == current_pids {
        if let Some(change) = single_block_change(&incoming_arrangement, &current_arrangement) {
            match change {
                SingleChange::Removed(id) => {
                    fast_delete(
                        state,
                        id,
                        &incoming_pages,
                        &incoming_arrangement,
                        &current_boxes,
                        incoming_pids,
                    );
                }
                SingleChange::Added(id) => {
                    fast_add(
                        state,
                        id,
                        &incoming_pages,
                        &incoming_arrangement,
                        &incoming_blocks,
                        &current_boxes,
                        incoming_pids,
                    );
                }
            }
            return;
        }
    }

    full_rebuild(
        state,
        &incoming_pages,
        &incoming_blocks,
        &current_map,
        &current_boxes,
        incoming_pids,
    );
}

fn single_block_change(
    incoming: &[Vec<String>],
    current: &[Vec<String>],
) -> Option<SingleChange> {
    if incoming.len() != current.len() {
        return None;
    }
    let mut change: Option<SingleChange> = None;
    let mut changed_pages = 0;
    for (inc, cur) in incoming.iter().zip(current.iter()) {
        if inc == cur {
            continue;
        }
        changed_pages += 1;
        if changed_pages > 1 {
            return None;
        }
        change = Some(one_block_diff(inc, cur)?);
    }
    if changed_pages == 1 { change } else { None }
}

fn one_block_diff(inc: &[String], cur: &[String]) -> Option<SingleChange> {
    if cur.len() == inc.len() + 1 {
        detect_single_extra(cur, inc, SingleChange::Removed)
    } else if inc.len() == cur.len() + 1 {
        detect_single_extra(inc, cur, SingleChange::Added)
    } else {
        None
    }
}

fn detect_single_extra(
    longer: &[String],
    shorter: &[String],
    wrap: fn(String) -> SingleChange,
) -> Option<SingleChange> {
    let mut j = 0;
    let mut extra = None;
    for item in longer {
        if j < shorter.len() && &shorter[j] == item {
            j += 1;
        } else if extra.is_none() {
            extra = Some(item.clone());
        } else {
            return None;
        }
    }
    if j == shorter.len() { extra.map(wrap) } else { None }
}

fn fast_delete(
    state: SyncState,
    removed_block_id: String,
    incoming_pages: &[PageInput],
    incoming_arrangement: &[Vec<String>],
    current_boxes: &[Block],
    incoming_pids: Vec<String>,
) {
    let mut new_boxes: Vec<Block> = current_boxes
        .iter()
        .filter(|b| b.block_id.as_ref() != removed_block_id.as_str())
        .cloned()
        .collect();

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
    finish_sync(state, new_boxes, incoming_pages, incoming_pids);
}

#[allow(clippy::too_many_arguments)]
fn fast_add(
    state: SyncState,
    added_block_id: String,
    incoming_pages: &[PageInput],
    incoming_arrangement: &[Vec<String>],
    incoming_blocks: &[BlockRow],
    current_boxes: &[Block],
    incoming_pids: Vec<String>,
) {
    let Some(added_row) = incoming_blocks.iter().find(|b| b.block_id == added_block_id) else {
        return;
    };

    let mut new_boxes: Vec<Block> = current_boxes.to_vec();
    let internal_id = state.next_id.get_untracked();
    state.next_id.set(internal_id + 1);
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
    finish_sync(state, new_boxes, incoming_pages, incoming_pids);
}

fn full_rebuild(
    state: SyncState,
    incoming_pages: &[PageInput],
    incoming_blocks: &[BlockRow],
    current_map: &HashMap<String, usize>,
    current_boxes: &[Block],
    incoming_pids: Vec<String>,
) {
    let mut new_boxes = Vec::new();
    for page in incoming_pages {
        for block in incoming_blocks {
            if block.page_id != page.page_id {
                continue;
            }
            let internal_id = if let Some(&id) = current_map.get(&block.block_id) {
                id
            } else {
                let id = state.next_id.get_untracked();
                state.next_id.set(id + 1);
                id
            };
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

    let id_of: HashMap<&str, usize> = new_boxes
        .iter()
        .map(|b| (b.block_id.as_ref(), b.internal_id))
        .collect();

    let orders: Vec<Vec<usize>> = incoming_pages
        .iter()
        .map(|p| {
            incoming_blocks
                .iter()
                .filter(|b| b.page_id == p.page_id)
                .map(|b| id_of[b.block_id.as_str()])
                .collect()
        })
        .collect();

    let snap_ids: Vec<usize> = new_boxes.iter().map(|b| b.internal_id).collect();
    commit(&mut new_boxes, &orders, &snap_ids);
    finish_sync(state, new_boxes, incoming_pages, incoming_pids);
}

fn finish_sync(
    state: SyncState,
    new_boxes: Vec<Block>,
    incoming_pages: &[PageInput],
    incoming_pids: Vec<String>,
) {
    if let Some(stored_id) = state.stored.get_untracked() {
        if !new_boxes.iter().any(|b| b.internal_id == stored_id) {
            state.stored.set(None);
        }
    }
    let new_map: HashMap<String, usize> = new_boxes
        .iter()
        .map(|b| (b.block_id.to_string(), b.internal_id))
        .collect();
    state.boxes.set(new_boxes.clone());
    state.id_map.set(new_map);
    state.current_page_ids.set(incoming_pids);
    state.page_count.set(incoming_pages.len());
    rebuild_world(state.rapier, &new_boxes, incoming_pages.len());
    state.drag.set(None);
    state.rect_drag.set(None);
}
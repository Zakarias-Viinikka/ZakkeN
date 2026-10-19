use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

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
    current_page_ids.set(incoming_pids);
    page_count.set(incoming_pages.len());
    rebuild_world(rapier, &new_boxes, incoming_pages.len());

    drag.set(None);
    rect_drag.set(None);
}

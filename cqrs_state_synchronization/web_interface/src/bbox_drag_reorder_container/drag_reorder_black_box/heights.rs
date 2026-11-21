use leptos::prelude::*;

use super::layout::{Block, commit};
use super::rapier_world::RapierWorld;
use super::rebuild::rebuild_world;

pub fn apply_heights(
    pairs: Vec<(usize, f32)>,
    boxes: RwSignal<Vec<Block>>,
    rapier: &RapierWorld,
    page_count: RwSignal<usize>,
) {
    let mut bs = boxes.get_untracked();
    let mut any_moved = false;
    for (id, h) in pairs {
        if let Some(b) = bs.iter_mut().find(|b| b.internal_id == id) {
            if !b.measured || (b.h - h).abs() >= 0.5 {
                any_moved = true;
            }
            b.h = h;
            b.measured = true;
        }
    }
    if !any_moved {
        return;
    }
    let pc = page_count.get_untracked();
    let mut all_orders: Vec<Vec<usize>> = Vec::with_capacity(pc);
    for p in 0..pc {
        all_orders.push(rapier.page_order(p));
    }
    let snap_ids: Vec<usize> = bs.iter().map(|b| b.internal_id).collect();
    commit(&mut bs, &all_orders, &snap_ids);
    boxes.set(bs.clone());
    rebuild_world(rapier, &bs, pc);
}

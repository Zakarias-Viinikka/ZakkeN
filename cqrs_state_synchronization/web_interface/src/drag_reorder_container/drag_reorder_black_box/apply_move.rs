use leptos::prelude::*;

use super::cascade::release_pass;
use super::layout::{Block, commit};
use super::rapier_world::RapierWorld;
use super::rebuild::rebuild_world;
use super::types::MoveEvent;

#[allow(clippy::too_many_arguments)]
pub fn apply_move(
    moved_id: usize,
    dest_page: usize,
    above: Option<usize>,
    is_restore: bool,
    boxes: RwSignal<Vec<Block>>,
    rapier: &RapierWorld,
    stored: RwSignal<Option<usize>>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    on_move: Callback<MoveEvent>,
) {
    let pc = page_count.get_untracked();
    let Some(src) = (0..pc).find(|&p| rapier.page_order(p).contains(&moved_id)) else {
        return;
    };

    // title never moves
    if boxes
        .get_untracked()
        .iter()
        .find(|b| b.internal_id == moved_id)
        .map(|b| b.is_title)
        .unwrap_or(false)
    {
        return;
    }

    let mut all_orders: Vec<Vec<usize>> = Vec::with_capacity(pc);
    for p in 0..pc {
        all_orders.push(
            rapier
                .page_order(p)
                .into_iter()
                .filter(|&i| i != moved_id)
                .collect(),
        );
    }
    let mut insert_at = match above {
        None => 0,
        Some(a) => all_orders[dest_page]
            .iter()
            .position(|&i| i == a)
            .map(|p| p + 1)
            .unwrap_or(all_orders[dest_page].len()),
    };

    // never insert above the title
    if insert_at == 0 {
        if let Some(&first_id) = all_orders[dest_page].first() {
            let first_is_title = boxes
                .get_untracked()
                .iter()
                .find(|b| b.internal_id == first_id)
                .map(|b| b.is_title)
                .unwrap_or(false);
            if first_is_title {
                insert_at = 1;
            }
        }
    }

    all_orders[dest_page].insert(insert_at, moved_id);

    let mut bs = boxes.get_untracked();

    if is_restore {
        if let Some(b) = bs.iter_mut().find(|b| b.internal_id == moved_id) {
            b.ghost = false;
        }
        stored.set(None);
        leptos::logging::log!("vacuumed");
    }

    commit(&mut bs, &all_orders, &[moved_id]);
    release_pass(&mut bs);
    boxes.set(bs.clone());
    rebuild_world(rapier, &bs, pc);

    let dest_order = &all_orders[dest_page];
    let pos = dest_order.iter().position(|&i| i == moved_id).unwrap_or(0);
    let above_id = if pos > 0 { Some(dest_order[pos - 1]) } else { None };
    let below_id = if pos + 1 < dest_order.len() {
        Some(dest_order[pos + 1])
    } else {
        None
    };

    let moved_str = bs
        .iter()
        .find(|b| b.internal_id == moved_id)
        .map(|b| b.block_id.to_string())
        .unwrap_or_default();
    let pids = current_page_ids.get_untracked();
    let from_str = pids.get(src).cloned().unwrap_or_default();
    let to_str = pids.get(dest_page).cloned().unwrap_or_default();
    let above_str = above_id.and_then(|id| {
        bs.iter()
            .find(|b| b.internal_id == id)
            .map(|b| b.block_id.to_string())
    });
    let below_str = below_id.and_then(|id| {
        bs.iter()
            .find(|b| b.internal_id == id)
            .map(|b| b.block_id.to_string())
    });

    on_move.run(MoveEvent {
        moved_block_id: moved_str,
        from_page_id: from_str,
        to_page_id: to_str,
        above: above_str,
        below: below_str,
    });
}

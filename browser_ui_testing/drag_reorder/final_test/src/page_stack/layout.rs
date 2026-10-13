use std::collections::HashMap;
use leptos::prelude::ArcRwSignal;
use std::sync::Arc;

pub const PAGE_W: f32 = 480.0;
pub const PAGE_GAP: f32 = 60.0;
pub const PAGE_START_X: f32 = 190.0;
pub const PAGE_TOP: f32 = 24.0;
pub const BLOCK_W: f32 = 432.0;
pub const BLOCK_PAD_X: f32 = 24.0;
pub const EDGE_PAD: f32 = 48.0;
pub const GAP: f32 = 16.0;
pub const STAGE_MIN_H: f32 = 720.0;
pub const BOTTOM_SLACK: f32 = 140.0;
pub const VACUUM_W: f32 = 120.0;
pub const VACUUM_PROXIMITY: f32 = 40.0;
pub const RECT_W: f32 = 60.0;
pub const RECT_H: f32 = 80.0;
pub const RECT_HOME: (f32, f32) = (30.0, 80.0);
pub const UNMEASURED_HEIGHT: f32 = 56.0;

pub fn page_x(page_index: usize) -> f32 {
    PAGE_START_X + (page_index as f32) * (PAGE_W + PAGE_GAP)
}

pub fn block_x(page_index: usize) -> f32 {
    page_x(page_index) + BLOCK_PAD_X
}

pub fn stage_width(page_count: usize) -> f32 {
    if page_count == 0 {
        return 0.0;
    }
    PAGE_START_X + (page_count as f32) * PAGE_W + ((page_count - 1) as f32) * PAGE_GAP + 40.0
}

pub fn page_panel_height(stage_h: f32) -> f32 {
    (stage_h - PAGE_TOP - 20.0).max(0.0)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Waiting,
    Moving,
}

#[derive(Clone)]
pub struct Block {
    pub internal_id: usize,
    pub block_id: Arc<str>,
    pub content: ArcRwSignal<String>,
    pub x: f32,
    pub y: f32,
    pub target_y: f32,
    pub vel: f32,
    pub h: f32,
    pub phase: Phase,
    pub prev: Option<usize>,
    pub ghost: bool,
    pub measured: bool,
}

pub fn commit(blocks: &mut Vec<Block>, orders: &[Vec<usize>], snap_ids: &[usize]) {
    let old_targets: HashMap<usize, f32> = blocks.iter().map(|b| (b.internal_id, b.target_y)).collect();
    let idx_of: HashMap<usize, usize> = blocks.iter().enumerate().map(|(i, b)| (b.internal_id, i)).collect();

    for (page_idx, page_order) in orders.iter().enumerate() {
        let mut y = PAGE_TOP + EDGE_PAD;
        let mut prev_id: Option<usize> = None;
        for &id in page_order {
            let Some(&idx) = idx_of.get(&id) else { continue; };
            let b = &mut blocks[idx];
            b.x = block_x(page_idx);
            b.prev = prev_id;
            let new_target = y;
            let old_y = b.y;
            let old_target = old_targets.get(&id).copied().unwrap_or(b.target_y);

            if snap_ids.contains(&id) {
                b.target_y = new_target;
                b.y = new_target;
                b.vel = 0.0;
                b.phase = Phase::Idle;
            } else {
                b.target_y = new_target;
                if (old_y - new_target).abs() < 0.5 {
                    b.y = new_target;
                    b.vel = 0.0;
                    b.phase = Phase::Idle;
                } else {
                    match b.phase {
                        Phase::Idle => b.phase = Phase::Waiting,
                        Phase::Waiting => {}
                        Phase::Moving => {
                            let old_dir = (old_target - old_y).signum();
                            let new_dir = (new_target - b.y).signum();
                            if old_dir != new_dir {
                                b.vel = 0.0;
                            }
                        }
                    }
                }
            }

            let current_h = if b.measured { b.h } else { UNMEASURED_HEIGHT };
            y = new_target + current_h + GAP;
            prev_id = Some(id);
        }
    }
}

pub fn stage_height(blocks: &[Block]) -> f32 {
    let mut h = STAGE_MIN_H;
    for b in blocks {
        let current_h = if b.measured { b.h } else { UNMEASURED_HEIGHT };
        let bottom = b.y.max(b.target_y) + current_h;
        let needed = bottom + BOTTOM_SLACK;
        if needed > h {
            h = needed;
        }
    }
    h
}

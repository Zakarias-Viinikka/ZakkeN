use std::collections::HashMap;

pub const STAGE_W: f32 = 1250.0;
pub const STAGE_MIN_H: f32 = 720.0;
pub const BOTTOM_SLACK: f32 = 140.0;

pub const PAGE_W: f32 = 480.0;
pub const PAGE_TOP: f32 = 24.0;
pub const PAGE_PAD: f32 = 24.0;
pub const PAGE_X: [f32; 2] = [190.0, 730.0];
pub const PAGE_SLOP: f32 = 16.0;

pub const BLOCK_W: f32 = 432.0;
pub const BLOCK_X: [f32; 2] = [214.0, 754.0];
pub const EDGE_PAD: f32 = 48.0;
pub const GAP: f32 = 16.0;

pub const VACUUM_W: f32 = 120.0;
pub const VACUUM_PROXIMITY: f32 = 40.0;

pub const RECT_W: f32 = 60.0;
pub const RECT_H: f32 = 80.0;
pub const RECT_HOME: (f32, f32) = (30.0, 80.0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Waiting,
    Moving,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: usize,
    pub label: &'static str,
    pub x: f32,
    pub y: f32,
    pub target_y: f32,
    pub vel: f32,
    pub h: f32,
    pub phase: Phase,
    pub prev: Option<usize>,
}

/// The only place that changes order, page, targets, or prev. `orders` is the
/// final top-to-bottom id list for each page. `snap_ids` are placed exactly at
/// their new target with no animation; every other block either snaps (if it
/// is already within half a pixel), stays Waiting, or keeps Moving and keeps
/// its velocity unless its direction flipped.
pub fn commit(blocks: &mut Vec<Block>, orders: &[Vec<usize>; 2], snap_ids: &[usize]) {
    let old_targets: HashMap<usize, f32> =
        blocks.iter().map(|b| (b.id, b.target_y)).collect();
    let idx_of: HashMap<usize, usize> = blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.id, i))
        .collect();

    for page in 0..2 {
        let mut y = PAGE_TOP + EDGE_PAD;
        let mut prev_id: Option<usize> = None;
        for &id in &orders[page] {
            let Some(&idx) = idx_of.get(&id) else {
                continue;
            };
            let b = &mut blocks[idx];
            b.x = BLOCK_X[page];
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

            y = new_target + b.h + GAP;
            prev_id = Some(id);
        }
    }
}

/// Stage height is the maximum of the min height and the bottom-most point of
/// any block (either its drawn y or its target) plus slack. Grows the instant
/// a drop is committed; shrinks only after blocks have climbed.
pub fn stage_height(blocks: &[Block]) -> f32 {
    let mut h = STAGE_MIN_H;
    for b in blocks {
        let bottom = b.y.max(b.target_y) + b.h;
        let needed = bottom + BOTTOM_SLACK;
        if needed > h {
            h = needed;
        }
    }
    h
}

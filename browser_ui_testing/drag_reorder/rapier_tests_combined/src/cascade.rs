use crate::layout::{Block, Phase, GAP};

pub const ACCEL: f32 = 1500.0;
pub const MAX_V: f32 = 600.0;
pub const DT: f32 = 0.016;
pub const CONTACT: f32 = 3.0;
pub const TRIGGER: f32 = GAP - CONTACT;

/// One frame. Returns whether anything was active this frame; callers only
/// write the signal when it returns true, so an idle page does no work.
pub fn step(blocks: &mut [Block]) -> bool {
    let mut did_work = false;

    for b in blocks.iter_mut() {
        if b.phase != Phase::Moving {
            continue;
        }
        did_work = true;
        let dir = (b.target_y - b.y).signum();
        b.vel = (b.vel + ACCEL * DT).min(MAX_V);
        b.y += dir * b.vel * DT;
        let reached = (dir > 0.0 && b.y >= b.target_y) || (dir < 0.0 && b.y <= b.target_y);
        if reached {
            b.y = b.target_y;
            b.vel = 0.0;
            b.phase = Phase::Idle;
        }
    }

    let snapshot: Vec<Block> = blocks.to_vec();
    let mut releases: Vec<(usize, f32)> = Vec::new();
    for b in &snapshot {
        if b.phase != Phase::Waiting {
            continue;
        }
        if should_release(b, &snapshot) {
            did_work = true;
            releases.push((b.id, inherit_vel(b, &snapshot)));
        }
    }
    for (id, vel) in releases {
        if let Some(b) = blocks.iter_mut().find(|x| x.id == id) {
            b.phase = Phase::Moving;
            b.vel = vel;
        }
    }

    did_work
}

/// The release rule on its own. Called once right after a drop's commit so
/// the head of each shifted run starts in the same frame, without waiting for
/// the next tick.
pub fn release_pass(blocks: &mut [Block]) {
    let snapshot: Vec<Block> = blocks.to_vec();
    let mut releases: Vec<(usize, f32)> = Vec::new();
    for b in &snapshot {
        if b.phase != Phase::Waiting {
            continue;
        }
        if should_release(b, &snapshot) {
            releases.push((b.id, inherit_vel(b, &snapshot)));
        }
    }
    for (id, vel) in releases {
        if let Some(b) = blocks.iter_mut().find(|x| x.id == id) {
            b.phase = Phase::Moving;
            b.vel = vel;
        }
    }
}

/// A Waiting block starts moving when it has no prev, its prev is Idle, or
/// the gap between it and its prev has drifted by at least TRIGGER from GAP.
/// That single rule produces the bump on downward shifts (gap closes) and the
/// pull on upward shifts (gap opens). The Idle clause rescues a chain whose
/// leader finished before covering TRIGGER.
fn should_release(b: &Block, snapshot: &[Block]) -> bool {
    match b.prev {
        None => true,
        Some(pid) => match snapshot.iter().find(|p| p.id == pid) {
            None => true,
            Some(p) => {
                if p.phase == Phase::Idle {
                    true
                } else {
                    let gap = (b.y - (p.y + p.h)) - GAP;
                    gap.abs() >= TRIGGER
                }
            }
        },
    }
}

fn inherit_vel(b: &Block, snapshot: &[Block]) -> f32 {
    let Some(pid) = b.prev else {
        return 0.0;
    };
    let Some(p) = snapshot.iter().find(|p| p.id == pid) else {
        return 0.0;
    };
    if p.phase != Phase::Moving {
        return 0.0;
    }
    let pdir = (p.target_y - p.y).signum();
    let bdir = (b.target_y - b.y).signum();
    if pdir == bdir && pdir != 0.0 {
        p.vel
    } else {
        0.0
    }
}

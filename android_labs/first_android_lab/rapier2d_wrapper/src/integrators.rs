use crate::types::{
    Block, Mode, Phase, ACCEL, CONTACT, MAX_V, SPRING_C, SPRING_K, SPRING_MAX_V,
    SPRING_SETTLE_POS, SPRING_SETTLE_VEL,
};

pub fn integrate_fall(b: &mut Block, dt: f32) {
    let dir = (b.target_y - b.y).signum();
    if dir == 0.0 {
        b.y = b.target_y;
        b.vel = 0.0;
        b.phase = Phase::Idle;
        return;
    }
    b.vel = (b.vel + ACCEL * dt).min(MAX_V);
    b.y += dir * b.vel * dt;
    let reached = (dir > 0.0 && b.y >= b.target_y) || (dir < 0.0 && b.y <= b.target_y);
    if reached {
        b.y = b.target_y;
        b.vel = 0.0;
        b.phase = Phase::Idle;
    }
}

pub fn integrate_spring(b: &mut Block, dt: f32) {
    let disp = b.y - b.target_y;
    let accel = -SPRING_K * disp - SPRING_C * b.vel;
    b.vel += accel * dt;
    if b.vel > SPRING_MAX_V {
        b.vel = SPRING_MAX_V;
    } else if b.vel < -SPRING_MAX_V {
        b.vel = -SPRING_MAX_V;
    }
    b.y += b.vel * dt;
    if disp.abs() < SPRING_SETTLE_POS && b.vel.abs() < SPRING_SETTLE_VEL {
        b.y = b.target_y;
        b.vel = 0.0;
        b.phase = Phase::Idle;
    }
}

pub fn should_release(b: &Block, all: &[Block], mode: Mode, block_gap: f32) -> bool {
    match mode {
        Mode::Fall => {
            let trigger = block_gap - CONTACT;
            match b.prev {
                None => true,
                Some(pid) => match all.iter().find(|p| p.id == pid) {
                    None => true,
                    Some(p) => {
                        if p.phase == Phase::Idle {
                            true
                        } else {
                            let gap = (b.y - (p.y + p.h)) - block_gap;
                            gap.abs() >= trigger
                        }
                    }
                },
            }
        }
        Mode::Spring => true,
        Mode::Ripple => b.delay == 0,
    }
}

pub fn inherit_vel(b: &Block, all: &[Block]) -> f32 {
    let Some(pid) = b.prev else { return 0.0 };
    let Some(p) = all.iter().find(|p| p.id == pid) else { return 0.0 };
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

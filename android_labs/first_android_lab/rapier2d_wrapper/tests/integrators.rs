use rapier2d_wrapper::integrators::{
    inherit_vel, integrate_fall, integrate_spring, should_release,
};
use rapier2d_wrapper::types::{Block, Mode, Phase};

fn moving_block(id: u64, y: f32, target_y: f32, h: f32) -> Block {
    Block {
        id,
        page: 0,
        y,
        target_y,
        h,
        vel: 0.0,
        phase: Phase::Moving,
        prev: None,
        delay: 0,
    }
}

fn idle_block(id: u64, y: f32, h: f32, prev: Option<u64>) -> Block {
    Block {
        id,
        page: 0,
        y,
        target_y: y,
        h,
        vel: 0.0,
        phase: Phase::Idle,
        prev,
        delay: 0,
    }
}

#[test]
fn fall_54dp_takes_about_268ms() {
    let mut b = moving_block(1, 0.0, 54.0, 44.0);
    let mut steps = 0u32;
    while b.phase != Phase::Idle {
        integrate_fall(&mut b, 0.016);
        steps += 1;
        assert!(steps < 1000, "fall never settled");
    }
    let ms = steps as f32 * 0.016 * 1000.0;
    assert!(ms >= 250.0 && ms <= 290.0, "fall took {} ms", ms);
}

#[test]
fn fall_hard_stops_at_target() {
    let mut b = moving_block(1, 0.0, 54.0, 44.0);
    for _ in 0..100 {
        integrate_fall(&mut b, 0.016);
        if b.phase == Phase::Idle {
            break;
        }
    }
    assert_eq!(b.y, 54.0);
    assert_eq!(b.vel, 0.0);
}

#[test]
fn spring_settles_and_overshoots() {
    let mut b = moving_block(1, 0.0, 100.0, 44.0);
    let mut overshoot_seen = false;
    for _ in 0..1000 {
        integrate_spring(&mut b, 0.016);
        if b.y > 100.0 {
            overshoot_seen = true;
        }
        if b.phase == Phase::Idle {
            break;
        }
    }
    assert_eq!(b.phase, Phase::Idle);
    assert_eq!(b.y, 100.0);
    assert!(overshoot_seen, "spring should overshoot");
}

#[test]
fn fall_gate_always_true_when_gap_is_2() {
    let parent = idle_block(1, 0.0, 44.0, None);
    let mut waiting = idle_block(2, 100.0, 44.0, Some(1));
    waiting.phase = Phase::Waiting;
    let all = vec![parent, waiting.clone()];
    assert!(should_release(&waiting, &all, Mode::Fall, 2.0));

    let mut moving_parent = idle_block(1, 0.0, 44.0, None);
    moving_parent.phase = Phase::Moving;
    let all2 = vec![moving_parent, waiting.clone()];
    assert!(should_release(&waiting, &all2, Mode::Fall, 2.0));
}

#[test]
fn ripple_gate_is_delay_zero() {
    let parent = idle_block(1, 0.0, 44.0, None);
    let mut waiting = idle_block(2, 46.0, 44.0, Some(1));
    waiting.phase = Phase::Waiting;
    waiting.delay = 28;
    let all = vec![parent.clone(), waiting.clone()];
    assert!(!should_release(&waiting, &all, Mode::Ripple, 2.0));
    waiting.delay = 0;
    let all2 = vec![parent, waiting.clone()];
    assert!(should_release(&waiting, &all2, Mode::Ripple, 2.0));
}

#[test]
fn spring_gate_always_true() {
    let mut waiting = idle_block(1, 0.0, 44.0, None);
    waiting.phase = Phase::Waiting;
    let all = vec![waiting.clone()];
    assert!(should_release(&waiting, &all, Mode::Spring, 2.0));
}

#[test]
fn inherit_vel_requires_same_direction() {
    let mut parent = moving_block(1, 0.0, 100.0, 44.0);
    parent.vel = 200.0;
    let mut child = moving_block(2, 44.0, 144.0, 44.0);
    child.prev = Some(1);
    let all = vec![parent.clone(), child.clone()];
    assert_eq!(inherit_vel(&child, &all), 200.0);

    parent.target_y = -100.0;
    let all2 = vec![parent, child.clone()];
    assert_eq!(inherit_vel(&child, &all2), 0.0);
}

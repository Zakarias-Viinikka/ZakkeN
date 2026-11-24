use rapier2d_wrapper::core::Core;
use rapier2d_wrapper::types::{Config, Mode, Phase};

fn config(mode: Mode) -> Config {
    Config {
        block_gap: 2.0,
        top_pad: 12.0,
        edge_pad: 24.0,
        page_gap: 24.0,
        min_page_h: 200.0,
        block_w: 300.0,
        mode,
    }
}

fn three_blocks(mode: Mode) -> Core {
    let mut core = Core::new(config(mode));
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![1, 2, 3]], &[]);
    core
}

#[test]
fn step_zero_releases_but_does_not_move() {
    let mut core = three_blocks(Mode::Fall);
    assert_eq!(core.block(1).unwrap().phase, Phase::Waiting);
    let moving = core.step(0.0);
    assert!(moving);
    assert_eq!(core.block(1).unwrap().phase, Phase::Moving);
    assert_eq!(core.block(1).unwrap().y, 0.0);
}

#[test]
fn fall_three_blocks_settles_eventually() {
    let mut core = three_blocks(Mode::Fall);
    let mut moving = true;
    let mut iterations = 0;
    while moving {
        moving = core.step(0.016);
        iterations += 1;
        assert!(iterations < 1000, "never settled");
    }
    for b in core.blocks() {
        assert_eq!(b.phase, Phase::Idle);
        assert_eq!(b.y, b.target_y);
    }
}

#[test]
fn spring_three_blocks_settles_eventually() {
    let mut core = three_blocks(Mode::Spring);
    let mut moving = true;
    let mut iterations = 0;
    while moving {
        moving = core.step(0.016);
        iterations += 1;
        assert!(iterations < 2000, "never settled");
    }
    for b in core.blocks() {
        assert_eq!(b.phase, Phase::Idle);
        assert_eq!(b.y, b.target_y);
    }
}

#[test]
fn ripple_releases_are_staggered() {
    let mut core = three_blocks(Mode::Ripple);
    assert_eq!(core.block(1).unwrap().phase, Phase::Waiting);
    assert_eq!(core.block(2).unwrap().phase, Phase::Waiting);
    assert_eq!(core.block(3).unwrap().phase, Phase::Waiting);
    assert_eq!(core.block(1).unwrap().delay, 0);
    assert_eq!(core.block(2).unwrap().delay, 28);
    assert_eq!(core.block(3).unwrap().delay, 56);

    // Flush: block 1 releases, others stay waiting.
    core.step(0.0);
    assert_eq!(core.block(1).unwrap().phase, Phase::Moving);
    assert_eq!(core.block(2).unwrap().phase, Phase::Waiting);
    assert_eq!(core.block(3).unwrap().phase, Phase::Waiting);

    // 4 steps of 16ms drains both remaining delays.
    for _ in 0..4 {
        core.step(0.016);
    }
    assert_ne!(core.block(2).unwrap().phase, Phase::Waiting);
    assert_ne!(core.block(3).unwrap().phase, Phase::Waiting);
}

#[test]
fn variable_dt_does_not_change_outcome() {
    // Two cores, same setup, different step sizes. Both should reach idle.
    let mut a = three_blocks(Mode::Fall);
    let mut b = three_blocks(Mode::Fall);
    for _ in 0..500 {
        if !a.step(0.016) {
            break;
        }
    }
    for _ in 0..300 {
        if !b.step(1.0 / 60.0) {
            break;
        }
    }
    for id in [1u64, 2, 3] {
        let ba = a.block(id).unwrap();
        let bb = b.block(id).unwrap();
        assert_eq!(ba.phase, Phase::Idle);
        assert_eq!(bb.phase, Phase::Idle);
        assert_eq!(ba.y, ba.target_y);
        assert_eq!(bb.y, bb.target_y);
    }
}

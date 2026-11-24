use rapier2d_wrapper::core::Core;
use rapier2d_wrapper::types::{Config, Mode, Phase};

fn config() -> Config {
    Config {
        block_gap: 2.0,
        top_pad: 12.0,
        edge_pad: 24.0,
        page_gap: 24.0,
        min_page_h: 200.0,
        block_w: 300.0,
        mode: Mode::Fall,
    }
}

fn three_blocks_settled() -> Core {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![1, 2, 3]], &[1, 2, 3]);
    core
}

#[test]
fn tiny_diff_is_no_op() {
    let mut core = three_blocks_settled();
    core.set_height(2, 44.3);
    for b in core.blocks() {
        assert_eq!(b.phase, Phase::Idle);
    }
}

#[test]
fn growing_block_snaps_blocks_below_it() {
    let mut core = three_blocks_settled();
    // Block 1 was 44 tall at y=12. Grow it to 60 (diff 16 >= 0.5).
    core.set_height(1, 60.0);

    let b1 = core.block(1).unwrap();
    let b2 = core.block(2).unwrap();
    let b3 = core.block(3).unwrap();
    // Targets move down by 16 on 2 and 3.
    assert_eq!(b2.target_y, 12.0 + 60.0 + 2.0);
    assert_eq!(b3.target_y, 12.0 + 60.0 + 2.0 + 44.0 + 2.0);
    // Blocks below snapped (they must not animate).
    assert_eq!(b2.phase, Phase::Idle);
    assert_eq!(b2.y, b2.target_y);
    assert_eq!(b3.phase, Phase::Idle);
    assert_eq!(b3.y, b3.target_y);
    // Block 1's own y doesn't move (its top is unchanged).
    assert_eq!(b1.y, 12.0);
    assert_eq!(b1.target_y, 12.0);
}

#[test]
fn shrinking_block_snaps_blocks_below_it() {
    let mut core = three_blocks_settled();
    core.set_height(2, 30.0);
    let b3 = core.block(3).unwrap();
    assert_eq!(b3.phase, Phase::Idle);
    assert_eq!(b3.target_y, 12.0 + 44.0 + 2.0 + 30.0 + 2.0);
}

#[test]
fn set_height_leaves_blocks_above_alone() {
    let mut core = three_blocks_settled();
    core.set_height(2, 60.0);
    let b1 = core.block(1).unwrap();
    assert_eq!(b1.target_y, 12.0);
    assert_eq!(b1.phase, Phase::Idle);
}

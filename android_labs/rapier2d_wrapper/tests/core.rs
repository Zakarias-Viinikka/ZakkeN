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

#[test]
fn three_blocks_commit_sets_targets_and_waits() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![1, 2, 3]], &[]);

    let b1 = core.block(1).unwrap();
    let b2 = core.block(2).unwrap();
    let b3 = core.block(3).unwrap();
    assert_eq!(b1.target_y, 12.0);
    assert_eq!(b2.target_y, 12.0 + 44.0 + 2.0);
    assert_eq!(b3.target_y, 12.0 + 2.0 * (44.0 + 2.0));
    assert_eq!(b1.prev, None);
    assert_eq!(b2.prev, Some(1));
    assert_eq!(b3.prev, Some(2));
    assert_eq!(b1.phase, Phase::Waiting);
    assert_eq!(b2.phase, Phase::Waiting);
    assert_eq!(b3.phase, Phase::Waiting);
    assert_eq!(b1.y, 0.0);
}

#[test]
fn snap_ids_set_blocks_idle_at_target() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.commit_order(&[vec![1, 2]], &[1, 2]);

    let b1 = core.block(1).unwrap();
    let b2 = core.block(2).unwrap();
    assert_eq!(b1.phase, Phase::Idle);
    assert_eq!(b1.y, 12.0);
    assert_eq!(b2.phase, Phase::Idle);
    assert_eq!(b2.y, 58.0);
}

#[test]
fn reorder_moves_only_changed_blocks() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![1, 2, 3]], &[1, 2, 3]);

    // Reorder [2, 1, 3]. Block 3 stays where it is (target unchanged).
    core.commit_order(&[vec![2, 1, 3]], &[]);

    let b1 = core.block(1).unwrap();
    let b2 = core.block(2).unwrap();
    let b3 = core.block(3).unwrap();
    assert_eq!(b2.phase, Phase::Waiting);
    assert_eq!(b1.phase, Phase::Waiting);
    assert_eq!(b3.phase, Phase::Idle, "block 3 should stay settled");
    assert_eq!(b1.prev, Some(2));
    assert_eq!(b2.prev, None);
    assert_eq!(b3.prev, Some(1));
}

#[test]
fn reduced_snaps_everything() {
    let mut core = Core::new(config());
    core.set_reduced(true);
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.commit_order(&[vec![1, 2]], &[]);

    for b in core.blocks() {
        assert_eq!(b.phase, Phase::Idle);
        assert_eq!(b.y, b.target_y);
    }
}

#[test]
fn page_order_returns_blocks_by_target_y() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![3, 1, 2]], &[1, 2, 3]);
    assert_eq!(core.page_order(0), vec![3, 1, 2]);
}

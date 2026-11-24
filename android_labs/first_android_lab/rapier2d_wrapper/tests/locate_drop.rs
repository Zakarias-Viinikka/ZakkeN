use rapier2d_wrapper::core::Core;
use rapier2d_wrapper::types::{Config, Mode};

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
fn picks_the_right_page_by_y_band() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 1, 44.0);
    core.commit_order(&[vec![1, 2], vec![3]], &[1, 2, 3]);

    let rects = core.page_rects();
    let page1_top = rects[1].1;

    // Point inside page 0 (y well above page 1 top)
    let (page, _) = core.locate_drop(0.0, 5.0, &[]).unwrap();
    assert_eq!(page, 0);

    // Point inside page 1
    let (page, _) = core.locate_drop(0.0, page1_top + 1.0, &[]).unwrap();
    assert_eq!(page, 1);

    // Point outside all pages
    assert!(core.locate_drop(0.0, 10000.0, &[]).is_none());
}

#[test]
fn picks_the_closest_block_above() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.add_block(3, 0, 44.0);
    core.commit_order(&[vec![1, 2, 3]], &[1, 2, 3]);

    let b1 = core.block(1).unwrap().clone();
    let b2 = core.block(2).unwrap().clone();
    let b3 = core.block(3).unwrap().clone();

    // Drop just below block 2's center: should insert after block 2.
    let drop_y = b2.target_y + b2.h / 2.0 + 1.0;
    let (_, above) = core.locate_drop(0.0, drop_y, &[]).unwrap();
    assert_eq!(above, Some(2));

    // Drop just below block 1's center (and above block 2's center):
    // should insert after block 1.
    let drop_y = b1.target_y + b1.h / 2.0 + 1.0;
    let (_, above) = core.locate_drop(0.0, drop_y, &[]).unwrap();
    assert_eq!(above, Some(1));

    // Drop above block 1's center: nothing above, insert at top.
    let drop_y = b1.target_y + 1.0;
    let (_, above) = core.locate_drop(0.0, drop_y, &[]).unwrap();
    assert_eq!(above, None);

    // Drop below block 3's center: above = 3.
    let drop_y = b3.target_y + b3.h / 2.0 + 1.0;
    let (_, above) = core.locate_drop(0.0, drop_y, &[]).unwrap();
    assert_eq!(above, Some(3));
}

#[test]
fn exclude_skips_dragged_block() {
    let mut core = Core::new(config());
    core.add_block(1, 0, 44.0);
    core.add_block(2, 0, 44.0);
    core.commit_order(&[vec![1, 2]], &[1, 2]);

    let b2 = core.block(2).unwrap().clone();
    // Drop below block 2's center but exclude it. Above should be 1.
    let drop_y = b2.target_y + b2.h / 2.0 + 1.0;
    let (_, above) = core.locate_drop(0.0, drop_y, &[2]).unwrap();
    assert_eq!(above, Some(1));
}

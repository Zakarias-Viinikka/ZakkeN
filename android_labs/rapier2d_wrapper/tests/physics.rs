use rapier2d_wrapper::layout::page_rects;
use rapier2d_wrapper::types::{Block, Config, Mode, Phase};

fn block(id: u64, page: usize, h: f32) -> Block {
    Block {
        id,
        page,
        y: 0.0,
        target_y: 0.0,
        h,
        vel: 0.0,
        phase: Phase::Idle,
        prev: None,
        delay: 0,
    }
}

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
fn empty_page_hits_min_h() {
    let rects = page_rects(&[], 1, &config());
    assert_eq!(rects, vec![(0, 0.0, 200.0)]);
}

#[test]
fn content_wins_when_above_min_h() {
    let mut c = config();
    c.min_page_h = 100.0;
    let blocks = vec![block(1, 0, 44.0), block(2, 0, 60.0), block(3, 0, 44.0)];
    // content_bottom = 12 + 148 + 2*2 = 164; panel_h = 164 + 24 = 188
    let rects = page_rects(&blocks, 1, &c);
    assert_eq!(rects, vec![(0, 0.0, 188.0)]);
}

#[test]
fn pages_stack_with_gap() {
    let mut c = config();
    c.min_page_h = 100.0;
    let blocks = vec![block(1, 0, 44.0), block(2, 1, 44.0)];
    // Page 0: content_bottom = 12 + 44 = 56; panel_h = max(100, 80) = 100
    // Page 1 top = 0 + 100 + 24 = 124
    let rects = page_rects(&blocks, 2, &c);
    assert_eq!(rects, vec![(0, 0.0, 100.0), (1, 124.0, 100.0)]);
}

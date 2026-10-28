use super::layout::{Block, page_x, BLOCK_W, PAGE_TOP, PAGE_W, UNMEASURED_HEIGHT};
use super::rapier_world::{RapierWorld, PAGE_ID_BASE};

const REGION_H: f32 = 20000.0;

pub fn rebuild_world(rapier: &RapierWorld, blocks: &[Block], page_count: usize) {
    let mut tuples: Vec<(usize, (f32, f32), (f32, f32))> = blocks
        .iter()
        .map(|b| {
            let h = if b.measured { b.h } else { UNMEASURED_HEIGHT };
            (b.internal_id, (b.x, b.target_y), (BLOCK_W, h))
        })
        .collect();
    for page in 0..page_count {
        let x0 = page_x(page) - 16.0;
        let x1 = page_x(page) + PAGE_W + 16.0;
        tuples.push((PAGE_ID_BASE + page, (x0, PAGE_TOP), (x1 - x0, REGION_H)));
    }
    rapier.rebuild(&tuples);
}

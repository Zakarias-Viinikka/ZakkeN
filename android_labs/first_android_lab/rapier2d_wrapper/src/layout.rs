use crate::types::{Block, Config};

/// (page, top_y, height) for each page, top to bottom.
/// `panel_h = max(min_page_h, content_bottom + edge_pad)`, where
/// `content_bottom` is 0 for an empty page, else
/// `top_pad + sum(heights) + (n-1) * block_gap`.
pub fn page_rects(blocks: &[Block], page_count: usize, config: &Config) -> Vec<(usize, f32, f32)> {
    let mut out = Vec::with_capacity(page_count);
    let mut top = 0.0_f32;
    for page in 0..page_count {
        let on_page: Vec<&Block> = blocks.iter().filter(|b| b.page == page).collect();
        let n = on_page.len();
        let content_bottom = if n == 0 {
            0.0
        } else {
            let total_h: f32 = on_page.iter().map(|b| b.h).sum();
            config.top_pad + total_h + (n as f32 - 1.0) * config.block_gap
        };
        let panel_h = (content_bottom + config.edge_pad).max(config.min_page_h);
        out.push((page, top, panel_h));
        top += panel_h + config.page_gap;
    }
    out
}

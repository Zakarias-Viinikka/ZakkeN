use leptos::prelude::*;

use crate::complicated_signal_flow::focus;
use crate::page_state::OpenPages;

pub fn install(open: OpenPages) -> Callback<(String, bool)> {
    Callback::new(move |(block_id, go_down): (String, bool)| {
        let blocks = open.page1.blocks.get_untracked();
        let Some(idx) = blocks.iter().position(|b| b.block_id == block_id) else {
            return;
        };
        if go_down {
            if let Some(next) = blocks.get(idx + 1) {
                focus::at_start(next.block_id.clone());
            }
        } else if idx > 0 {
            focus::at_end(blocks[idx - 1].block_id.clone());
        }
    })
}

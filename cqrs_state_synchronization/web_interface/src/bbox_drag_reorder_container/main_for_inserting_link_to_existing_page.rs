//! Main for inserting a link to an existing page into a block, at a UTF-16 position.
//! Same path as any other edit: diff -> batcher (db/sync), then the row (the editor follows),
//! then ask `CaretOwner` for the caret right after the link.
use leptos::prelude::*;

use crate::complex_state::{batcher_handle, focus};
use crate::leptos_components::small_components::popup::show_popup;
use crate::markdown_conversion::keys::split_utf16;
use crate::page_state::OpenPages;

pub fn main_for_inserting_link_to_existing_page_at_position(
    open: OpenPages,
    block_id: String,
    utf16_position: usize,
    page_id: String,
) {
    if block_id.is_empty() {
        show_popup("link failed: click into a block first".to_string());
        return;
    }
    let Some((boss, page)) = open.page_and_boss_for_block(&block_id) else {
        leptos::logging::error!("link insert: no page/boss for block {}", block_id);
        show_popup(format!("link failed: no open page has block {block_id}"));
        return;
    };
    let Some(old_text) = page.blocks.with_untracked(|rows| {
        rows.iter()
            .find(|b| b.block_id == block_id)
            .map(|b| b.content.clone())
    }) else {
        return;
    };

    let link = make_link(&page_id);
    let (before, after) = split_utf16(&old_text, utf16_position, utf16_position);
    let caret_after_link = before.encode_utf16().count() + link.encode_utf16().count();
    let new_text = format!("{before}{link}{after}");

    let diff = text_diff::diff_logic::get_diff(&old_text, &new_text);
    batcher_handle::push_edit(boss, block_id.clone(), diff);
    page.blocks.update(|rows| {
        if let Some(b) = rows.iter_mut().find(|b| b.block_id == block_id) {
            b.content = new_text;
        }
    });
    focus::at_pos(block_id, caret_after_link);
}

/// `[[PageNav|page_id]]`. A click looks up the text before the `|` in COMMANDS
/// (markdown_conversion/main_for_on_link_click/core.rs), and PageNav is the command that
/// opens a page.
fn make_link(page_id: &str) -> String {
    format!("[[PageNav|{page_id}]]")
}

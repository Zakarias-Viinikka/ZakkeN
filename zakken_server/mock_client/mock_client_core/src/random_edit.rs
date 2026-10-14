use love_letter::LoveLetterSketch;
use my_yrs_lib::{Block, BossOfYrs, EditTarget, TextEdit};
use rand::RngExt;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq)]
pub enum EditKind {
    Random,
    Insert,
    Delete,
    Replace,
}

pub struct EditOutcome {
    pub sketch: LoveLetterSketch,
    pub description: String,
}

pub fn perform_edit(boss: &Arc<BossOfYrs>, kind: EditKind) -> Result<EditOutcome, String> {
    let blocks = boss
        .clone()
        .get_entire_page()
        .map_err(|e| format!("get_entire_page: {e}"))?;
    if blocks.is_empty() {
        return Err("page has no blocks".to_string());
    }

    let mut rng = rand::rng();

    let mut action = match kind {
        EditKind::Random => rng.random_range(0..3u8),
        EditKind::Insert => 0,
        EditKind::Delete => 1,
        EditKind::Replace => 2,
    };

    // Insert works on any block. Delete and replace need text to work with, so
    // they can only be picked when at least one block has some.
    if action != 0 && blocks.iter().all(|b| b.text.is_empty()) {
        if kind == EditKind::Random {
            action = 0;
        } else {
            return Err(match action {
                1 => "no block has text to delete".to_string(),
                _ => "no block has text to replace".to_string(),
            });
        }
    }

    let pool: Vec<&Block> = if action == 0 {
        blocks.iter().collect()
    } else {
        blocks.iter().filter(|b| !b.text.is_empty()).collect()
    };

    let idx = rng.random_range(0..pool.len());
    let block = pool[idx];
    let chars: Vec<char> = block.text.chars().collect();
    let block_id = block.id_in_yrs.clone();
    let page_id = boss.page_id.clone();

    let (text_edit, description) = match action {
        0 => {
            let pos = if chars.is_empty() {
                0
            } else {
                rng.random_range(0..=chars.len()) as u32
            };
            let new_text = "mock_insert".to_string();
            let desc = format!("insert {:?} at {} in block {}", new_text, pos, block_id);
            (
                TextEdit::Insert {
                    text: new_text,
                    position: pos,
                },
                desc,
            )
        }
        1 => {
            let pos = rng.random_range(0..chars.len()) as u32;
            let max_len = chars.len() - pos as usize;
            let len = rng.random_range(1..=max_len);
            let removed: String = chars[pos as usize..pos as usize + len].iter().collect();
            let desc = format!("delete {:?} at {} in block {}", removed, pos, block_id);
            (
                TextEdit::Delete {
                    text: removed,
                    position: pos,
                },
                desc,
            )
        }
        _ => {
            let pos = rng.random_range(0..chars.len()) as u32;
            let max_len = chars.len() - pos as usize;
            let len = rng.random_range(1..=max_len);
            let old: String = chars[pos as usize..pos as usize + len].iter().collect();
            let new_text = "mock_replace".to_string();
            let desc = format!(
                "replace {:?} with {:?} at {} in block {}",
                old, new_text, pos, block_id
            );
            (
                TextEdit::Replace {
                    old_text: old,
                    new_text,
                    position: pos,
                },
                desc,
            )
        }
    };

    boss.clone()
        .edit_text_block(block_id.clone(), text_edit.clone(), EditTarget::Text)
        .map_err(|e| format!("edit_text_block: {e}"))?;

    let sketch = LoveLetterSketch::EditBlock {
        text_edit,
        edit_target: EditTarget::Text,
        target_page_id: page_id,
        block_id,
    };

    Ok(EditOutcome { sketch, description })
}
use serde::{Deserialize, Serialize};

/// Change record emitted by CodeMirror's `update.changes.iterChanges`.
/// `from` and `to` are UTF-16 code-unit offsets in the pre-edit (old) document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmChange {
    pub from: u32,
    pub to: u32,
    pub insert: String,
}

/// Same enum shape as `yrs/src/yrs_wrapper/types.rs` so the spike proves
/// compatibility with the real project's edit pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextEdit {
    Insert {
        text: String,
        position: u32,
    },
    Delete {
        text: String,
        position: u32,
    },
    Replace {
        old_text: String,
        new_text: String,
        position: u32,
    },
}

/// Converts CM changes (in old-doc UTF-16 coordinates) into `TextEdit`s sorted in
/// descending `position` order. Applying in descending order keeps every earlier
/// old-doc offset valid without shifting indices.
pub fn convert_changes_to_edits_desc(
    old_authority: &str,
    changes: &[CmChange],
) -> Result<Vec<TextEdit>, String> {
    let mut edits = Vec::with_capacity(changes.len());
    for ch in changes {
        if ch.from > ch.to {
            return Err(format!("invalid change range {}..{}", ch.from, ch.to));
        }
        let is_insert = ch.from == ch.to;
        let is_delete = ch.insert.is_empty();

        let edit = if is_insert && is_delete {
            continue;
        } else if is_insert {
            TextEdit::Insert {
                text: ch.insert.clone(),
                position: ch.from,
            }
        } else {
            let old_slice = slice_by_utf16_range(old_authority, ch.from, ch.to)?;
            if is_delete {
                TextEdit::Delete {
                    text: old_slice.to_string(),
                    position: ch.from,
                }
            } else {
                TextEdit::Replace {
                    old_text: old_slice.to_string(),
                    new_text: ch.insert.clone(),
                    position: ch.from,
                }
            }
        };
        edits.push(edit);
    }

    // Sort descending by old-doc position. Non-overlapping changes have unique positions.
    edits.sort_by(|a, b| edit_position(b).cmp(&edit_position(a)));
    Ok(edits)
}

pub fn edit_position(edit: &TextEdit) -> u32 {
    match edit {
        TextEdit::Insert { position, .. }
        | TextEdit::Delete { position, .. }
        | TextEdit::Replace { position, .. } => *position,
    }
}

pub fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// Maps a UTF-16 code-unit offset in `s` to a UTF-8 byte offset.
/// Returns Err if out of bounds or splitting a UTF-16 surrogate pair.
pub fn utf16_to_byte_offset(s: &str, target_utf16: u32) -> Result<usize, String> {
    let mut u16_pos: u32 = 0;
    for (byte_idx, ch) in s.char_indices() {
        if u16_pos == target_utf16 {
            return Ok(byte_idx);
        }
        let next_u16 = u16_pos + ch.len_utf16() as u32;
        if target_utf16 < next_u16 {
            return Err(format!(
                "utf16 offset {} splits surrogate pair of char {:?} at byte {}",
                target_utf16, ch, byte_idx
            ));
        }
        u16_pos = next_u16;
    }
    if u16_pos == target_utf16 {
        Ok(s.len())
    } else {
        Err(format!(
            "utf16 offset {} out of bounds (doc utf16 len is {})",
            target_utf16, u16_pos
        ))
    }
}

pub fn slice_by_utf16_range(s: &str, from_utf16: u32, to_utf16: u32) -> Result<&str, String> {
    let start = utf16_to_byte_offset(s, from_utf16)?;
    let end = utf16_to_byte_offset(s, to_utf16)?;
    Ok(&s[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_and_cjk_single_edits() {
        let old = "a😀日b";
        // UTF-16 offsets: a=0..1, 😀=1..3, 日=3..4, b=4..5
        let edits = convert_changes_to_edits_desc(
            old,
            &[
                CmChange {
                    from: 1,
                    to: 3,
                    insert: "🚀".to_string(),
                },
                CmChange {
                    from: 4,
                    to: 5,
                    insert: "".to_string(),
                },
            ],
        )
        .unwrap();

        assert_eq!(
            edits,
            vec![
                TextEdit::Delete {
                    text: "b".to_string(),
                    position: 4,
                },
                TextEdit::Replace {
                    old_text: "😀".to_string(),
                    new_text: "🚀".to_string(),
                    position: 1,
                },
            ]
        );
    }

    #[test]
    fn multi_change_sorts_descending() {
        let old = "hello world";
        let edits = convert_changes_to_edits_desc(
            old,
            &[
                CmChange {
                    from: 0,
                    to: 0,
                    insert: "!".to_string(),
                },
                CmChange {
                    from: 6,
                    to: 11,
                    insert: "rust".to_string(),
                },
            ],
        )
        .unwrap();
        assert_eq!(edit_position(&edits[0]), 6);
        assert_eq!(edit_position(&edits[1]), 0);
    }

    #[test]
    fn rejects_surrogate_split() {
        let old = "😀";
        let err = slice_by_utf16_range(old, 1, 2).unwrap_err();
        assert!(err.contains("surrogate"));
    }
}

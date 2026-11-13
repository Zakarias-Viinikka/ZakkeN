//! Caller pov: turn the user's final block list for one page into love
//! letter sketches, given the server's copy of that page. Blocks are
//! keyed by `block_id`. A server block that's gone from the user's list,
//! or whose user-side text is empty, becomes `RemoveBlock`. An edited
//! block becomes `EditBlock` with a full-replace `TextEdit::Replace`. A
//! moved block (same id and text, different `frac_position`) becomes
//! `MoveBlock`. A new block becomes `CreateNewBlockWithId`, anchored
//! between the nearest surviving neighbours. Unchanged blocks produce
//! nothing.
//!
//! Out of scope on purpose: duplicate ids (a hard error), and any
//! snapshot work (caller builds snapshots from these sketches via
//! `build_a_love_letter`). `frac_position` and `metadata` are opaque
//! strings the caller provides; only `frac_position` is compared.

use std::collections::HashSet;

pub use love_letter::LoveLetterSketch;
use my_yrs_lib::{EditTarget, TextEdit, yrs_wrapper::PositionToInsert};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Block {
    pub block_id: String,
    pub text: String,
    pub metadata: String,
    pub frac_position: String,
}

#[derive(Debug)]
pub enum GenErr {
    DuplicateBlockId(String),
}

impl std::fmt::Display for GenErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenErr::DuplicateBlockId(id) => write!(f, "duplicate block id: {id}"),
        }
    }
}

impl std::error::Error for GenErr {}

pub fn generate_sketches(
    target_page_id: &str,
    server: &[Block],
    user: &[Block],
) -> Result<Vec<LoveLetterSketch>, GenErr> {
    check_unique(server)?;
    check_unique(user)?;

    let server_ids: HashSet<&str> = server.iter().map(|b| b.block_id.as_str()).collect();


    let mut sketches = Vec::new();

    for (b_index, b) in server.iter().enumerate() {
        let is_title = b_index == 0;
        match user.iter().find(|x| x.block_id == b.block_id) {
            None => {
                if !is_title {
                    sketches.push(LoveLetterSketch::RemoveBlock {
                        block_id: b.block_id.clone(),
                        target_page_id: target_page_id.to_string(),
                    });
                }
            }
            Some(u) if u.text.is_empty() && !b.text.is_empty() && !is_title => {
                sketches.push(LoveLetterSketch::RemoveBlock {
                    block_id: b.block_id.clone(),
                    target_page_id: target_page_id.to_string(),
                });
            }
            Some(u) => {
                if u.text != b.text {
                    if let Some(text_edit) = text_edit_for_block_change(&b.text, &u.text) {
                        sketches.push(LoveLetterSketch::EditBlock {
                            text_edit,
                            edit_target: EditTarget::Text,
                            target_page_id: target_page_id.to_string(),
                            block_id: b.block_id.clone(),
                        });
                    }
                }
                if u.frac_position != b.frac_position {
                    sketches.push(LoveLetterSketch::MoveBlock {
                        block_id: b.block_id.clone(),
                        target_page_id: target_page_id.to_string(),
                        new_frac_position: u.frac_position.clone(),
                    });
                }
            }
        }
    }

    for (i, u) in user.iter().enumerate() {
        if server_ids.contains(u.block_id.as_str()) {
            continue;
        }
        if u.text.is_empty() {
            continue;
        }
        let above = user[..i]
            .iter()
            .rev()
            .find(|x| server_ids.contains(x.block_id.as_str()))
            .map(|x| x.block_id.clone());
        let below = user[i + 1..]
            .iter()
            .find(|x| server_ids.contains(x.block_id.as_str()))
            .map(|x| x.block_id.clone());
        let position_to_insert = if above.is_none() && below.is_none() {
            PositionToInsert::AtEnd
        } else {
            PositionToInsert::Between {
                block_id_of_above: above,
                block_id_of_below: below,
            }
        };
        sketches.push(LoveLetterSketch::CreateNewBlockWithId {
            block_id: u.block_id.clone(),
            position_to_insert,
            target_page_id: target_page_id.to_string(),
        });
    }

    Ok(sketches)
}

/// Longest-common-prefix-then-suffix, then whatever's left in the middle
/// is the actual change. Char-based, so emoji and non-ASCII are handled
/// as single characters, matching `TextEdit`'s convention.
///
/// Not `text_diff::get_diff`. `get_diff` returns a whole-string insert
/// for different-length replacements ("cat" -> "dogs" is an Insert of
/// "dogs", which corrupts the block). This helper produces a Replace on
/// just the differing middle.
fn text_edit_for_block_change(old: &str, new: &str) -> Option<TextEdit> {
    if old == new {
        return None;
    }
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();
    let min_len = old_chars.len().min(new_chars.len());

    let mut prefix = 0usize;
    while prefix < min_len && old_chars[prefix] == new_chars[prefix] {
        prefix += 1;
    }

    let mut suffix = 0usize;
    let max_suffix = min_len - prefix;
    while suffix < max_suffix
        && old_chars[old_chars.len() - 1 - suffix] == new_chars[new_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let old_mid: String = old_chars[prefix..old_chars.len() - suffix].iter().collect();
    let new_mid: String = new_chars[prefix..new_chars.len() - suffix].iter().collect();
    let position = prefix as u32;

    if old_mid.is_empty() && new_mid.is_empty() {
        None
    } else if old_mid.is_empty() {
        Some(TextEdit::Insert {
            text: new_mid,
            position,
        })
    } else if new_mid.is_empty() {
        Some(TextEdit::Delete {
            text: old_mid,
            position,
        })
    } else {
        Some(TextEdit::Replace {
            old_text: old_mid,
            new_text: new_mid,
            position,
        })
    }
}

fn check_unique(blocks: &[Block]) -> Result<(), GenErr> {
    let mut seen = HashSet::new();
    for b in blocks {
        if !seen.insert(b.block_id.as_str()) {
            return Err(GenErr::DuplicateBlockId(b.block_id.clone()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(old: &str, new: &str) -> Option<(String, String, u32, &'static str)> {
        match text_edit_for_block_change(old, new)? {
            TextEdit::Insert { text, position } => Some((String::new(), text, position, "insert")),
            TextEdit::Delete { text, position } => Some((text, String::new(), position, "delete")),
            TextEdit::Replace {
                old_text,
                new_text,
                position,
            } => Some((old_text, new_text, position, "replace")),
        }
    }

    #[test]
    fn identical_returns_none() {
        assert!(text_edit_for_block_change("abc", "abc").is_none());
    }

    #[test]
    fn cat_to_dogs() {
        let (old, new, pos, kind) = edit("cat", "dogs").unwrap();
        assert_eq!(kind, "replace");
        assert_eq!(old, "cat");
        assert_eq!(new, "dogs");
        assert_eq!(pos, 0);
    }

    #[test]
    fn abc_x_def_to_abc_yy_def() {
        let (old, new, pos, kind) = edit("abcXdef", "abcYYdef").unwrap();
        assert_eq!(kind, "replace");
        assert_eq!(old, "X");
        assert_eq!(new, "YY");
        assert_eq!(pos, 3);
    }

    #[test]
    fn aa_to_aaa() {
        let (old, new, pos, kind) = edit("aa", "aaa").unwrap();
        assert_eq!(kind, "insert");
        assert_eq!(old, "");
        assert_eq!(new, "a");
        assert_eq!(pos, 2);
    }

    #[test]
    fn abcabc_to_abc() {
        let (old, new, pos, kind) = edit("abcabc", "abc").unwrap();
        assert_eq!(kind, "delete");
        assert_eq!(old, "abc");
        assert_eq!(new, "");
        assert_eq!(pos, 3);
    }

    #[test]
    fn trailing_newlines() {
        let (old, new, pos, kind) = edit("a\n\n", "a\n\n\n").unwrap();
        assert_eq!(kind, "insert");
        assert_eq!(old, "");
        assert_eq!(new, "\n");
        assert_eq!(pos, 3);
    }

    #[test]
    fn emoji() {
        let (old, new, pos, kind) = edit("hi 😀 there", "hi 😎 there").unwrap();
        assert_eq!(kind, "replace");
        assert_eq!(old, "😀");
        assert_eq!(new, "😎");
        assert_eq!(pos, 3);
    }

    #[test]
    fn empty_to_text() {
        let (old, new, pos, kind) = edit("", "abc").unwrap();
        assert_eq!(kind, "insert");
        assert_eq!(old, "");
        assert_eq!(new, "abc");
        assert_eq!(pos, 0);
    }

    #[test]
    fn text_to_empty() {
        let (old, new, pos, kind) = edit("abc", "").unwrap();
        assert_eq!(kind, "delete");
        assert_eq!(old, "abc");
        assert_eq!(new, "");
        assert_eq!(pos, 0);
    }

    #[test]
    fn identical_blank_is_no_edit() {
        assert!(text_edit_for_block_change("", "").is_none());
    }

    fn b(id: &str, text: &str, frac: &str) -> Block {
        Block {
            block_id: id.to_string(),
            text: text.to_string(),
            metadata: String::new(),
            frac_position: frac.to_string(),
        }
    }

    #[test]
    fn blanked_title_is_an_edit_not_a_remove() {
        let server = vec![b("title", "My Title", "80"), b("body", "hello", "81")];
        let user = vec![b("title", "", "80"), b("body", "hello", "81")];
        let sketches = generate_sketches("p", &server, &user).unwrap();
        let removes = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::RemoveBlock { .. }))
            .count();
        let edits = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::EditBlock { .. }))
            .count();
        assert_eq!(removes, 0, "title must never be removed");
        assert_eq!(edits, 1, "title blanking should become an edit");
    }

    #[test]
    fn blanked_non_title_block_still_removes() {
        let server = vec![b("title", "My Title", "80"), b("body", "hello", "81")];
        let user = vec![b("title", "My Title", "80"), b("body", "", "81")];
        let sketches = generate_sketches("p", &server, &user).unwrap();
        let removes = sketches
            .iter()
            .filter(|s| matches!(s, LoveLetterSketch::RemoveBlock { .. }))
            .count();
        assert_eq!(removes, 1);
    }
}

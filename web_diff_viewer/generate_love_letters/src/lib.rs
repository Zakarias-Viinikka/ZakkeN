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

#[derive(Debug, Clone)]
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

    for b in server {
        match user.iter().find(|x| x.block_id == b.block_id) {
            None => {
                sketches.push(LoveLetterSketch::RemoveBlock {
                    block_id: b.block_id.clone(),
                    target_page_id: target_page_id.to_string(),
                });
            }
            Some(u) if u.text.is_empty() => {
                sketches.push(LoveLetterSketch::RemoveBlock {
                    block_id: b.block_id.clone(),
                    target_page_id: target_page_id.to_string(),
                });
            }
            Some(u) => {
                if u.text != b.text {
                    sketches.push(LoveLetterSketch::EditBlock {
                        text_edit: TextEdit::Replace {
                            old_text: b.text.clone(),
                            new_text: u.text.clone(),
                            position: 0,
                        },
                        edit_target: EditTarget::Text,
                        target_page_id: target_page_id.to_string(),
                        block_id: b.block_id.clone(),
                    });
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

fn check_unique(blocks: &[Block]) -> Result<(), GenErr> {
    let mut seen = HashSet::new();
    for b in blocks {
        if !seen.insert(b.block_id.as_str()) {
            return Err(GenErr::DuplicateBlockId(b.block_id.clone()));
        }
    }
    Ok(())
}
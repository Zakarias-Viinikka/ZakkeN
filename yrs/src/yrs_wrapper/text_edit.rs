use yrs::{Doc, In, Map, MapRef, Text, Transact, XmlTextRef};

use super::keys::{CONTENT_KEY, META_KEY};
use super::types::{EditTarget, TextEdit};
use crate::yrs_error::YrsError;

pub(crate) fn apply_edit(
    doc: &Doc,
    block: MapRef,
    edit: TextEdit,
    edit_target: EditTarget,
) -> Result<(), YrsError> {
    match edit_target {
        EditTarget::Text => edit_block_text(doc, block, edit),
        EditTarget::Meta => edit_block_meta(doc, block, edit),
    }
}

fn edit_block_text(doc: &Doc, block: MapRef, edit: TextEdit) -> Result<(), YrsError> {
    let mut txn = doc.transact_mut();
    let text_ref = block
        .get(&txn, CONTENT_KEY)
        .ok_or_else(|| crate::generic_error!("edit_block: no text field", "edit_block_text"))?
        .cast::<XmlTextRef>()
        .map_err(|_| {
            crate::generic_error!(
                "edit_block: failed to cast text field to XmlTextRef",
                "edit_block_text"
            )
        })?;

    match edit {
        TextEdit::Insert { text, position } => text_ref.insert(&mut txn, position, &text),
        TextEdit::Delete { text, position } => {
            text_ref.remove_range(&mut txn, position, text.chars().count() as u32)
        }
        TextEdit::Replace {
            old_text,
            new_text,
            position,
        } => {
            text_ref.remove_range(&mut txn, position, old_text.chars().count() as u32);
            text_ref.insert(&mut txn, position, &new_text)
        }
    }
    Ok(())
}

fn edit_block_meta(doc: &Doc, block: MapRef, edit: TextEdit) -> Result<(), YrsError> {
    let mut txn = doc.transact_mut();
    let current_meta = block
        .get(&txn, META_KEY)
        .ok_or_else(|| crate::generic_error!("edit_block: no meta field", "edit_block_meta"))?
        .cast::<String>()
        .map_err(|_| {
            crate::generic_error!(
                "edit_block: failed to cast meta field to String",
                "edit_block_meta"
            )
        })?;
    let new_meta = apply_edit_to_string(current_meta, edit)?;
    block.insert(&mut txn, META_KEY.to_string(), In::from(new_meta));
    Ok(())
}

fn apply_edit_to_string(old_meta: String, edit: TextEdit) -> Result<String, YrsError> {
    let mut chars: Vec<char> = old_meta.chars().collect();

    match edit {
        TextEdit::Insert { text, position } => {
            let pos = position as usize;
            if pos > chars.len() {
                return Err(crate::generic_error!(
                    "insert position exceeds string length",
                    "apply_edit_to_string"
                ));
            }
            chars.splice(pos..pos, text.chars());
        }
        TextEdit::Delete { text, position } => {
            let pos = position as usize;
            let delete_len = text.chars().count();
            if pos + delete_len > chars.len() {
                return Err(crate::generic_error!(
                    "delete range exceeds string length",
                    "apply_edit_to_string"
                ));
            }
            chars.drain(pos..(pos + delete_len));
        }
        TextEdit::Replace {
            old_text,
            new_text,
            position,
        } => {
            let pos = position as usize;
            let old_len = old_text.chars().count();
            if pos + old_len > chars.len() {
                return Err(crate::generic_error!(
                    "replace range exceeds string length",
                    "apply_edit_to_string"
                ));
            }
            chars.splice(pos..(pos + old_len), new_text.chars());
        }
    }

    Ok(chars.into_iter().collect())
}

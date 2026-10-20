#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use yrs::block::Item;
use yrs::types::ToJson;
use yrs::types::TypeRef::XmlText;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Array, ArrayRef, Doc, GetString, In, Map, MapPrelim, MapRef, ReadTxn, Text, TextRef,
    Transact, Update, XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef,
};
use yrs::{StateVector, Transaction};

use crate::anti_deadlock::{DeadlockCtx, DurationSettings, prevent_deadlock};
use smart_frac_index::frac_core as smart_frac_index;
use crate::yrs_error::{DeadlockPrediction, ErrorInfo, YrsError};

fn error_info(error_msg: impl Into<String>, method: &str) -> ErrorInfo {
    ErrorInfo {
        error_msg: error_msg.into(),
        file: file!().to_string(),
        method: method.to_string(),
    }
}

fn yrs_error(error_msg: impl Into<String>, method: &str) -> YrsError {
    YrsError::YrsInternalError {
        info: error_info(error_msg, method),
    }
}

#[derive(uniffi::Object)]
pub struct BossOfYrs {
    pub doc: RwLock<Doc>,
    pub page_id: String,
    pub user_id: String,
    pub time: String,
}

#[derive(uniffi::Record)]
pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}

#[derive(uniffi::Enum, Serialize, Deserialize, Clone)]
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

#[derive(uniffi::Enum, Serialize, Deserialize)]
pub enum EditTarget {
    Text,
    Meta,
}

const BLOCKS_KEY: &str = "blocks";
const CONTENT_KEY: &str = "text";
const META_KEY: &str = "meta";
const ID_KEY: &str = "id";

fn edit_block(
    doc: &Doc,
    block: MapRef,
    edit: TextEdit,
    edit_target: EditTarget,
) -> Result<(), YrsError> {
    let mut txn = doc.transact_mut();

    match edit_target {
        EditTarget::Text => {
            let text_ref = block
                .get(&txn, &CONTENT_KEY)
                .ok_or_else(|| YrsError::GenericError {
                    info: error_info("edit_block: no text field", "edit_block"),
                })?
                .cast::<XmlTextRef>()
                .map_err(|_| YrsError::GenericError {
                    info: error_info(
                        "edit_block: failed to cast text field to XmlTextRef",
                        "edit_block",
                    ),
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
        EditTarget::Meta => {
            let current_meta = block
                .get(&txn, &META_KEY)
                .ok_or_else(|| YrsError::GenericError {
                    info: error_info("edit_block: no meta field", "edit_block"),
                })?
                .cast::<String>()
                .map_err(|_| YrsError::GenericError {
                    info: error_info(
                        "edit_block: failed to cast meta field to String",
                        "edit_block",
                    ),
                })?;

            let new_meta = apply_edit_to_string(current_meta, edit)?;
            block.insert(&mut txn, META_KEY.to_string(), In::from(new_meta));
            Ok(())
        }
    }
}

fn apply_edit_to_string(old_meta: String, edit: TextEdit) -> Result<String, YrsError> {
    let mut chars: Vec<char> = old_meta.chars().collect();

    match edit {
        TextEdit::Insert { text, position } => {
            let pos = position as usize;
            if pos > chars.len() {
                return Err(YrsError::GenericError {
                    info: error_info(
                        "insert position exceeds string length",
                        "apply_edit_to_string",
                    ),
                });
            }
            chars.splice(pos..pos, text.chars());
        }
        TextEdit::Delete { text, position } => {
            let pos = position as usize;
            let delete_len = text.chars().count();
            if pos + delete_len > chars.len() {
                return Err(YrsError::GenericError {
                    info: error_info("delete range exceeds string length", "apply_edit_to_string"),
                });
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
                return Err(YrsError::GenericError {
                    info: error_info(
                        "replace range exceeds string length",
                        "apply_edit_to_string",
                    ),
                });
            }
            chars.splice(pos..(pos + old_len), new_text.chars());
        }
    }

    Ok(chars.into_iter().collect())
}

// ===== Helpers (defined after impl, but usable because module order doesn't matter) =====
fn get_block_index_from_id(doc: &Doc, block_id: &str) -> Option<u32> {
    let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
    let txn = doc.transact();
    for (position_by_regular_counting, block) in array.iter(&txn).enumerate() {
        if let Ok(block_map) = block.cast::<MapRef>() {
            if let Some(id_val) = block_map.get(&txn, ID_KEY) {
                if let Ok(id_str) = id_val.cast::<String>() {
                    if id_str == block_id {
                        return Some(position_by_regular_counting as u32);
                    }
                }
            }
        }
    }
    None
}

fn get_block_from_id(doc: &Doc, block_id: &str) -> Option<MapRef> {
    let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
    let txn = doc.transact();
    for block in array.iter(&txn) {
        if let Ok(block_map) = block.cast::<MapRef>() {
            if let Some(id_val) = block_map.get(&txn, ID_KEY) {
                if let Ok(id_str) = id_val.cast::<String>() {
                    if id_str == block_id {
                        return Some(block_map);
                    }
                }
            }
        }
    }
    None
}

const FRAC_POSITION_KEY: &str = "frac_position";

fn get_frac_position_of_block_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position_by_regular_counting: usize,
) -> Result<String, YrsError> {
    let block_map = get_block_map_at(array, txn, position_by_regular_counting)?;
    block_map
        .get(txn, FRAC_POSITION_KEY)
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("block at position {position_by_regular_counting} missing {FRAC_POSITION_KEY}"),
                "get_frac_position_of_block_at",
            ),
        })?
        .cast::<String>()
        .map_err(|_| YrsError::GenericError {
            info: error_info(
                format!(
                    "block at position {position_by_regular_counting} {FRAC_POSITION_KEY} is not a string"
                ),
                "get_frac_position_of_block_at",
            ),
        })
}

fn get_block_map_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position_by_regular_counting: usize,
) -> Result<MapRef, YrsError> {
    let block = array
        .iter(txn)
        .nth(position_by_regular_counting)
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("no block at position {position_by_regular_counting}"),
                "get_block_map_at",
            ),
        })?;
    block.cast::<MapRef>().map_err(|_| YrsError::GenericError {
        info: error_info(
            format!("block at position {position_by_regular_counting} is not a map"),
            "get_block_map_at",
        ),
    })
}

fn read_block_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position_by_regular_counting: usize,
) -> Result<(String, Block), YrsError> {
    let block_map = get_block_map_at(array, txn, position_by_regular_counting)?;
    let text = block_map
        .get(txn, CONTENT_KEY)
        .and_then(|v| v.cast::<XmlTextRef>().ok())
        .map(|t| t.get_string(txn))
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!(
                    "block at position {position_by_regular_counting} content is not XmlTextRef"
                ),
                "read_block_at",
            ),
        })?;
    let metadata = block_map
        .get(txn, META_KEY)
        .and_then(|v| v.cast::<String>().ok())
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("block at position {position_by_regular_counting} meta is not a string"),
                "read_block_at",
            ),
        })?;
    let id_in_yrs = block_map
        .get(txn, ID_KEY)
        .and_then(|v| v.cast::<String>().ok())
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("block at position {position_by_regular_counting} id is not a string"),
                "read_block_at",
            ),
        })?;
    let frac_position = block_map
        .get(txn, FRAC_POSITION_KEY)
        .and_then(|v| v.cast::<String>().ok())
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("block at position {position_by_regular_counting} missing {FRAC_POSITION_KEY}"),
                "read_block_at",
            ),
        })?;
    Ok((
        frac_position,
        Block {
            text,
            metadata,
            id_in_yrs,
        },
    ))
}

fn find_block_with_id<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id: &str,
) -> Result<MapRef, YrsError> {
    for block in array.iter(txn) {
        if let Ok(block_map) = block.cast::<MapRef>() {
            if let Some(id_val) = block_map.get(txn, ID_KEY) {
                if let Ok(id_str) = id_val.cast::<String>() {
                    if id_str == block_id {
                        return Ok(block_map);
                    }
                }
            }
        }
    }
    Err(YrsError::GenericError {
        info: error_info(format!("no block with id {block_id}"), "find_block_with_id"),
    })
}

fn get_frac_position_by_id<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id: &str,
) -> Result<String, YrsError> {
    let block_map = find_block_with_id(array, txn, block_id)?;
    block_map
        .get(txn, FRAC_POSITION_KEY)
        .and_then(|v| v.cast::<String>().ok())
        .ok_or_else(|| YrsError::GenericError {
            info: error_info(
                format!("block {block_id} missing {FRAC_POSITION_KEY}"),
                "get_frac_position_by_id",
            ),
        })
}

fn compute_new_frac_position_in<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id_of_above: Option<&str>,
    block_id_of_below: Option<&str>,
) -> Result<String, YrsError> {
    match (block_id_of_above, block_id_of_below) {
        (Some(above), Some(below)) => {
            let above_frac = get_frac_position_by_id(array, txn, above)?;
            let below_frac = get_frac_position_by_id(array, txn, below)?;
            Ok(smart_frac_index::key_between(&above_frac, &below_frac)?)
        }
        (Some(above), None) => {
            let above_frac = get_frac_position_by_id(array, txn, above)?;
            Ok(smart_frac_index::key_after(&above_frac)?)
        }
        (None, Some(below)) => {
            let below_frac = get_frac_position_by_id(array, txn, below)?;
            Ok(smart_frac_index::key_before(&below_frac)?)
        }
        (None, None) => Err(YrsError::GenericError {
            info: error_info(
                "at least one neighbour is required",
                "compute_new_frac_position_in",
            ),
        }),
    }
}

fn get_new_frac_position<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    insert_at: u32,
    len: u32,
) -> Result<String, YrsError> {
    if len == 0 {
        return Ok(smart_frac_index::key_at_start());
    }
    if insert_at == 0 {
        let first = get_frac_position_of_block_at(array, txn, 0)?;
        return Ok(smart_frac_index::key_before(&first)?);
    }
    if insert_at == len {
        let last = get_frac_position_of_block_at(array, txn, (len - 1) as usize)?;
        return Ok(smart_frac_index::key_after(&last)?);
    }
    let prev = get_frac_position_of_block_at(array, txn, (insert_at - 1) as usize)?;
    let next = get_frac_position_of_block_at(array, txn, insert_at as usize)?;
    Ok(smart_frac_index::key_between(&prev, &next)?)
}

use std::sync::atomic::{AtomicU64, Ordering};

fn insert_block_into_doc(
    doc: &Doc,
    block_id: String,
    block_content: String,
    block_meta_data: String,
    position_to_insert: PositionToInsert,
) -> Result<(), YrsError> {
    let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
    let mut txn = doc.transact_mut();
    let len = array.len(&txn) as u32;
    let frac_position = match &position_to_insert {
        PositionToInsert::AtEnd => get_new_frac_position(&array, &txn, len, len)?,
        PositionToInsert::Between {
            block_id_of_above,
            block_id_of_below,
        } => compute_new_frac_position_in(
            &array,
            &txn,
            block_id_of_above.as_deref(),
            block_id_of_below.as_deref(),
        )?,
    };
    let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
    array.insert(
        &mut txn,
        len,
        MapPrelim::from([
            (ID_KEY.to_string(), In::from(block_id)),
            (CONTENT_KEY.to_string(), In::from(text_as_xml_text_ref)),
            (META_KEY.to_string(), In::from(block_meta_data)),
            (FRAC_POSITION_KEY.to_string(), In::from(frac_position)),
        ]),
    );
    Ok(())
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[uniffi::export]
impl BossOfYrs {
    #[uniffi::constructor]
    pub fn new(user_id: String, unix_time: String) -> Self {
        let unique_id = generate_unique_key(&user_id, &unix_time);
        Self {
            doc: RwLock::new(Doc::new()),
            page_id: unique_id,
            user_id,
            time: unix_time,
        }
    }

    pub fn page_id(self: Arc<Self>) -> String {
        self.page_id.clone()
    }

    pub fn user_id(self: Arc<Self>) -> String {
        self.user_id.clone()
    }

    pub fn insert_new_block(
        self: Arc<Self>,
        block_content: String,
        block_meta_data: String,
        position: PositionToInsert,
    ) -> Result<String, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "insert_new_block",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "insert_new_block"),
                })?;
                let block_id = generate_unique_key(&self.user_id, &self.time);
                insert_block_into_doc(&doc, block_id.clone(), block_content, block_meta_data, position)?;
                Ok(block_id)
            },
        )
    }

    ///returns a list of blocks ordered by frac_index
    pub fn insert_new_block_with_preexisting_id(
        self: Arc<Self>,
        block_id: String,
        block_content: String,
        block_meta_data: String,
        position_to_insert: PositionToInsert,
    ) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "insert_new_block_with_preexisting_id",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "insert_new_block_with_preexisting_id"),
                })?;
                {
                    let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                    let txn = doc.transact();
                    if find_block_with_id(&array, &txn, &block_id).is_ok() {
                        return Err(YrsError::GenericError {
                            info: error_info(
                                format!("block id {block_id} already exists in this page"),
                                "insert_new_block_with_preexisting_id",
                            ),
                        });
                    }
                }
                insert_block_into_doc(&doc, block_id, block_content, block_meta_data, position_to_insert)
            },
        )
    }

    pub fn get_entire_page(self: Arc<Self>) -> Result<Vec<Block>, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "get_entire_page",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "get_entire_page"),
                })?;
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();

                let len = array.len(&txn) as usize;
                let mut with_keys: Vec<(String, Block)> = Vec::with_capacity(len);
                for position_by_regular_counting in 0..len {
                    with_keys.push(read_block_at(&array, &txn, position_by_regular_counting)?);
                }
                with_keys.sort_by(|a, b| {
                    a.0.cmp(&b.0)
                        .then_with(|| a.1.id_in_yrs.cmp(&b.1.id_in_yrs))
                });
                Ok(with_keys.into_iter().map(|(_, b)| b).collect())
            },
        )
    }

    pub fn show_doc_info(self: Arc<Self>) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "show_doc_info",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "show_doc_info"),
                })?;
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                let json_representation = array.to_json(&txn);
                println!("data blocks represented as json:");
                println!("{}", json_representation);
                Ok(())
            },
        )
    }

    pub fn edit_text_block(
        self: Arc<Self>,
        block_id: String,
        text_edit: TextEdit,
        edit_target: EditTarget,
    ) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "edit_text_block_insert",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "edit_text_block_insert"),
                })?;
                // Use new helper
                let block =
                    get_block_from_id(&doc, &block_id).ok_or_else(|| YrsError::GenericError {
                        info: error_info(
                            format!("found no block with id: {block_id}"),
                            "edit_text_block_insert",
                        ),
                    })?;

                edit_block(&doc, block, text_edit, edit_target)
            },
        )
    }

    pub fn delete_block(self: Arc<Self>, block_id: String) -> Result<(), YrsError> {
        let do_delete = move || -> Result<(), YrsError> {
            let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                info: error_info("lock poisoned", "delete_block"),
            })?;
            let position_by_regular_counting = get_block_index_from_id(&doc, &block_id)
                .ok_or_else(|| YrsError::GenericError {
                    info: error_info(
                        format!("found no block with id: {block_id}"),
                        "delete_block",
                    ),
                })?;
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let mut txn = doc.transact_mut();
            array.remove(&mut txn, position_by_regular_counting);
            Ok(())
        };

        prevent_deadlock(
            DeadlockCtx::new(
                "delete_block",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            do_delete,
        )
    }

    pub fn snapshot(self: Arc<Self>) -> Result<Vec<u8>, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "snapshot",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "snapshot"),
                })?;
                Ok(doc.transact().encode_diff_v1(&StateVector::default()))
            },
        )
    }

    pub fn merge_with_snapshot(self: Arc<Self>, snapshot: Vec<u8>) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "merge_with_snapshot",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "merge_with_snapshot"),
                })?;
                let update = Update::decode_v1(&snapshot).map_err(|e| {
                    yrs_error(
                        format!("merge_with_snapshot: failed to decode update: {e}"),
                        "merge_with_snapshot",
                    )
                })?;
                doc.transact_mut().apply_update(update).map_err(|e| {
                    yrs_error(
                        format!("merge_with_snapshot: failed to apply update: {e}"),
                        "merge_with_snapshot",
                    )
                })?;
                Ok(())
            },
        )
    }

    pub fn merge_with(self: Arc<Self>, other: Arc<BossOfYrs>) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "merge_with",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let sv = {
                    let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                        info: error_info("lock poisoned", "merge_with"),
                    })?;
                    doc.transact().state_vector()
                };
                let diff = {
                    let other_doc = other.doc.read().map_err(|_| YrsError::GenericError {
                        info: error_info("lock poisoned", "merge_with"),
                    })?;
                    other_doc.transact().encode_diff_v1(&sv)
                };
                let update = Update::decode_v1(&diff).map_err(|e| {
                    yrs_error(
                        format!("merge_with: failed to decode update: {e}"),
                        "merge_with",
                    )
                })?;
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "merge_with"),
                })?;
                doc.transact_mut().apply_update(update).map_err(|e| {
                    yrs_error(
                        format!("merge_with: failed to apply update: {e}"),
                        "merge_with",
                    )
                })?;
                Ok(())
            },
        )
    }



    pub fn move_block(
        self: Arc<Self>,
        block_id: String,
        block_id_of_above: Option<String>,
        block_id_of_below: Option<String>,
    ) -> Result<(), YrsError> {
        if block_id_of_above.as_deref() == Some(block_id.as_str()) {
            return Err(YrsError::GenericError {
                info: error_info(
                    "move_block: block_id_of_above equals block_id",
                    "move_block",
                ),
            });
        }
        if block_id_of_below.as_deref() == Some(block_id.as_str()) {
            return Err(YrsError::GenericError {
                info: error_info(
                    "move_block: block_id_of_below equals block_id",
                    "move_block",
                ),
            });
        }

        prevent_deadlock(
            DeadlockCtx::new(
                "move_block",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "move_block"),
                })?;
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let mut txn = doc.transact_mut();

                let moving_map = find_block_with_id(&array, &txn, &block_id)?;

                let new_frac = compute_new_frac_position_in(
                    &array,
                    &txn,
                    block_id_of_above.as_deref(),
                    block_id_of_below.as_deref(),
                )?;

                moving_map.insert(&mut txn, FRAC_POSITION_KEY.to_string(), In::from(new_frac));
                Ok(())
            },
        )
    }

    pub fn get_frac_position_of_block(
        self: Arc<Self>,
        block_id: String,
    ) -> Result<String, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "get_frac_position_of_block",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "get_frac_position_of_block"),
                })?;
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                get_frac_position_by_id(&array, &txn, &block_id)
            },
        )
    }

    pub fn read_block_meta(self: Arc<Self>, block_id: String) -> Result<Option<String>, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "read_block_meta",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "read_block_meta"),
                })?;
                let block = get_block_from_id(&doc, &block_id);

                match block {
                    None => Ok(None),
                    Some(block_map) => {
                        let txn = doc.transact();
                        let meta = block_map
                            .get(&txn, &META_KEY)
                            .ok_or_else(|| YrsError::GenericError {
                                info: error_info("read_block_meta: no meta field", "read_block_meta"),
                            })?
                            .cast::<String>()
                            .map_err(|_| YrsError::GenericError {
                                info: error_info(
                                    "read_block_meta: failed to cast meta field to String",
                                    "read_block_meta",
                                ),
                            })?;
                        Ok(Some(meta))
                    }
                }
            },
        )
    }

    pub fn read_block(self: Arc<Self>, block_id: String) -> Result<Option<String>, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "read_block",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "read_block"),
                })?;
                // Use new helper
                let block = get_block_from_id(&doc, &block_id);

                match block {
                    None => Ok(None),
                    Some(block_map) => {
                        let txn = doc.transact();
                        let text = block_map
                            .get(&txn, &CONTENT_KEY)
                            .ok_or_else(|| YrsError::GenericError {
                                info: error_info("read_block: no content field", "read_block"),
                            })?
                            .cast::<XmlTextRef>()
                            .map_err(|_| YrsError::GenericError {
                                info: error_info(
                                    "read_block: failed to cast content field to XmlTextRef",
                                    "read_block",
                                ),
                            })?
                            .get_string(&txn);
                        Ok(Some(text))
                    }
                }
            },
        )
    }
}

#[uniffi::export]
pub fn generate_unique_key(user_id: &str, time: &str) -> String {
    let unique_time = format!("{}-{}", time, COUNTER.fetch_add(1, Ordering::SeqCst));

    let mut rng = rand::rng();
    let random_part: u64 = rng.random();

    format!("{unique_time}-{random_part}-{user_id}")
}

#[uniffi::export]
pub fn doc_from_snapshot(
    snapshot: Vec<u8>,
    user_id: String,
    page_id: String,
    time: String,
) -> Result<BossOfYrs, YrsError> {
    prevent_deadlock(
        DeadlockCtx::new(
            "doc_from_snapshot",
            file!(),
            DeadlockPrediction::ProbablyJustADeadlock,
        ),
        move || {
            let doc = yrs::Doc::new();
            let update = Update::decode_v1(&snapshot).map_err(|e| {
                yrs_error(
                    format!("doc_from_snapshot: failed to decode update: {e}"),
                    "doc_from_snapshot",
                )
            })?;
            doc.transact_mut().apply_update(update).map_err(|e| {
                yrs_error(
                    format!("doc_from_snapshot: failed to apply update: {e}"),
                    "doc_from_snapshot",
                )
            })?;
            Ok(BossOfYrs {
                doc: RwLock::new(doc),
                page_id,
                user_id,
                time,
            })
        },
    )
}

#[uniffi::export]
pub fn create_bookmark(boss: Arc<BossOfYrs>) -> Result<Vec<u8>, YrsError> {
    prevent_deadlock(
        DeadlockCtx::new(
            "create_bookmark",
            file!(),
            DeadlockPrediction::ProbablyJustADeadlock,
        ),
        move || {
            let doc = boss.doc.read().map_err(|_| YrsError::GenericError {
                info: error_info("lock poisoned", "create_bookmark"),
            })?;
            Ok(doc.transact().state_vector().encode_v1())
        },
    )
}

#[uniffi::export]
pub fn generate_diff_snapshot(
    boss: Arc<BossOfYrs>,
    bookmark_serialized: Vec<u8>,
) -> Result<Vec<u8>, YrsError> {
    prevent_deadlock(
        DeadlockCtx::new(
            "generate_diff_snapshot",
            file!(),
            DeadlockPrediction::ProbablyJustADeadlock,
        ),
        move || {
            let bookmark = deserialize_bookmark(&bookmark_serialized)?;
            let doc = boss.doc.read().map_err(|_| YrsError::GenericError {
                info: error_info("lock poisoned", "generate_diff_snapshot"),
            })?;
            Ok(doc.transact().encode_diff_v1(&bookmark))
        },
    )
}

fn deserialize_bookmark(bytes: &[u8]) -> Result<StateVector, YrsError> {
    StateVector::decode_v1(bytes).map_err(|e| {
        yrs_error(
            format!("deserialize_bookmark: failed to decode: {e}"),
            "deserialize_bookmark",
        )
    })
}

#[derive(uniffi::Enum, Serialize, Deserialize, Clone)]
pub enum PositionToInsert {
    AtEnd,
    Between {
        block_id_of_above: Option<String>,
        block_id_of_below: Option<String>,
    },
}

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]

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

use rand::prelude::*;

use crate::anti_deadlock::{DeadlockCtx, DurationSettings, prevent_deadlock};
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
}

#[derive(uniffi::Record)]
pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}

#[derive(uniffi::Enum)]
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

#[derive(uniffi::Enum)]
pub enum EditTarget {
    Text,
    Meta,
}

const BLOCKS_KEY: &str = "blocks";
const CONTENT_KEY: &str = "text";
const META_KEY: &str = "meta";
const ID_KEY: &str = "id";

fn generate_unique_key(user_id: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();

    let mut rng = rand::rng();
    let random_part: u64 = rng.random();

    format!("{timestamp}-{random_part}-{user_id}")
}

fn block_from_block_id(doc: &Doc, block_id: &str, array_ref: ArrayRef) -> Option<MapRef> {
    let txn = doc.transact();
    for block in array_ref.iter(&txn) {
        if let Ok(block_map) = block.cast::<MapRef>() {
            let id = block_map
                .get(&txn, ID_KEY)
                .and_then(|v| v.cast::<String>().ok());
            if id.as_deref() == Some(block_id) {
                return Some(block_map);
            }
        }
    }
    None
}

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

#[uniffi::export]
impl BossOfYrs {
    #[uniffi::constructor]
    pub fn new(user_id: String) -> Self {
        Self {
            doc: RwLock::new(Doc::new()),
            page_id: generate_unique_key(&user_id),
            user_id,
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
    ) -> Result<(), YrsError> {
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
                let block_id = generate_unique_key(&self.user_id);

                let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
                let yrs_array_ref = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let mut txn = doc.transact_mut();

                yrs_array_ref.push_back(
                    &mut txn,
                    MapPrelim::from([
                        (ID_KEY.to_string(), In::from(block_id)),
                        (CONTENT_KEY.to_string(), In::from(text_as_xml_text_ref)),
                        (META_KEY.to_string(), In::from(block_meta_data)),
                    ]),
                );
                Ok(())
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

                if array.len(&txn) == 0 {
                    return Err(YrsError::GenericError {
                        info: error_info(
                            format!("no page found for id: {BLOCKS_KEY}"),
                            "get_entire_page",
                        ),
                    });
                }

                let mut result = Vec::new();
                for block in array.iter(&txn) {
                    if let Ok(block_map) = block.cast::<MapRef>() {
                        let text = block_map
                            .get(&txn, &CONTENT_KEY)
                            .and_then(|v| v.cast::<XmlTextRef>().ok())
                            .map(|t| t.get_string(&txn))
                            .ok_or_else(|| YrsError::GenericError {
                                info: error_info(
                                    "read_page: failed to cast content field to XmlTextRef",
                                    "get_entire_page",
                                ),
                            })?;
                        let meta = block_map
                            .get(&txn, &META_KEY)
                            .and_then(|v| v.cast::<String>().ok())
                            .ok_or_else(|| YrsError::GenericError {
                                info: error_info(
                                    "read_page: failed to cast meta field to String",
                                    "get_entire_page",
                                ),
                            })?;
                        let id_in_yrs = block_map
                            .get(&txn, &ID_KEY)
                            .and_then(|v| v.cast::<String>().ok())
                            .ok_or_else(|| YrsError::GenericError {
                                info: error_info(
                                    "read_page: failed to cast id field to String",
                                    "get_entire_page",
                                ),
                            })?;

                        result.push(Block {
                            text,
                            metadata: meta,
                            id_in_yrs,
                        });
                    }
                }
                Ok(result)
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

    pub fn edit_text_block_insert(
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
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let block = block_from_block_id(&doc, &block_id, array).ok_or_else(|| {
                    YrsError::GenericError {
                        info: error_info(
                            format!("found no block with id: {block_id}"),
                            "edit_text_block_insert",
                        ),
                    }
                })?;

                edit_block(&doc, block, text_edit, edit_target)
            },
        )
    }

    pub fn get_user_id(self: Arc<Self>) -> Result<u64, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "get_user_id",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "get_user_id"),
                })?;
                Ok(doc.client_id().get())
            },
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
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let block = block_from_block_id(&doc, &block_id, array);

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
pub fn doc_from_snapshot(
    snapshot: Vec<u8>,
    user_id: String,
    page_id: String,
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
            })
        },
    )
}

#[uniffi::export]
pub fn create_bookmark_of_synced_state(boss: Arc<BossOfYrs>) -> Result<Vec<u8>, YrsError> {
    prevent_deadlock(
        DeadlockCtx::new(
            "create_bookmark_of_synced_state",
            file!(),
            DeadlockPrediction::ProbablyJustADeadlock,
        ),
        move || {
            let doc = boss.doc.read().map_err(|_| YrsError::GenericError {
                info: error_info("lock poisoned", "create_bookmark_of_synced_state"),
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

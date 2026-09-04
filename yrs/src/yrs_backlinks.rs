use std::sync::{Arc, RwLock};
use yrs::updates::decoder::Decode;
use yrs::{Doc, In, Map, MapRef, ReadTxn, StateVector, Transact, Update};

use crate::anti_deadlock::{DeadlockCtx, prevent_deadlock};
use crate::yrs_error::{DeadlockPrediction, ErrorInfo, YrsError};

const BACKLINKS_KEY: &str = "backlinks";

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
pub struct YrsBacklinks {
    doc: RwLock<Doc>,
}

#[uniffi::export]
impl YrsBacklinks {
    #[uniffi::constructor]
    pub fn new_empty() -> Self {
        Self {
            doc: RwLock::new(Doc::new()),
        }
    }

    #[uniffi::constructor]
    pub fn new(loaded_from_db: Vec<u8>) -> Result<Self, YrsError> {
        let doc = Doc::new();
        if !loaded_from_db.is_empty() {
            let update = Update::decode_v1(&loaded_from_db).map_err(|e| {
                yrs_error(format!("Failed to decode backlinks snapshot: {e}"), "new")
            })?;
            doc.transact_mut().apply_update(update).map_err(|e| {
                yrs_error(format!("Failed to apply backlinks snapshot: {e}"), "new")
            })?;
        }
        Ok(Self {
            doc: RwLock::new(doc),
        })
    }

    pub fn add_backlink(
        self: Arc<Self>,
        owner_of_backlink_id: String,
        page_im_linking_to_id: String,
    ) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "add_backlink",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "add_backlink"),
                })?;
                let backlinks_map = doc.get_or_insert_map(BACKLINKS_KEY);
                let mut txn = doc.transact_mut();
                let inner_map: MapRef =
                    backlinks_map.get_or_init(&mut txn, page_im_linking_to_id.as_str());
                inner_map.insert(&mut txn, owner_of_backlink_id, In::from(true));
                Ok(())
            },
        )
    }

    pub fn remove_backlink(
        self: Arc<Self>,
        owner_of_backlink_id: String,
        page_im_linking_to_id: String,
    ) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "remove_backlink",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "remove_backlink"),
                })?;
                let backlinks_map = doc.get_or_insert_map(BACKLINKS_KEY);
                let mut txn = doc.transact_mut();

                if let Some(inner_map_ref) = backlinks_map.get(&txn, &page_im_linking_to_id) {
                    if let Ok(inner_map) = inner_map_ref.cast::<MapRef>() {
                        inner_map.remove(&mut txn, &owner_of_backlink_id);
                        if inner_map.len(&txn) == 0 {
                            backlinks_map.remove(&mut txn, &page_im_linking_to_id);
                        }
                    }
                }
                Ok(())
            },
        )
    }

    pub fn get_backlinks_for_page(
        self: Arc<Self>,
        page_id: String,
    ) -> Result<Vec<String>, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "get_backlinks_for_page",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "get_backlinks_for_page"),
                })?;
                let backlinks_map = doc.get_or_insert_map(BACKLINKS_KEY);
                let txn = doc.transact();
                let mut result = Vec::new();

                if let Some(inner_map_ref) = backlinks_map.get(&txn, &page_id) {
                    if let Ok(inner_map) = inner_map_ref.cast::<MapRef>() {
                        for (key, _) in inner_map.iter(&txn) {
                            result.push(key.to_string());
                        }
                    }
                }
                Ok(result)
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
}

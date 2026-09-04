use std::sync::{Arc, RwLock};
use yrs::updates::decoder::Decode;
use yrs::{Doc, In, Map, ReadTxn, StateVector, Transact, Update};

use crate::anti_deadlock::{prevent_deadlock, DeadlockCtx};
use crate::yrs_error::{DeadlockPrediction, ErrorInfo, YrsError};

const ACTIVE_PAGES_KEY: &str = "active_pages";

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
pub struct YrsActivePages {
    doc: RwLock<Doc>,
}

#[uniffi::export]
impl YrsActivePages {
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
                yrs_error(
                    format!("Failed to decode active pages snapshot: {e}"),
                    "new",
                )
            })?;
            doc.transact_mut().apply_update(update).map_err(|e| {
                yrs_error(format!("Failed to apply active pages snapshot: {e}"), "new")
            })?;
        }
        Ok(Self {
            doc: RwLock::new(doc),
        })
    }

    pub fn mark_page_active(self: Arc<Self>, page_id: String) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "mark_page_active",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "mark_page_active"),
                })?;
                let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
                let mut txn = doc.transact_mut();
                map.insert(&mut txn, page_id, In::from(true));
                Ok(())
            },
        )
    }

    pub fn mark_page_deleted(self: Arc<Self>, page_id: String) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "mark_page_deleted",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "mark_page_deleted"),
                })?;
                let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
                let mut txn = doc.transact_mut();
                map.insert(&mut txn, page_id, In::from(false));
                Ok(())
            },
        )
    }

    pub fn is_page_active(self: Arc<Self>, page_id: String) -> Result<bool, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "is_page_active",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "is_page_active"),
                })?;
                let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
                let txn = doc.transact();
                Ok(match map.get(&txn, &page_id) {
                    Some(value) => value.cast::<bool>().unwrap_or(true),
                    None => true,
                })
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

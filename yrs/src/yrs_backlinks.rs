use std::sync::{Arc, RwLock};
use yrs::updates::decoder::Decode;
use yrs::{Doc, In, Map, ReadTxn, StateVector, Transact, Update};

use crate::anti_deadlock::{DeadlockCtx, prevent_deadlock};
use crate::yrs_error::{DeadlockPrediction, ErrorInfo, YrsError};

const STATE_MAP_KEY: &str = "state";
const DISABLED_KEY: &str = "disabled";

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

    pub fn set_disabled(self: Arc<Self>, disabled: bool) -> Result<(), YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "set_disabled",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.write().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "set_disabled"),
                })?;
                let state_map = doc.get_or_insert_map(STATE_MAP_KEY);
                let mut txn = doc.transact_mut();
                state_map.insert(&mut txn, DISABLED_KEY, disabled);
                Ok(())
            },
        )
    }

    pub fn is_disabled(self: Arc<Self>) -> Result<bool, YrsError> {
        prevent_deadlock(
            DeadlockCtx::new(
                "is_disabled",
                file!(),
                DeadlockPrediction::ProbablyJustADeadlock,
            ),
            move || {
                let doc = self.doc.read().map_err(|_| YrsError::GenericError {
                    info: error_info("lock poisoned", "is_disabled"),
                })?;
                let state_map = doc.get_or_insert_map(STATE_MAP_KEY);
                let txn = doc.transact();
                match state_map.get(&txn, DISABLED_KEY) {
                    Some(value) => value.cast::<bool>().map_err(|_| YrsError::GenericError {
                        info: error_info("failed to cast disabled to bool", "is_disabled"),
                    }),
                    None => Ok(false), // default: not disabled
                }
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
